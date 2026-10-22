//! Xvfb launcher and a minimal X11 clipboard client for the session tests.
//!
//! The client is deliberately independent of `rstory::backend::x11`, so a bug in the backend
//! cannot hide behind the same bug in the test peer. All text is synthetic.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt, CreateWindowAux, EventMask, PropMode, PropertyNotifyEvent,
    SELECTION_NOTIFY_EVENT, SelectionNotifyEvent, SelectionRequestEvent, WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// A private Xvfb. Killed on drop.
pub struct Xvfb {
    child: Child,
    /// `:N`
    pub display: String,
}

impl Xvfb {
    /// Start Xvfb on a free display; returns once it accepts connections.
    pub fn start() -> Res<Self> {
        let mut child = Command::new("Xvfb")
            .args([
                "-displayfd",
                "1",
                "-nolisten",
                "tcp",
                "-ac",
                "-screen",
                "0",
                "1024x768x24",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let out = child.stdout.take().ok_or("no stdout")?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let _ = BufReader::new(out).read_line(&mut line);
            let _ = tx.send(line);
        });
        let line = rx
            .recv_timeout(Duration::from_secs(15))
            .map_err(|_| "Xvfb did not report a display")?;
        let n: u32 = line
            .trim()
            .parse()
            .map_err(|_| "Xvfb printed no display number")?;
        Ok(Self {
            child,
            display: format!(":{n}"),
        })
    }

    /// Kill the server (simulates a lost connection).
    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// Open a client connection.
    pub fn connect(&self) -> Res<Client> {
        Client::connect(&self.display, None)
    }
}

impl Drop for Xvfb {
    fn drop(&mut self) {
        self.kill();
    }
}

/// A plain connection plus a window, able to read selections.
pub struct Client {
    pub conn: RustConnection,
    pub win: u32,
    root: u32,
}

fn atom(conn: &RustConnection, name: &str) -> Res<u32> {
    Ok(conn.intern_atom(false, name.as_bytes())?.reply()?.atom)
}

fn name_of(conn: &RustConnection, a: u32) -> Res<String> {
    Ok(String::from_utf8_lossy(&conn.get_atom_name(a)?.reply()?.name).into_owned())
}

/// Read the Xauthority cookie for `display` (":N") from `path`, if the file has one.
pub fn auth_for(display: &str, path: &std::path::Path) -> Option<(Vec<u8>, Vec<u8>)> {
    let data = std::fs::read(path).ok()?;
    let want = display
        .trim_start_matches(':')
        .split('.')
        .next()?
        .as_bytes()
        .to_vec();
    let mut i = 0;
    let take = |i: &mut usize, n: usize| -> Option<&[u8]> {
        let s = data.get(*i..*i + n)?;
        *i += n;
        Some(s)
    };
    let mut found = None;
    while i < data.len() {
        take(&mut i, 2)?; // family
        let n = u16::from_be_bytes(take(&mut i, 2)?.try_into().ok()?) as usize;
        take(&mut i, n)?; // address
        let n = u16::from_be_bytes(take(&mut i, 2)?.try_into().ok()?) as usize;
        let number = take(&mut i, n)?.to_vec();
        let n = u16::from_be_bytes(take(&mut i, 2)?.try_into().ok()?) as usize;
        let name = take(&mut i, n)?.to_vec();
        let n = u16::from_be_bytes(take(&mut i, 2)?.try_into().ok()?) as usize;
        let value = take(&mut i, n)?.to_vec();
        if number.is_empty() || number == want {
            found = Some((name, value));
            if !number.is_empty() {
                break;
            }
        }
    }
    found
}

impl Client {
    /// Connect to `display` (":N"), with an Xauthority file when the server needs one.
    pub fn connect(display: &str, xauthority: Option<&std::path::Path>) -> Res<Self> {
        let num = display
            .trim_start_matches(':')
            .split('.')
            .next()
            .ok_or("bad display")?;
        let stream = std::os::unix::net::UnixStream::connect(format!("/tmp/.X11-unix/X{num}"))?;
        let stream = x11rb::rust_connection::DefaultStream::from_unix_stream(stream)?.0;
        let (name, data) = xauthority
            .and_then(|p| auth_for(display, p))
            .unwrap_or_default();
        let conn = RustConnection::connect_to_stream_with_auth_info(stream, 0, name, data)?;
        let root = conn.setup().roots[0].root;
        let win = conn.generate_id()?;
        conn.create_window(
            0,
            win,
            root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
        )?;
        conn.flush()?;
        Ok(Self { conn, win, root })
    }

    /// Window id that owns CLIPBOARD, 0 for none.
    pub fn clipboard_owner(&self) -> Res<u32> {
        let cb = atom(&self.conn, "CLIPBOARD")?;
        Ok(self.conn.get_selection_owner(cb)?.reply()?.owner)
    }

