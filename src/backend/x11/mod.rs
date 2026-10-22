//! X11 backend: XFIXES selection watching, TARGETS-first reads, INCR, serving as owner.
//!
//! Pure Rust through x11rb's `RustConnection` (ADR-0006). Unix sockets only: no network
//! (Constitution III). Types, sizes and timings may be logged; content never.
//!
//! One task owns the connection (`run`). It sleeps on the connection fd (`AsyncFd`), the command
//! channel and the next deadline, so an idle backend never wakes (no polling, Constitution V).

pub mod incr;
pub mod read;
pub mod serve;

use std::collections::HashMap;
use std::future::Future;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use thiserror::Error;
use tokio::io::Interest;
use tokio::net::UnixStream as AsyncUnix;
use tokio::sync::{mpsc, oneshot};
use x11rb::connection::{Connection, RequestConnection};
use x11rb::errors::ReplyError;
use x11rb::protocol::Event;
use x11rb::protocol::xfixes::{ConnectionExt as _, SelectionEventMask};
use x11rb::protocol::xproto::{
    ConnectionExt as _, CreateWindowAux, EventMask, Property, WindowClass,
};
use x11rb::rust_connection::{DefaultStream, RustConnection};

use super::{Backend, BackendEvent, EventStream, Mode, ReadError, ServeError};

/// Why `X11Backend::connect` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ConnectError {
    /// The display could not be reached.
    #[error("the X display is unreachable")]
    Unreachable,
    /// The server lacks XFIXES 2 or newer.
    #[error("the X server has no usable XFIXES")]
    NoXfixes,
}

/// Connection settings.
#[derive(Debug, Clone)]
pub struct X11Options {
    /// `:N` or `unix:N`; `None` reads `$DISPLAY`.
    pub display: Option<String>,
    /// Xauthority file; `None` uses `$XAUTHORITY` or `~/.Xauthority`.
    pub xauthority: Option<PathBuf>,
    /// Bridge mode retries the connection while XWayland starts on demand.
    pub mode: Mode,
    /// How long bridge mode keeps retrying.
    pub connect_timeout: Duration,
    /// Per-step timeout of a read.
    pub read_timeout: Duration,
    /// Served values longer than this use INCR.
    pub incr_threshold: usize,
}

impl X11Options {
    /// Native-mode options for `display`.
    #[must_use]
    pub fn new(display: &str) -> Self {
        Self {
            display: Some(display.to_string()),
            xauthority: None,
            mode: Mode::Native,
            connect_timeout: Duration::from_secs(3),
            read_timeout: Duration::from_secs(2),
            incr_threshold: 256 * 1024,
        }
    }

    /// Use this Xauthority file.
    #[must_use]
    pub fn xauthority(mut self, path: &Path) -> Self {
        self.xauthority = Some(path.to_path_buf());
        self
    }

    /// Bridge mode: retry the connection for up to `connect_timeout`.
    #[must_use]
    pub fn bridge(mut self) -> Self {
        self.mode = Mode::XwaylandBridge;
        self
    }
}

// ---------------------------------------------------------------------------------------------
// Connecting

/// Display number and screen from `:N[.S]` or `unix:N[.S]`. Remote displays are refused.
fn parse_display(display: &str) -> Option<(String, usize)> {
    let rest = display.strip_prefix("unix:").unwrap_or(display);
    let rest = rest.strip_prefix(':')?;
    let (num, screen) = rest.split_once('.').unwrap_or((rest, "0"));
    if num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((num.to_string(), screen.parse().ok()?))
}

