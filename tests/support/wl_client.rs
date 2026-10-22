//! A minimal Wayland data-control client for the session tests (T001): it sets the selection
//! from a source, reads the current selection, and records what the compositor told it.
//!
//! It shares the generated bindings and the dispatch glue with the backend
//! (`rstory::backend::wayland::protocols`) but none of the backend's logic: no engine, no read
//! limits, no serve state. All text here is synthetic.

use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::os::fd::{AsFd, OwnedFd};
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use rstory::backend::wayland::protocols::{Device, Handler, Manager, Offer as DcOffer, Protocol};
use rstory::impl_data_control_dispatch;
use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::pipe::{PipeFlags, pipe_with};
use wayland_client::protocol::wl_callback::WlCallback;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Connection, Dispatch, QueueHandle};

use super::wait_for;

type Res<T> = Result<T, Box<dyn Error>>;

/// What a source offers: MIME type to bytes.
#[derive(Clone, Default)]
pub struct Offer {
    items: Vec<(String, Vec<u8>)>,
    /// Types whose `send` fd is held open and never written (a stuck owner).
    stalled: Vec<String>,
}

impl Offer {
    /// The usual text set, all carrying `text`.
    pub fn utf8(text: &str) -> Self {
        [
            "text/plain;charset=utf-8",
            "text/plain",
            "UTF8_STRING",
            "STRING",
            "TEXT",
        ]
        .iter()
        .fold(Self::default(), |o, m| o.with(m, text.as_bytes()))
    }

    /// Offer `mime` but never answer its `send`: the pipe stays open and empty.
    pub fn stalling(mut self, mime: &str) -> Self {
        self.stalled.push(mime.to_string());
        self
    }

    /// Add or replace one type.
    pub fn with(mut self, mime: &str, bytes: &[u8]) -> Self {
        self.items.retain(|(m, _)| m != mime);
        self.items.push((mime.to_string(), bytes.to_vec()));
        self
    }
}

/// Which protocol the client should bind.
#[derive(Clone, Copy)]
pub enum Proto {
    /// ext if offered, else zwlr.
    Auto,
    Ext,
    Zwlr,
}

#[derive(Default)]
struct Shared {
    /// Types of the current selection offer; `None` for no owner.
    types: Option<Vec<String>>,
    /// Count of selection events seen.
    generation: u64,
    cancelled: bool,
    sends: usize,
}

type Ack = Sender<Result<(), String>>;

enum Cmd {
    Set(Offer, Ack),
    Clear(Ack),
    Receive(String, Sender<Result<OwnedFd, String>>),
    Stop,
}

struct State {
    shared: Arc<Mutex<Shared>>,
    seat: Option<WlSeat>,
    manager_name: Option<(u32, u32, Protocol)>,
    want: Proto,
    current: Option<DcOffer>,
    served: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    stalled: Vec<String>,
    held: Vec<OwnedFd>,
    source_id: u64,
}

impl Handler for State {
    fn selection(&mut self, offer: Option<DcOffer>) {
        if let Some(old) = self.current.take() {
            old.destroy();
        }
        let mut s = self.shared.lock().unwrap();
        s.types = offer.as_ref().map(DcOffer::mimes);
        s.generation += 1;
        drop(s);
        self.current = offer;
    }

    fn finished(&mut self) {}

    fn send(&mut self, _source: u64, mime: String, fd: OwnedFd) {
        self.shared.lock().unwrap().sends += 1;
        if self.stalled.contains(&mime) {
            self.held.push(fd);
            return;
        }
        let data = self.served.lock().unwrap().get(&mime).cloned();
        std::thread::spawn(move || {
            let mut f = std::fs::File::from(fd);
            if let Some(d) = data {
                let _ = std::io::Write::write_all(&mut f, &d);
            }
        });
    }

    fn cancelled(&mut self, source: u64) {
        if source == self.source_id {
            self.shared.lock().unwrap().cancelled = true;
        }
    }
}

impl_data_control_dispatch!(State);

