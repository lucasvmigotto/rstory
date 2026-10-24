//! The e2e harness: a session (Xvfb, headless sway, headless mutter) plus temporary XDG
//! directories, the real `rstory` binary run with an empty environment, a daemon process
//! with its stderr captured, a pty for the picker, and a clipboard "copier" and "reader" per
//! session type. All clipboard text is synthetic.

use std::cell::RefCell;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

use crate::session::mutter::Mutter;
use crate::session::sway::Sway;
use crate::session::wait_for;
use crate::session::wl_client::{self, Proto, WlClient};
use crate::session::x11::{self, Owner, Xvfb};

pub const BIN: &str = env!("CARGO_BIN_EXE_rstory");
pub const SECS5: Duration = Duration::from_secs(5);
pub const SECS15: Duration = Duration::from_secs(15);

/// The content-sensitivity hint a password manager adds to its copies.
pub const HINT: &str = "x-kde-passwordManagerHint";

/// Which headless session a journey runs in.
pub enum Session {
    /// No display at all.
    Bare,
    Xvfb(Xvfb),
    Sway(Sway),
    /// Headless mutter with XWayland: no data-control, so the daemon bridges through X11.
    Bridge(Mutter),
    /// Headless mutter with `--no-x11`: no data-control and no XWayland.
    NoX11(Mutter),
}

/// Temporary state, config and home directories; they outlive one session so a journey can
/// capture in one session and read the same store in another.
pub struct Root(TempDir);

impl Root {
    pub fn new() -> Self {
        let t = tempfile::tempdir().unwrap();
        for d in ["run", "state", "config", "home"] {
            std::fs::create_dir(t.path().join(d)).unwrap();
        }
        Self(t)
    }
}

/// What a finished run left behind.
pub struct Ran {
    pub code: i32,
    pub stdout: Vec<u8>,
    pub stderr: String,
}

impl Ran {
    pub fn out(&self) -> String {
        String::from_utf8(self.stdout.clone()).unwrap()
    }

    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.stdout).unwrap_or_else(|e| panic!("not JSON ({e})"))
    }
}

/// A session, the temp directories and the copier state.
pub struct World {
    pub session: Session,
    root: Root,
    xauth: Option<PathBuf>,
    extra_env: RefCell<Vec<(String, String)>>,
    holders: RefCell<Vec<Owner>>,
    wl: RefCell<Option<WlClient>>,
}

impl World {
    pub fn new(session: Session) -> Self {
        Self::in_root(session, Root::new())
    }

    pub fn bare() -> Self {
        Self::new(Session::Bare)
    }

    pub fn xvfb() -> Self {
        Self::new(Session::Xvfb(Xvfb::start().expect("Xvfb")))
    }

    pub fn sway() -> Self {
        Self::new(Session::Sway(Sway::start()))
    }

    pub fn bridge() -> Self {
        Self::new(Session::Bridge(Mutter::start().expect("mutter")))
    }

    pub fn no_x11() -> Self {
        Self::new(Session::NoX11(Mutter::start_no_x11().expect("mutter")))
    }

    /// A session over the same store and config as `previous`, which is dropped (its session
    /// and daemon must be stopped by then).
    pub fn reuse(session: Session, previous: World) -> Self {
        Self::in_root(session, previous.root)
    }

    fn in_root(session: Session, root: Root) -> Self {
        let xauth = match &session {
            Session::Bridge(m) => Some(m.xauthority().expect("mutter Xauthority")),
            _ => None,
        };
        Self {
            session,
            root,
            xauth,
            extra_env: RefCell::default(),
            holders: RefCell::default(),
            wl: RefCell::default(),
        }
    }

    pub fn root_path(&self) -> &Path {
        self.root.0.path()
    }

    pub fn store_dir(&self) -> PathBuf {
        self.root_path().join("state/rstory")
    }

    pub fn config_home(&self) -> PathBuf {
        self.root_path().join("config")
    }

    /// `$XDG_RUNTIME_DIR` of the journey: the compositor's private directory, or a temp one.
    pub fn runtime_dir(&self) -> PathBuf {
        match &self.session {
            Session::Sway(s) => s.dir().to_path_buf(),
            Session::Bridge(m) | Session::NoX11(m) => m.runtime_dir().to_path_buf(),
            _ => self.root_path().join("run"),
        }
    }

    pub fn socket(&self) -> PathBuf {
        rstory::ipc::server::socket_path(&self.runtime_dir())
    }