/// The MIT cookie for display `num` from an Xauthority file, tolerant of local-family entries.
fn cookie_for(num: &str, path: &Path) -> Option<(Vec<u8>, Vec<u8>)> {
    let data = std::fs::read(path).ok()?;
    let mut i = 0;
    let mut take = |n: usize| -> Option<&[u8]> {
        let s = data.get(i..i + n)?;
        i += n;
        Some(s)
    };
    let mut found = None;
    loop {
        if take(2).is_none() {
            break;
        }
        let mut field = || -> Option<Vec<u8>> {
            let n = u16::from_be_bytes(take(2)?.try_into().ok()?) as usize;
            Some(take(n)?.to_vec())
        };
        let (Some(_addr), Some(number), Some(name), Some(value)) =
            (field(), field(), field(), field())
        else {
            break;
        };
        if number.is_empty() || number == num.as_bytes() {
            found = Some((name, value));
            if !number.is_empty() {
                break;
            }
        }
    }
    found
}

fn auth_path(opts: &X11Options) -> Option<PathBuf> {
    opts.xauthority
        .clone()
        .or_else(|| std::env::var_os("XAUTHORITY").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".Xauthority")))
}

type Opened = (RustConnection, UnixStream, usize);

fn try_open(num: &str, screen: usize, auth: Option<&Path>) -> std::io::Result<Opened> {
    let stream = UnixStream::connect(format!("/tmp/.X11-unix/X{num}"))?;
    let watch = stream.try_clone()?;
    let (stream, _) = DefaultStream::from_unix_stream(stream)?;
    let (name, data) = auth.and_then(|p| cookie_for(num, p)).unwrap_or_default();
    let conn = RustConnection::connect_to_stream_with_auth_info(stream, screen, name, data)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok((conn, watch, screen))
}

/// Blocking connect; bridge mode retries (XWayland may start on the first connection).
fn open(opts: &X11Options) -> Result<Opened, ConnectError> {
    let display = opts
        .display
        .clone()
        .or_else(|| std::env::var("DISPLAY").ok())
        .ok_or(ConnectError::Unreachable)?;
    let (num, screen) = parse_display(&display).ok_or(ConnectError::Unreachable)?;
    let auth = auth_path(opts);
    let end = Instant::now() + opts.connect_timeout;
    let mut delay = Duration::from_millis(25);
    loop {
        match try_open(&num, screen, auth.as_deref()) {
            Ok(o) => return Ok(o),
            Err(_) if opts.mode == Mode::XwaylandBridge && Instant::now() < end => {
                std::thread::sleep(delay);
                delay = (delay * 2).min(Duration::from_millis(250));
            }
            Err(_) => return Err(ConnectError::Unreachable),
        }
    }
}

/// Whether `display` accepts a connection (bridge mode waits for on-demand XWayland).
#[must_use]
pub fn reachable(opts: &X11Options) -> bool {
    open(opts).is_ok()
}

// ---------------------------------------------------------------------------------------------
// Engine

pub(crate) struct Atoms {
    pub clipboard: u32,
    pub targets: u32,
    pub timestamp: u32,
    pub utf8: u32,
    pub text: u32,
    pub plain_utf8: u32,
    pub incr: u32,
    pub prop_targets: u32,
    pub prop_data: u32,
}

/// What we know about the current owner's offer.
pub(crate) enum TargetsState {
    /// No owner.
    None,
    /// TARGETS conversion in flight.
    Pending { deadline: Instant },
    /// Offered type names.
    Known(Vec<String>),
    /// The owner would not tell.
    Failed,
}

pub(crate) struct Selection {
    pub owner: u32,
    pub time: u32,
    pub targets: TargetsState,
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

pub(crate) struct Engine {
    pub conn: Arc<RustConnection>,
    pub win: u32,
    pub atoms: Atoms,
    pub tx: mpsc::UnboundedSender<BackendEvent>,
    pub interned: HashMap<String, u32>,
    pub names: HashMap<u32, String>,
    pub sel: Selection,
    pub read: Option<read::ReadOp>,
    pub serve: serve::ServeState,
    pub read_timeout: Duration,
    pub incr_threshold: usize,
    pub dead: bool,
}

impl Engine {
    pub fn emit(&self, ev: BackendEvent) {
        let _ = self.tx.send(ev);
    }

