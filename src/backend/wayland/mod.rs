//! Wayland backend: `ext_data_control_v1` with a `zwlr_data_control_v1` fallback.
//!
//! Pure Rust through `wayland-client`'s own backend (Constitution I). Types, sizes and timings
//! may be logged; content never (Constitution III).

pub mod protocols;
pub mod read;
pub mod serve;

use std::future::Future;
use std::io;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use tokio::io::Interest;
use tokio::net::UnixStream as AsyncUnix;
use tokio::sync::{mpsc, oneshot};
use wayland_client::backend::{ReadEventsGuard, WaylandError};
use wayland_client::protocol::wl_callback::{self, WlCallback};
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_seat::{self, WlSeat};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};

use super::{Backend, BackendEvent, EventStream, ReadError, ServeError};
use crate::impl_data_control_dispatch;
pub use protocols::Protocol;
use protocols::{Handler, Manager, Offer};

/// Why `WaylandBackend::connect` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ConnectError {
    /// The compositor could not be reached.
    #[error("the Wayland display is unreachable")]
    Unreachable,
    /// The compositor offers neither data-control protocol, or the seat is missing.
    #[error("the compositor offers no data-control protocol")]
    NoDataControl,
}

/// Connection settings.
#[derive(Debug, Clone)]
pub struct WaylandOptions {
    /// Socket to connect to; `None` uses `$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY`.
    pub socket: Option<PathBuf>,
    /// Bind only this protocol; `None` prefers `ext`, then `zwlr`.
    pub protocol: Option<Protocol>,
    /// How long one read may take.
    pub read_timeout: Duration,
    /// How long one `send` write to a reader may take.
    pub write_timeout: Duration,
}

impl WaylandOptions {
    /// Defaults: environment socket, either protocol, 2 s reads.
    #[must_use]
    pub fn new() -> Self {
        Self {
            socket: None,
            protocol: None,
            read_timeout: Duration::from_secs(2),
            write_timeout: Duration::from_secs(10),
        }
    }

    /// Connect to this socket path.
    #[must_use]
    pub fn socket(mut self, path: impl Into<PathBuf>) -> Self {
        self.socket = Some(path.into());
        self
    }

    /// Bind only `protocol` (the other one is ignored even if offered).
    #[must_use]
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }
}

impl Default for WaylandOptions {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------------------------
// Engine state

struct Global {
    name: u32,
    interface: String,
    version: u32,
}

pub(crate) struct State {
    tx: mpsc::UnboundedSender<BackendEvent>,
    conn: Connection,
    globals: Vec<Global>,
    manager: Option<Manager>,
    device: Option<protocols::Device>,
    /// The foreign selection offer, if any.
    current: Option<Offer>,
    /// The device has announced its first selection (an empty one is not a change).
    announced: bool,
    epoch: Arc<read::Epoch>,
    serve: serve::ServeState,
    read_timeout: Duration,
    write_timeout: Duration,
    dead: bool,
}

impl State {
    fn emit(&self, ev: BackendEvent) {
        let _ = self.tx.send(ev);
    }

    /// The connection failed: tell the consumer once and fail everything pending.
    fn disconnect(&mut self) {
        if self.dead {
            return;
        }
        self.dead = true;
        self.epoch.kill();
        self.fail_serve();
        self.emit(BackendEvent::Disconnected);
    }
}

impl Handler for State {
    fn selection(&mut self, offer: Option<Offer>) {
        if let Some(old) = std::mem::replace(&mut self.current, offer.clone()) {
            old.destroy();
        }
        self.epoch.bump();
        let first = !std::mem::replace(&mut self.announced, true);
        match offer {
            // No owner at connect time is the starting state, not a change.
            None if first => {}
            None => {
                tracing::debug!("wl.selection_cleared");
                self.emit(BackendEvent::SelectionCleared);
            }
            Some(o) => {
                let types = o.mimes();
                let self_owned = self.serve.claims_echo(&types);
                tracing::debug!(types = %types.join(","), self_owned, "wl.selection_changed");
                self.emit(BackendEvent::SelectionChanged { types, self_owned });
            }
        }
    }

    fn finished(&mut self) {
        // The seat's device is gone; nothing more can be read or served.
        self.disconnect();
    }

    fn send(&mut self, source: u64, mime: String, fd: std::os::fd::OwnedFd) {
        self.source_send(source, &mime, fd);
    }

    fn cancelled(&mut self, source: u64) {
        self.source_cancelled(source);
    }
}

impl_data_control_dispatch!(State);

impl Dispatch<WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        _: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            state.globals.push(Global {
                name,
                interface,
                version,
            });
        }
    }
}