    /// The session variables the binary sees.
    pub fn env(&self) -> Vec<(String, String)> {
        let mut v = vec![
            ("XDG_RUNTIME_DIR".into(), s(&self.runtime_dir())),
            ("XDG_STATE_HOME".into(), s(&self.root_path().join("state"))),
            ("XDG_CONFIG_HOME".into(), s(&self.config_home())),
            ("HOME".into(), s(&self.root_path().join("home"))),
            ("LANG".into(), "C.UTF-8".into()),
            ("NO_COLOR".into(), "1".into()),
            ("TERM".into(), "xterm-256color".into()),
            // No terminal, editor or opener can be found, let alone started.
            ("PATH".into(), "/var/empty".into()),
        ];
        v.extend(self.extra_env.borrow().iter().cloned());
        match &self.session {
            Session::Bare => {}
            Session::Xvfb(x) => {
                v.push(("DISPLAY".into(), x.display.clone()));
                v.push(("XDG_SESSION_TYPE".into(), "x11".into()));
            }
            Session::Sway(sw) => {
                v.push(("WAYLAND_DISPLAY".into(), sw.display().into()));
                v.push(("XDG_SESSION_TYPE".into(), "wayland".into()));
            }
            Session::Bridge(m) => {
                v.push(("WAYLAND_DISPLAY".into(), m.wayland_display.clone()));
                v.push(("DISPLAY".into(), m.display.clone()));
                v.push(("XAUTHORITY".into(), s(self.xauth.as_ref().unwrap())));
                v.push(("XDG_SESSION_TYPE".into(), "wayland".into()));
            }
            Session::NoX11(m) => {
                v.push(("WAYLAND_DISPLAY".into(), m.wayland_display.clone()));
                v.push(("XDG_SESSION_TYPE".into(), "wayland".into()));
            }
        }
        v
    }

    /// Adds a variable to every later `rstory` run (e.g. a private D-Bus address).
    pub fn set_env(&self, key: &str, value: &str) {
        self.extra_env.borrow_mut().push((key.into(), value.into()));
    }

    /// `rstory <args>` with an empty environment plus [`World::env`].
    pub fn command(&self, args: &[&str]) -> Command {
        let mut c = Command::new(BIN);
        c.env_clear().envs(self.env()).args(args);
        c
    }

    /// `rstory <args>` run to completion with no stdin.
    pub fn run(&self, args: &[&str]) -> Ran {
        let mut c = self.command(args);
        c.stdin(Stdio::null());
        let o = c.output().unwrap();
        Ran {
            code: o.status.code().unwrap_or(-1),
            stdout: o.stdout,
            stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
        }
    }

    /// `rstory <args> --json`, which must exit 0.
    pub fn json(&self, args: &[&str]) -> Value {
        let r = self.run(args);
        assert_eq!(r.code, 0, "{args:?}: {}", r.stderr);
        r.json()
    }

    pub fn status(&self) -> Value {
        self.json(&["status", "--json"])
    }

    pub fn entries(&self) -> Vec<Value> {
        self.json(&["ls", "--json"])["entries"]
            .as_array()
            .unwrap()
            .clone()
    }

    pub fn previews(&self) -> Vec<String> {
        self.entries()
            .iter()
            .map(|e| e["preview"].as_str().unwrap().to_string())
            .collect()
    }