impl Dispatch<WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_seat" if state.seat.is_none() => {
                    state.seat = Some(registry.bind(name, version.min(1), qh, ()));
                }
                i if i == Protocol::Ext.interface() => {
                    if matches!(state.want, Proto::Auto | Proto::Ext) {
                        state.manager_name = Some((name, version, Protocol::Ext));
                    }
                }
                i if i == Protocol::Zwlr.interface() => {
                    let have_ext = matches!(state.manager_name, Some((_, _, Protocol::Ext)));
                    if matches!(state.want, Proto::Zwlr)
                        || (matches!(state.want, Proto::Auto) && !have_ext)
                    {
                        state.manager_name = Some((name, version, Protocol::Zwlr));
                    }
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        _: &WlSeat,
        _: wayland_client::protocol::wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlCallback, ()> for State {
    fn event(
        _: &mut Self,
        _: &WlCallback,
        _: wayland_client::protocol::wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

struct Registry(Vec<String>);

impl Dispatch<WlRegistry, ()> for Registry {
    fn event(
        s: &mut Self,
        _: &WlRegistry,
        e: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { interface, .. } = e {
            s.0.push(interface);
        }
    }
}

/// Interface names a compositor advertises (an independent registry round trip).
pub fn registry_interfaces(socket: &Path) -> Option<Vec<String>> {
    let stream = std::os::unix::net::UnixStream::connect(socket).ok()?;
    let conn = Connection::from_socket(stream).ok()?;
    let mut queue = conn.new_event_queue::<Registry>();
    let _r = conn.display().get_registry(&queue.handle(), ());
    let mut st = Registry(Vec::new());
    queue.roundtrip(&mut st).ok()?;
    Some(st.0)
}

/// A data-control client running on its own thread. Stopped on drop.
pub struct WlClient {
    cmds: Sender<Cmd>,
    wake: OwnedFd,
    shared: Arc<Mutex<Shared>>,
    protocol: Protocol,
    thread: Option<JoinHandle<()>>,
}

impl WlClient {
    /// Connect to the compositor socket at `socket` and bind data-control.
    pub fn connect(socket: &Path, want: Proto) -> Res<Self> {
        let stream = std::os::unix::net::UnixStream::connect(socket)?;
        let conn = Connection::from_socket(stream)?;
        let shared = Arc::new(Mutex::new(Shared::default()));
        let served = Arc::new(Mutex::new(HashMap::new()));
        let mut state = State {
            shared: shared.clone(),
            seat: None,
            manager_name: None,
            want,
            current: None,
            served: served.clone(),
            stalled: Vec::new(),
            held: Vec::new(),
            source_id: 0,
        };
        let mut queue = conn.new_event_queue::<State>();
        let qh = queue.handle();
        let registry = conn.display().get_registry(&qh, ());
        queue.roundtrip(&mut state)?;
        let (name, version, protocol) = state.manager_name.ok_or("no data-control global")?;
        let seat = state.seat.clone().ok_or("no seat")?;
        let manager = Manager::bind(&registry, protocol, name, version, &qh);
        let device = manager.get_data_device(&seat, &qh);
        queue.roundtrip(&mut state)?;

        let (cmds, rx) = mpsc::channel();
        let (wake_r, wake) = pipe_with(PipeFlags::CLOEXEC)?;
        let thread = std::thread::spawn(move || {
            run(
                &conn, &mut queue, &mut state, &manager, &device, &rx, &wake_r,
            );
        });
        Ok(Self {
            cmds,
            wake,
            shared,
            protocol,
            thread: Some(thread),
        })
    }

    /// The protocol that was bound.
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    fn send(&self, cmd: Cmd) {
        self.cmds.send(cmd).expect("client thread alive");
        let _ = rustix::io::write(&self.wake, b"x");
    }

    fn ask(&self, make: impl FnOnce(Ack) -> Cmd) -> Res<()> {
        let (tx, rx) = mpsc::channel();
        self.send(make(tx));
        rx.recv_timeout(Duration::from_secs(5))?.map_err(Into::into)
    }

    /// Become the selection owner offering `offer`; returns once the compositor processed it.
    pub fn set_selection(&self, offer: Offer) -> Res<()> {
        self.ask(|a| Cmd::Set(offer, a))
    }

    /// Clear the selection.
    pub fn clear_selection(&self) -> Res<()> {
        self.ask(Cmd::Clear)
    }

    /// Types of the current selection offer (`None`: no owner).
    pub fn types(&self) -> Option<Vec<String>> {
        self.shared.lock().unwrap().types.clone()
    }

    /// How many selection events this client has seen.
    pub fn generation(&self) -> u64 {
        self.shared.lock().unwrap().generation
    }

    /// Wait for a selection event after generation `after`; returns the new types.
    pub fn wait_change(&self, after: u64, timeout: Duration) -> Option<Option<Vec<String>>> {
        wait_for(timeout, || {
            let s = self.shared.lock().unwrap();
            (s.generation > after).then(|| s.types.clone())
        })
    }

    /// Wait until the current selection offers `mime`.
    pub fn wait_offering(&self, mime: &str, timeout: Duration) -> bool {
        wait_for(timeout, || {
            self.types()
                .is_some_and(|t| t.iter().any(|m| m == mime))
                .then_some(())
        })
        .is_some()
    }

    /// Whether our source was cancelled (another owner took over).
    pub fn cancelled(&self) -> bool {
        self.shared.lock().unwrap().cancelled
    }

    /// Wait for `cancelled`.
    pub fn wait_cancelled(&self, timeout: Duration) -> bool {
        wait_for(timeout, || self.cancelled().then_some(())).is_some()
    }

    /// How many `send` requests our source has answered.
    pub fn sends(&self) -> usize {
        self.shared.lock().unwrap().sends
    }

    /// Ask the current owner for `mime` and return the read end without reading it.
    pub fn receive_fd(&self, mime: &str) -> Res<OwnedFd> {
        let (tx, rx) = mpsc::channel();
        self.send(Cmd::Receive(mime.to_string(), tx));
        rx.recv_timeout(Duration::from_secs(5))?.map_err(Into::into)
    }

    /// Read `mime` from the current owner to EOF, or fail at `timeout`.
    pub fn read(&self, mime: &str, timeout: Duration) -> Res<Vec<u8>> {
        let fd = self.receive_fd(mime)?;
        let end = Instant::now() + timeout;
        let mut f = std::fs::File::from(fd);
        let mut out = Vec::new();
        let mut buf = [0u8; 16 * 1024];
        loop {
            let left = end.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err("read timed out".into());
            }
            let mut pfd = [PollFd::new(&f, PollFlags::IN)];
            let ts = Timespec {
                tv_sec: i64::try_from(left.as_secs()).unwrap_or(i64::MAX),
                tv_nsec: left.subsec_nanos().into(),
            };
            let n = poll(&mut pfd, Some(&ts))?;
            if n == 0 {
                return Err("read timed out".into());
            }
            match f.read(&mut buf)? {
                0 => return Ok(out),
                n => out.extend_from_slice(&buf[..n]),
            }
        }
    }
}

impl Drop for WlClient {
    fn drop(&mut self) {
        let _ = self.cmds.send(Cmd::Stop);
        let _ = rustix::io::write(&self.wake, b"x");
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn run(
    conn: &Connection,
    queue: &mut wayland_client::EventQueue<State>,
    state: &mut State,
    manager: &Manager,
    device: &Device,
    rx: &mpsc::Receiver<Cmd>,
    wake: &OwnedFd,
) {
    let qh = queue.handle();
    let mut source = None;
    loop {
        if queue.dispatch_pending(state).is_err() {
            return;
        }
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                Cmd::Stop => return,
                Cmd::Set(offer, ack) => {
                    state.source_id += 1;
                    let src = manager.create_data_source(state.source_id, &qh);
                    for m in offer.items.iter().map(|(m, _)| m).chain(&offer.stalled) {
                        src.offer(m);
                    }
                    state.stalled = offer.stalled;
                    state.held.clear();
                    *state.served.lock().unwrap() = offer.items.into_iter().collect();
                    state.shared.lock().unwrap().cancelled = false;
                    device.set_selection(Some(&src));
                    if let Some(old) = source.replace(src) {
                        old.destroy();
                    }
                    let _ = ack.send(queue.roundtrip(state).map(drop).map_err(|e| e.to_string()));
                }
                Cmd::Clear(ack) => {
                    device.set_selection(None);
                    let _ = ack.send(queue.roundtrip(state).map(drop).map_err(|e| e.to_string()));
                }
                Cmd::Receive(mime, reply) => {
                    let r = match &state.current {
                        None => Err("no selection".to_string()),
                        Some(o) => pipe_with(PipeFlags::CLOEXEC)
                            .map_err(|e| e.to_string())
                            .map(|(r, w)| {
                                o.receive(&mime, w.as_fd());
                                r
                            }),
                    };
                    let _ = conn.flush();
                    let _ = reply.send(r);
                }
            }
        }
        if conn.flush().is_err() {
            return;
        }
        let Some(guard) = conn.prepare_read() else {
            continue;
        };
        let sock = guard.connection_fd().try_clone_to_owned();
        let Ok(sock) = sock else { return };
        let mut fds = [
            PollFd::new(&sock, PollFlags::IN),
            PollFd::new(wake, PollFlags::IN),
        ];
        let _ = poll(&mut fds, None);
        let socket_ready = fds[0].revents().contains(PollFlags::IN);
        let woken = fds[1].revents().contains(PollFlags::IN);
        if socket_ready {
            if guard.read().is_err() {
                // Either WouldBlock or a dead connection; dispatch_pending reports the latter.
            }
        } else {
            drop(guard);
        }
        if woken {
            let mut sink = [0u8; 64];
            let _ = rustix::io::read(wake, &mut sink);
        }
    }
}