    /// The connection failed: tell the consumer once and fail everything pending.
    pub fn disconnect(&mut self) {
        if self.dead {
            return;
        }
        self.dead = true;
        if let Some(op) = self.read.take() {
            let _ = op.reply.send(Err(ReadError::Disconnected));
        }
        self.emit(BackendEvent::Disconnected);
    }

    /// Whether a reply error means the connection is gone (as opposed to one bad request).
    pub fn fatal(&mut self, e: &ReplyError) -> bool {
        let fatal = matches!(e, ReplyError::ConnectionError(_));
        if fatal {
            self.disconnect();
        }
        fatal
    }

    pub fn intern(&mut self, name: &str) -> Option<u32> {
        if let Some(a) = self.interned.get(name) {
            return Some(*a);
        }
        let r = self
            .conn
            .intern_atom(false, name.as_bytes())
            .map_err(ReplyError::from)
            .and_then(|c| c.reply());
        match r {
            Ok(r) => {
                self.interned.insert(name.to_string(), r.atom);
                self.names.insert(r.atom, name.to_string());
                Some(r.atom)
            }
            Err(e) => {
                self.fatal(&e);
                None
            }
        }
    }

    pub fn atom_name(&mut self, atom: u32) -> Option<String> {
        if let Some(n) = self.names.get(&atom) {
            return Some(n.clone());
        }
        let r = self
            .conn
            .get_atom_name(atom)
            .map_err(ReplyError::from)
            .and_then(|c| c.reply());
        match r {
            Ok(r) => {
                let n = String::from_utf8_lossy(&r.name).into_owned();
                self.names.insert(atom, n.clone());
                self.interned.insert(n.clone(), atom);
                Some(n)
            }
            Err(e) => {
                self.fatal(&e);
                None
            }
        }
    }

    pub fn start(&mut self) {
        let owner = self
            .conn
            .get_selection_owner(self.atoms.clipboard)
            .map_err(ReplyError::from)
            .and_then(|c| c.reply());
        match owner {
            Ok(o) if o.owner != 0 && o.owner != self.win => self.on_owner_change(o.owner, 0),
            Ok(_) => {}
            Err(e) => {
                self.fatal(&e);
            }
        }
        self.flush();
    }

    pub fn flush(&mut self) {
        if self.conn.flush().is_err() {
            self.disconnect();
        }
    }

    /// XFIXES told us the owner changed.
    fn on_owner_change(&mut self, owner: u32, time: u32) {
        if let Some(op) = self.read.take() {
            let _ = op.reply.send(Err(ReadError::ConversionRefused));
        }
        if owner == 0 {
            self.sel = Selection {
                owner: 0,
                time,
                targets: TargetsState::None,
            };
            self.serve.clear();
            self.emit(BackendEvent::SelectionCleared);
        } else if owner == self.win {
            if self.serve.active() {
                self.serve.set_acquired(time);
                let types = self.serve.type_names();
                self.sel = Selection {
                    owner,
                    time,
                    targets: TargetsState::Known(types.clone()),
                };
                tracing::debug!(
                    types = types.len(),
                    self_owned = true,
                    "x11.selection_changed"
                );
                self.emit(BackendEvent::SelectionChanged {
                    types,
                    self_owned: true,
                });
            }
        } else {
            self.sel = Selection {
                owner,
                time,
                targets: TargetsState::Pending {
                    deadline: Instant::now() + self.read_timeout,
                },
            };
            let failed = self
                .conn
                .convert_selection(
                    self.win,
                    self.atoms.clipboard,
                    self.atoms.targets,
                    self.atoms.prop_targets,
                    time,
                )
                .is_err();
            if failed {
                self.disconnect();
            }
        }
    }