    /// Starts `rstory daemon` and returns once it answers `status`.
    pub fn start_daemon(&self) -> DaemonProc {
        let log_path = self.root_path().join(format!(
            "daemon-{}.log",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let log = File::create(&log_path).unwrap();
        let mut c = self.command(&["daemon"]);
        let child = c
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(log)
            .spawn()
            .unwrap();
        let mut d = DaemonProc {
            child: Some(child),
            log: log_path,
        };
        let up = wait_for(SECS15, || {
            if let Some(st) = d.child.as_mut().unwrap().try_wait().unwrap() {
                panic!("the daemon exited early ({st}); log:\n{}", d.log_text());
            }
            (self.run(&["status", "--json"]).code != 3).then_some(())
        });
        assert!(up.is_some(), "daemon not up; log:\n{}", d.log_text());
        d
    }

    /// Polls `ls --json` until the newest entry previews `text`.
    pub fn wait_top(&self, text: &str) {
        let ok = wait_for(SECS15, || {
            (self.previews().first().map(String::as_str) == Some(text)).then_some(())
        });
        assert!(
            ok.is_some(),
            "an entry never became the newest one; status: {}",
            self.run(&["status", "--json"]).out()
        );
    }

    /// Polls `ls --json` until the history has `n` entries.
    pub fn wait_count(&self, n: usize) {
        let ok = wait_for(SECS15, || (self.entries().len() == n).then_some(()));
        assert!(ok.is_some(), "the history never reached {n} entries");
    }

    /// Polls `status` until `f` holds; panics with the last status otherwise.
    pub fn wait_status(&self, what: &str, timeout: Duration, f: impl Fn(&Value) -> bool) -> Value {
        let mut last = Value::Null;
        let got = wait_for(timeout, || {
            last = self.status();
            f(&last).then(|| last.clone())
        });
        got.unwrap_or_else(|| panic!("status never showed {what}: {last}"))
    }

    // ------------------------------------------------------------------ the clipboard

    /// A copier takes the clipboard with `text`; a password manager adds the hint type.
    pub fn copy(&self, text: &str) {
        self.copy_with(text, &[]);
    }

    /// Like [`World::copy`], with extra `(type, bytes)` the copier offers.
    pub fn copy_with(&self, text: &str, extra: &[(&str, &[u8])]) {
        match &self.session {
            Session::Xvfb(x) => self.x_owner(&x.display, None, text, extra),
            Session::Bridge(m) => self.x_owner(&m.display, self.xauth.as_deref(), text, extra),
            Session::Sway(sw) => {
                let mut offer = wl_client::Offer::utf8(text);
                for (t, b) in extra {
                    offer = offer.with(t, b);
                }
                let mut slot = self.wl.borrow_mut();
                let c = slot.get_or_insert_with(|| {
                    WlClient::connect(&sw.socket(), Proto::Auto).expect("wayland copier")
                });
                c.set_selection(offer).unwrap();
            }
            _ => panic!("this session has no clipboard to copy into"),
        }
    }

    /// Copies as a password manager does: the text plus the sensitivity hint.
    pub fn copy_sensitive(&self, text: &str) {
        self.copy_with(text, &[(HINT, b"secret")]);
    }

    fn x_owner(&self, display: &str, auth: Option<&Path>, text: &str, extra: &[(&str, &[u8])]) {
        let mut offer = x11::Offer::utf8(text);
        for (t, b) in extra {
            offer = offer.with(t, b);
        }
        let owner = Owner::start_with_auth(display, auth, offer).expect("x11 copier");
        self.holders.borrow_mut().push(owner);
    }

    /// What a paste would give now: the text of the current CLIPBOARD owner.
    pub fn clipboard(&self) -> Vec<u8> {
        match &self.session {
            Session::Xvfb(x) => x11::Client::connect(&x.display, None)
                .unwrap()
                .convert("UTF8_STRING", SECS5)
                .unwrap(),
            Session::Bridge(m) => x11::Client::connect(&m.display, self.xauth.as_deref())
                .unwrap()
                .convert("UTF8_STRING", SECS5)
                .unwrap(),
            Session::Sway(sw) => {
                let c = WlClient::connect(&sw.socket(), Proto::Auto).expect("wayland reader");
                assert!(c.wait_offering("text/plain;charset=utf-8", SECS5));
                c.read("text/plain;charset=utf-8", SECS5).unwrap()
            }
            _ => panic!("this session has no clipboard"),
        }
    }

    /// Waits until the clipboard holds `text` (a restore answers once the daemon owns it, but
    /// the compositor may need a moment to announce the new owner).
    pub fn wait_clipboard(&self, text: &str) {
        let ok = wait_for(SECS5, || {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.clipboard()))
                .ok()
                .filter(|b| b == text.as_bytes())
                .map(|_| ())
        });
        assert!(ok.is_some(), "the clipboard never held the expected text");
    }

    // ---------------------------------------------------------------------- the pty

    /// `rstory <args>` on a pty, 100x30. The caller drives it with expectrl.
    pub fn pty(&self, args: &[&str]) -> expectrl::session::OsSession {
        let mut s = expectrl::session::OsSession::spawn(self.command(args)).expect("pty spawn");
        s.get_process_mut().set_window_size(100, 30).unwrap();
        s.set_expect_timeout(Some(SECS15));
        s
    }

    /// All the files of the store directory, concatenated, for "the canary is nowhere" checks.
    pub fn store_bytes(&self) -> Vec<u8> {
        fn walk(d: &Path, out: &mut Vec<u8>) {
            let Ok(rd) = std::fs::read_dir(d) else { return };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if let Ok(b) = std::fs::read(&p) {
                    out.extend_from_slice(p.to_string_lossy().as_bytes());
                    out.extend_from_slice(&b);
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.store_dir(), &mut out);
        out
    }
}

fn s(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// A running `rstory daemon`; stopped with SIGTERM on drop.
pub struct DaemonProc {
    child: Option<Child>,
    pub log: PathBuf,
}

impl DaemonProc {
    pub fn log_text(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }

    /// SIGTERM, then wait for the exit status.
    pub fn stop(&mut self) -> Option<std::process::ExitStatus> {
        use rustix::process::{Pid, Signal, kill_process};
        let mut child = self.child.take()?;
        if let Some(pid) = Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)) {
            let _ = kill_process(pid, Signal::TERM);
        }
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(Some(st)) = child.try_wait() {
                return Some(st);
            }
            if Instant::now() >= end {
                let _ = child.kill();
                return child.wait().ok();
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for DaemonProc {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The exit code of a pty child, once it has exited.
pub fn pty_exit_code(s: &expectrl::session::OsSession) -> i32 {
    use expectrl::process::unix::WaitStatus;
    let end = Instant::now() + SECS15;
    loop {
        match s.get_process().status() {
            Ok(WaitStatus::Exited(_, code)) => return code,
            Ok(WaitStatus::StillAlive) => {}
            Ok(other) => panic!("the picker ended abnormally: {other:?}"),
            Err(e) => panic!("pty status: {e}"),
        }
        assert!(Instant::now() < end, "the picker never exited");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Fails when `needle` occurs in `hay`.
pub fn assert_absent(hay: &[u8], needle: &str, where_: &str) {
    assert!(
        !hay.windows(needle.len()).any(|w| w == needle.as_bytes()),
        "the canary was found in {where_}"
    );
}