    /// Target names offered by the current owner.
    pub fn targets(&self) -> Res<Vec<String>> {
        let data = self.convert("TARGETS", Duration::from_secs(3))?;
        data.as_chunks::<4>()
            .0
            .iter()
            .map(|c| name_of(&self.conn, u32::from_ne_bytes(*c)))
            .collect()
    }

    /// Convert CLIPBOARD to `target` and return the raw bytes (INCR handled).
    pub fn convert(&self, target: &str, timeout: Duration) -> Res<Vec<u8>> {
        let cb = atom(&self.conn, "CLIPBOARD")?;
        let t = atom(&self.conn, target)?;
        let prop = atom(&self.conn, "TEST_PROP")?;
        let incr = atom(&self.conn, "INCR")?;
        self.conn.delete_property(self.win, prop)?;
        self.conn
            .convert_selection(self.win, cb, t, prop, CURRENT_TIME)?;
        self.conn.flush()?;
        let end = Instant::now() + timeout;
        loop {
            match self.conn.poll_for_event()? {
                Some(Event::SelectionNotify(e)) if e.requestor == self.win => {
                    if e.property == 0 {
                        return Err("conversion refused".into());
                    }
                    break;
                }
                Some(_) => {}
                None => {
                    if Instant::now() > end {
                        return Err("timeout waiting for SelectionNotify".into());
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
        let r = self
            .conn
            .get_property(true, self.win, prop, AtomEnum::ANY, 0, u32::MAX / 4)?
            .reply()?;
        if r.type_ != incr {
            return Ok(r.value);
        }
        let mut out = Vec::new();
        let mut end = Instant::now() + timeout;
        loop {
            match self.conn.poll_for_event()? {
                Some(Event::PropertyNotify(PropertyNotifyEvent {
                    window,
                    atom: a,
                    state,
                    ..
                })) if window == self.win
                    && a == prop
                    && state == x11rb::protocol::xproto::Property::NEW_VALUE =>
                {
                    let r = self
                        .conn
                        .get_property(true, self.win, prop, AtomEnum::ANY, 0, u32::MAX / 4)?
                        .reply()?;
                    if r.value.is_empty() {
                        return Ok(out);
                    }
                    out.extend_from_slice(&r.value);
                    end = Instant::now() + timeout;
                }
                Some(_) => {}
                None => {
                    if Instant::now() > end {
                        return Err("timeout during INCR".into());
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
    }
}

/// What an [`Owner`] offers and how it behaves.
#[derive(Clone, Default)]
pub struct Offer {
    /// `(target name, bytes)`; the property type equals the target name.
    pub items: Vec<(String, Vec<u8>)>,
    /// Never answer anything but TARGETS (simulates a hung owner).
    pub stall: bool,
    /// Answer every conversion except TARGETS with "refused".
    pub refuse: bool,
    /// Values longer than this use INCR. 0 means 64 KiB.
    pub incr_threshold: usize,
}

impl Offer {
    /// UTF8_STRING only.
    pub fn utf8(text: &str) -> Self {
        Self {
            items: vec![("UTF8_STRING".into(), text.as_bytes().to_vec())],
            ..Self::default()
        }
    }

    /// STRING only, with raw Latin-1 bytes.
    pub fn latin1(bytes: &[u8]) -> Self {
        Self {
            items: vec![("STRING".into(), bytes.to_vec())],
            ..Self::default()
        }
    }

    /// Add or replace a target.
    pub fn with(mut self, target: &str, bytes: &[u8]) -> Self {
        self.items.retain(|(t, _)| t != target);
        self.items.push((target.into(), bytes.to_vec()));
        self
    }

    /// Advertise targets but refuse to convert any of them.
    pub fn refusing(targets: &[&str]) -> Self {
        Self {
            items: targets.iter().map(|t| ((*t).into(), Vec::new())).collect(),
            refuse: true,
            ..Self::default()
        }
    }
}

struct Transfer {
    data: Vec<u8>,
    pos: usize,
    ty: u32,
    finished: bool,
}

/// A client that owns CLIPBOARD and answers requests on its own thread. Dropping it closes the
/// connection, which is how an owner "exits without handing the selection over".
pub struct Owner {
    stop: Arc<AtomicBool>,
    lost: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    pub window: u32,
}

impl Owner {
    /// Take CLIPBOARD on a fresh connection and start serving `offer`.
    pub fn start(display: &str, offer: Offer) -> Res<Self> {
        Self::start_with_auth(display, None, offer)
    }

    pub fn start_with_auth(
        display: &str,
        auth: Option<&std::path::Path>,
        offer: Offer,
    ) -> Res<Self> {
        let c = Client::connect(display, auth)?;
        let cb = atom(&c.conn, "CLIPBOARD")?;
        c.conn.set_selection_owner(c.win, cb, CURRENT_TIME)?;
        c.conn.flush()?;
        if c.conn.get_selection_owner(cb)?.reply()?.owner != c.win {
            return Err("could not take CLIPBOARD".into());
        }
        let window = c.win;
        let stop = Arc::new(AtomicBool::new(false));
        let lost = Arc::new(AtomicBool::new(false));
        let (s2, l2) = (stop.clone(), lost.clone());
        let thread = std::thread::spawn(move || {
            let _ = serve_loop(c, offer, &s2, &l2);
        });
        Ok(Self {
            stop,
            lost,
            thread: Some(thread),
            window,
        })
    }

    /// Whether another client has taken the selection from this owner.
    pub fn lost(&self) -> bool {
        self.lost.load(Ordering::SeqCst)
    }

    /// Wait until another client took the selection.
    pub fn wait_lost(&self, timeout: Duration) -> bool {
        super::wait_for(timeout, || self.lost().then_some(())).is_some()
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn serve_loop(c: Client, offer: Offer, stop: &AtomicBool, lost: &AtomicBool) -> Res<()> {
    let conn = &c.conn;
    let (targets, timestamp, incr) = (
        atom(conn, "TARGETS")?,
        atom(conn, "TIMESTAMP")?,
        atom(conn, "INCR")?,
    );
    let mut by_atom: HashMap<u32, Vec<u8>> = HashMap::new();
    for (name, bytes) in &offer.items {
        by_atom.insert(atom(conn, name)?, bytes.clone());
    }
    let threshold = if offer.incr_threshold == 0 {
        64 * 1024
    } else {
        offer.incr_threshold
    };
    let mut transfers: HashMap<(u32, u32), Transfer> = HashMap::new();
    while !stop.load(Ordering::SeqCst) {
        let Some(ev) = conn.poll_for_event()? else {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        };
        match ev {
            Event::SelectionClear(_) => lost.store(true, Ordering::SeqCst),
            Event::SelectionRequest(r) if offer.stall && r.target != targets => {}
            Event::SelectionRequest(r) => {
                let prop = if r.property == 0 {
                    r.target
                } else {
                    r.property
                };
                let ok = answer(
                    conn,
                    &r,
                    prop,
                    targets,
                    timestamp,
                    incr,
                    &by_atom,
                    &offer,
                    threshold,
                    &mut transfers,
                )?;
                let ev = SelectionNotifyEvent {
                    response_type: SELECTION_NOTIFY_EVENT,
                    sequence: 0,
                    time: r.time,
                    requestor: r.requestor,
                    selection: r.selection,
                    target: r.target,
                    property: if ok { prop } else { 0 },
                };
                conn.send_event(false, r.requestor, EventMask::NO_EVENT, ev)?;
                conn.flush()?;
            }
            Event::PropertyNotify(p) if p.state == x11rb::protocol::xproto::Property::DELETE => {
                if let Some(t) = transfers.get_mut(&(p.window, p.atom)) {
                    if t.finished {
                        transfers.remove(&(p.window, p.atom));
                    } else {
                        let end = (t.pos + threshold).min(t.data.len());
                        conn.change_property8(
                            PropMode::REPLACE,
                            p.window,
                            p.atom,
                            t.ty,
                            &t.data[t.pos..end],
                        )?;
                        t.finished = t.pos == end;
                        t.pos = end;
                        conn.flush()?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn answer(
    conn: &RustConnection,
    r: &SelectionRequestEvent,
    prop: u32,
    targets: u32,
    timestamp: u32,
    incr: u32,
    by_atom: &HashMap<u32, Vec<u8>>,
    offer: &Offer,
    threshold: usize,
    transfers: &mut HashMap<(u32, u32), Transfer>,
) -> Res<bool> {
    if r.target == targets {
        let mut list: Vec<u32> = vec![targets, timestamp];
        list.extend(by_atom.keys());
        conn.change_property32(PropMode::REPLACE, r.requestor, prop, AtomEnum::ATOM, &list)?;
        return Ok(true);
    }
    if offer.refuse {
        return Ok(false);
    }
    let Some(data) = by_atom.get(&r.target) else {
        return Ok(false);
    };
    if data.len() > threshold {
        conn.change_window_attributes(
            r.requestor,
            &x11rb::protocol::xproto::ChangeWindowAttributesAux::new()
                .event_mask(EventMask::PROPERTY_CHANGE),
        )?;
        conn.change_property32(
            PropMode::REPLACE,
            r.requestor,
            prop,
            incr,
            &[u32::try_from(data.len())?],
        )?;
        transfers.insert(
            (r.requestor, prop),
            Transfer {
                data: data.clone(),
                pos: 0,
                ty: r.target,
                finished: false,
            },
        );
    } else {
        conn.change_property8(PropMode::REPLACE, r.requestor, prop, r.target, data)?;
    }
    Ok(true)
}