    pub fn handle_event(&mut self, ev: Event) {
        match ev {
            Event::XfixesSelectionNotify(e) if e.selection == self.atoms.clipboard => {
                self.on_owner_change(e.owner, e.selection_timestamp);
            }
            Event::SelectionNotify(e) if e.requestor == self.win => self.on_selection_notify(&e),
            Event::PropertyNotify(e) if e.window == self.win => {
                if e.atom == self.atoms.prop_data && e.state == Property::NEW_VALUE {
                    self.incr_new_value();
                }
            }
            Event::PropertyNotify(e) if e.state == Property::DELETE => {
                self.serve_property_deleted(e.window, e.atom)
            }
            Event::SelectionRequest(e) => self.serve_request(&e),
            Event::SelectionClear(e) if e.owner == self.win => self.serve.clear(),
            _ => {}
        }
    }

    pub fn handle_command(&mut self, cmd: Command) {
        if self.dead {
            match cmd {
                Command::Read { reply, .. } => {
                    let _ = reply.send(Err(ReadError::Disconnected));
                }
                Command::Serve { reply, .. } => {
                    let _ = reply.send(Err(ServeError::Disconnected));
                }
                Command::Stop => {}
            }
            return;
        }
        match cmd {
            Command::Read { limit, reply } => self.cmd_read(limit, reply),
            Command::Serve { text, reply } => {
                let r = self.cmd_serve(text);
                let _ = reply.send(r);
            }
            Command::Stop => self.cmd_stop(),
        }
        self.flush();
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        let t = match self.sel.targets {
            TargetsState::Pending { deadline } => Some(deadline),
            _ => None,
        };
        [t, self.read_deadline(), self.serve.next_deadline()]
            .into_iter()
            .flatten()
            .min()
    }

    pub fn on_deadline(&mut self, now: Instant) {
        if let TargetsState::Pending { deadline } = self.sel.targets
            && now >= deadline
        {
            self.finish_targets(false);
        }
        self.read_deadline_passed(now);
        self.serve_deadline_passed(now);
        self.flush();
    }