impl Dispatch<WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        _: &WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlCallback, u64> for State {
    fn event(
        state: &mut Self,
        _: &WlCallback,
        event: wl_callback::Event,
        id: &u64,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            state.sync_done(*id);
        }
    }
}

pub(crate) enum Command {
    Read {
        limit: usize,
        reply: oneshot::Sender<Result<Vec<u8>, ReadError>>,
    },
    Serve {
        text: Arc<[u8]>,
        reply: oneshot::Sender<Result<(), ServeError>>,
    },
    Stop,
}

impl State {
    fn handle_command(&mut self, cmd: Command, qh: &QueueHandle<State>) {
        match cmd {
            Command::Read { limit, reply } => self.cmd_read(limit, reply),
            Command::Serve { text, reply } => self.cmd_serve(text, reply, qh),
            Command::Stop => self.cmd_stop(),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Connecting

fn socket_path(opts: &WaylandOptions) -> Option<PathBuf> {
    if let Some(p) = &opts.socket {
        return Some(p.clone());
    }
    let name = std::env::var_os("WAYLAND_DISPLAY").filter(|n| !n.is_empty())?;
    let p = Path::new(&name);
    if p.is_absolute() {
        return Some(p.to_path_buf());
    }
    let dir = std::env::var_os("XDG_RUNTIME_DIR").filter(|d| !d.is_empty())?;
    Some(Path::new(&dir).join(p))
}

struct Ready {
    queue: EventQueue<State>,
    state: State,
    watch: UnixStream,
    protocol: Protocol,
}

/// Blocking connect, registry round trip, binding and the first selection round trip.
fn open(
    opts: &WaylandOptions,
    tx: mpsc::UnboundedSender<BackendEvent>,
) -> Result<Ready, ConnectError> {
    let path = socket_path(opts).ok_or(ConnectError::Unreachable)?;
    let stream = UnixStream::connect(path).map_err(|_| ConnectError::Unreachable)?;
    let watch = stream.try_clone().map_err(|_| ConnectError::Unreachable)?;
    let conn = Connection::from_socket(stream).map_err(|_| ConnectError::Unreachable)?;
    let mut queue = conn.new_event_queue::<State>();
    let qh = queue.handle();
    let registry = conn.display().get_registry(&qh, ());
    let mut state = State {
        tx,
        conn: conn.clone(),
        globals: Vec::new(),
        manager: None,
        device: None,
        current: None,
        announced: false,
        epoch: Arc::default(),
        serve: serve::ServeState::default(),
        read_timeout: opts.read_timeout,
        write_timeout: opts.write_timeout,
        dead: false,
    };
    queue
        .roundtrip(&mut state)
        .map_err(|_| ConnectError::Unreachable)?;

    let find = |interface: &str| state.globals.iter().find(|g| g.interface == interface);
    let protocol = match opts.protocol {
        Some(p) => find(p.interface()).map(|_| p),
        None => [Protocol::Ext, Protocol::Zwlr]
            .into_iter()
            .find(|p| find(p.interface()).is_some()),
    }
    .ok_or(ConnectError::NoDataControl)?;
    let mgr = find(protocol.interface()).ok_or(ConnectError::NoDataControl)?;
    let seat = find("wl_seat").ok_or(ConnectError::NoDataControl)?;
    let seat: WlSeat = registry.bind(seat.name, seat.version.min(1), &qh, ());
    let manager = Manager::bind(&registry, protocol, mgr.name, mgr.version, &qh);
    state.device = Some(manager.get_data_device(&seat, &qh));
    state.manager = Some(manager);
    state.globals.clear();
    // The device announces the current selection right away; this round trip delivers it.
    queue
        .roundtrip(&mut state)
        .map_err(|_| ConnectError::Unreachable)?;
    watch
        .set_nonblocking(true)
        .map_err(|_| ConnectError::Unreachable)?;
    Ok(Ready {
        queue,
        state,
        watch,
        protocol,
    })
}

// ---------------------------------------------------------------------------------------------
// Loop

fn flush(conn: &Connection) -> Result<bool, ()> {
    match conn.flush() {
        Ok(()) => Ok(false),
        Err(WaylandError::Io(e)) if e.kind() == io::ErrorKind::WouldBlock => Ok(true),
        Err(_) => Err(()),
    }
}

fn read_io(guard: ReadEventsGuard) -> io::Result<usize> {
    guard.read().map_err(|e| match e {
        WaylandError::Io(e) => e,
        WaylandError::Protocol(p) => io::Error::other(p.to_string()),
    })
}

/// One task owns the connection. It sleeps on the socket, the command channel and nothing
/// else, so an idle backend never wakes (Constitution V).
async fn run(
    conn: Connection,
    mut queue: EventQueue<State>,
    mut state: State,
    mut cmds: mpsc::UnboundedReceiver<Command>,
    watch: AsyncUnix,
) {
    let qh = queue.handle();
    loop {
        if queue.dispatch_pending(&mut state).is_err() {
            state.disconnect();
        }
        if state.dead {
            break;
        }
        let Ok(want_write) = flush(&conn) else {
            state.disconnect();
            break;
        };
        // Events already queued must be dispatched before sleeping.
        let Some(guard) = conn.prepare_read() else {
            continue;
        };
        drop(guard);
        tokio::select! {
            biased;
            cmd = cmds.recv() => match cmd {
                Some(c) => state.handle_command(c, &qh),
                None => break,
            },
            ready = watch.readable() => match ready {
                Ok(()) => {
                    if let Some(guard) = conn.prepare_read() {
                        // WouldBlock clears the readiness we just saw; anything else is fatal.
                        match watch.try_io(Interest::READABLE, || read_io(guard)) {
                            Ok(_) => {}
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                            Err(_) => state.disconnect(),
                        }
                    }
                }
                Err(_) => state.disconnect(),
            },
            ready = watch.writable(), if want_write => match ready {
                Ok(()) => {
                    let r = watch.try_io(Interest::WRITABLE, || {
                        flush(&conn)
                            .map_err(|()| io::Error::other("flush"))
                            .and_then(|blocked| {
                                if blocked { Err(io::ErrorKind::WouldBlock.into()) } else { Ok(()) }
                            })
                    });
                    if r.is_err_and(|e| e.kind() != io::ErrorKind::WouldBlock) {
                        state.disconnect();
                    }
                }
                Err(_) => state.disconnect(),
            },
        }
        if state.dead {
            break;
        }
    }
    // Pending callers learn the connection is gone; a closing handle drops the replies.
    state.dead = true;
    state.epoch.kill();
    state.fail_serve();
    while let Ok(cmd) = cmds.try_recv() {
        match cmd {
            Command::Read { reply, .. } => {
                let _ = reply.send(Err(ReadError::Disconnected));
            }
            Command::Serve { reply, .. } => {
                let _ = reply.send(Err(ServeError::Disconnected));
            }
            Command::Stop => {}
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Handle

/// The Wayland implementation of [`Backend`].
pub struct WaylandBackend {
    events: EventStream,
    cmds: mpsc::UnboundedSender<Command>,
    protocol: Protocol,
}

impl WaylandBackend {
    /// Connect, bind data-control and start watching the selection. An owner already present is
    /// reported first.
    ///
    /// # Errors
    /// `Unreachable` when the compositor does not accept a connection, `NoDataControl` when it
    /// offers no data-control manager (or not the requested one).
    pub async fn connect(opts: WaylandOptions) -> Result<Self, ConnectError> {
        let (tx, events) = EventStream::channel();
        let conn_opts = opts.clone();
        let setup = tokio::task::spawn_blocking(move || open(&conn_opts, tx));
        // A wedged compositor must not wedge startup.
        let ready = tokio::time::timeout(Duration::from_secs(5), setup)
            .await
            .map_err(|_| ConnectError::Unreachable)?
            .map_err(|_| ConnectError::Unreachable)??;
        let Ready {
            queue,
            state,
            watch,
            protocol,
        } = ready;
        let watch = AsyncUnix::from_std(watch).map_err(|_| ConnectError::Unreachable)?;
        let (cmds, rx) = mpsc::unbounded_channel();
        tokio::spawn(run(state.conn.clone(), queue, state, rx, watch));
        Ok(Self {
            events,
            cmds,
            protocol,
        })
    }

    /// The protocol this connection speaks.
    #[must_use]
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }
}

impl Backend for WaylandBackend {
    fn events(&mut self) -> &mut EventStream {
        &mut self.events
    }

    fn read_text(
        &mut self,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<u8>, ReadError>> + Send {
        let (reply, rx) = oneshot::channel();
        let sent = self.cmds.send(Command::Read { limit, reply }).is_ok();
        async move {
            if !sent {
                return Err(ReadError::Disconnected);
            }
            rx.await.unwrap_or(Err(ReadError::Disconnected))
        }
    }

    fn serve(&mut self, text: Arc<[u8]>) -> impl Future<Output = Result<(), ServeError>> + Send {
        let (reply, rx) = oneshot::channel();
        let sent = self.cmds.send(Command::Serve { text, reply }).is_ok();
        async move {
            if !sent {
                return Err(ServeError::Disconnected);
            }
            rx.await.unwrap_or(Err(ServeError::Disconnected))
        }
    }

    fn stop_serving(&mut self) {
        let _ = self.cmds.send(Command::Stop);
    }
}