    /// Process everything the connection has queued.
    pub fn drain(&mut self) {
        while !self.dead {
            match self.conn.poll_for_event() {
                Ok(Some(ev)) => self.handle_event(ev),
                Ok(None) => break,
                Err(_) => self.disconnect(),
            }
        }
        self.flush();
    }
}

async fn run(mut engine: Engine, mut cmds: mpsc::UnboundedReceiver<Command>, watch: AsyncUnix) {
    engine.start();
    loop {
        engine.drain();
        if engine.dead {
            break;
        }
        let deadline = engine.next_deadline().map(tokio::time::Instant::from_std);
        let timer = async {
            match deadline {
                Some(d) => tokio::time::sleep_until(d).await,
                None => std::future::pending().await,
            }
        };
        tokio::select! {
            biased;
            cmd = cmds.recv() => match cmd {
                Some(c) => engine.handle_command(c),
                None => break,
            },
            ready = watch.readable() => match ready {
                Ok(()) => {
                    // Forget the readiness we just saw; the next edge wakes us again.
                    let _ = watch.try_io(Interest::READABLE, || {
                        Err::<(), _>(std::io::ErrorKind::WouldBlock.into())
                    });
                }
                Err(_) => engine.disconnect(),
            },
            () = timer => engine.on_deadline(Instant::now()),
        }
    }
    // Pending callers learn the connection is gone; a closing handle drops the replies.
    engine.dead = true;
    if let Some(op) = engine.read.take() {
        let _ = op.reply.send(Err(ReadError::Disconnected));
    }
}

// ---------------------------------------------------------------------------------------------
// Handle

/// The X11 implementation of [`Backend`].
pub struct X11Backend {
    events: EventStream,
    cmds: mpsc::UnboundedSender<Command>,
}

fn blocking_setup(conn: &Arc<RustConnection>) -> Result<(u32, Atoms), ConnectError> {
    let q = conn
        .xfixes_query_version(5, 0)
        .map_err(|_| ConnectError::Unreachable)?
        .reply()
        .map_err(|_| ConnectError::NoXfixes)?;
    if q.major_version < 2 {
        return Err(ConnectError::NoXfixes);
    }
    fn unreachable<E>(_: E) -> ConnectError {
        ConnectError::Unreachable
    }
    let intern = |name: &str| -> Result<_, ConnectError> {
        conn.intern_atom(false, name.as_bytes())
            .map_err(unreachable)
    };
    let cookies = [
        "CLIPBOARD",
        "TARGETS",
        "TIMESTAMP",
        "UTF8_STRING",
        "TEXT",
        "text/plain;charset=utf-8",
        "INCR",
        "RSTORY_TARGETS",
        "RSTORY_DATA",
    ]
    .map(intern);
    let mut a = [0u32; 9];
    for (slot, c) in a.iter_mut().zip(cookies) {
        *slot = c?.reply().map_err(|_| ConnectError::Unreachable)?.atom;
    }
    let atoms = Atoms {
        clipboard: a[0],
        targets: a[1],
        timestamp: a[2],
        utf8: a[3],
        text: a[4],
        plain_utf8: a[5],
        incr: a[6],
        prop_targets: a[7],
        prop_data: a[8],
    };
    let root = conn.setup().roots[0].root;
    let win = conn.generate_id().map_err(unreachable)?;
    conn.create_window(
        0,
        win,
        root,
        0,
        0,
        1,
        1,
        0,
        WindowClass::INPUT_ONLY,
        0,
        &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
    )
    .map_err(unreachable)?;
    conn.xfixes_select_selection_input(
        win,
        atoms.clipboard,
        SelectionEventMask::SET_SELECTION_OWNER
            | SelectionEventMask::SELECTION_WINDOW_DESTROY
            | SelectionEventMask::SELECTION_CLIENT_CLOSE,
    )
    .map_err(unreachable)?;
    conn.flush().map_err(unreachable)?;
    Ok((win, atoms))
}

impl X11Backend {
    /// Connect and start watching CLIPBOARD. An owner already present is reported first.
    ///
    /// # Errors
    /// `Unreachable` when the display does not accept a connection (bridge mode waits up to
    /// `connect_timeout`), `NoXfixes` when XFIXES 2+ is missing.
    pub async fn connect(opts: X11Options) -> Result<Self, ConnectError> {
        let read_timeout = opts.read_timeout;
        let incr_threshold = opts.incr_threshold;
        let (conn, watch, _screen, win, atoms, incr_threshold) =
            tokio::task::spawn_blocking(move || {
                let (conn, watch, screen) = open(&opts)?;
                let conn = Arc::new(conn);
                // A served chunk must fit one request.
                let incr_threshold = incr_threshold
                    .min(conn.maximum_request_bytes().saturating_sub(256))
                    .max(1024);
                let (win, atoms) = blocking_setup(&conn)?;
                Ok::<_, ConnectError>((conn, watch, screen, win, atoms, incr_threshold))
            })
            .await
            .map_err(|_| ConnectError::Unreachable)??;
        let watch = AsyncUnix::from_std(watch).map_err(|_| ConnectError::Unreachable)?;
        let (tx, events) = EventStream::channel();
        let (cmds, rx) = mpsc::unbounded_channel();
        let engine = Engine {
            conn,
            win,
            atoms,
            tx,
            interned: HashMap::new(),
            names: HashMap::new(),
            sel: Selection {
                owner: 0,
                time: 0,
                targets: TargetsState::None,
            },
            read: None,
            serve: serve::ServeState::default(),
            read_timeout,
            incr_threshold,
            dead: false,
        };
        tokio::spawn(run(engine, rx, watch));
        Ok(Self { events, cmds })
    }
}

impl Backend for X11Backend {
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
