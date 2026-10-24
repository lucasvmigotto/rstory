//! Shared pieces of the performance suite: the real `rstory daemon` as a child process on a
//! private Xvfb, a clipboard copier that can re-copy fast, and /proc readings of the child.
//!
//! The daemon is measured from outside: RSS and wake-ups come from `/proc/<pid>` of the child,
//! never from the test process. All clipboard text is synthetic.
#![allow(dead_code)]

use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt, PropMode, SELECTION_NOTIFY_EVENT, SelectionNotifyEvent,
};
use x11rb::wrapper::ConnectionExt as _;

use crate::support::daemon::{Client, Dirs};
use crate::support::x11::{Client as XClient, Xvfb};

pub const MIB: u64 = 1024 * 1024;

/// CI runners are shared and noisy: the documented CI budget applies there (test-strategy
/// section 6), the exact budget everywhere else. GitHub sets `CI=true`.
pub fn in_ci() -> bool {
    std::env::var_os("CI").is_some_and(|v| !v.is_empty())
}

/// `local` ms on a developer machine, `ci` ms on a CI runner.
pub fn budget(local_ms: u64, ci_ms: u64) -> Duration {
    Duration::from_millis(if in_ci() { ci_ms } else { local_ms })
}

/// The p-th percentile (0..=100) of a non-empty sample, nearest rank.
pub fn percentile(samples: &mut [Duration], p: usize) -> Duration {
    assert!(!samples.is_empty());
    samples.sort();
    let rank = (samples.len() * p).div_ceil(100).max(1);
    samples[rank - 1]
}

/// A private Xvfb, a temporary store and runtime dir, and the real `rstory daemon` on them.
pub struct DaemonProc {
    pub x: Xvfb,
    pub dirs: Dirs,
    child: Child,
}

impl DaemonProc {
    pub async fn start() -> Self {
        let x = Xvfb::start().expect("Xvfb");
        let dirs = Dirs::new();
        let root = dirs._tmp.path();
        let child = Command::new(env!("CARGO_BIN_EXE_rstory"))
            .arg("daemon")
            .env_clear()
            .env("XDG_RUNTIME_DIR", &dirs.runtime)
            .env("XDG_STATE_HOME", root.join("state"))
            .env("XDG_CONFIG_HOME", root.join("config"))
            .env("HOME", root.join("home"))
            .env("DISPLAY", &x.display)
            .env("XDG_SESSION_TYPE", "x11")
            .env("LANG", "C.UTF-8")
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn rstory daemon");
        let this = Self { x, dirs, child };
        let socket = this.dirs.socket();
        crate::support::wait_for_async(|| socket.exists()).await;
        let mut c = this.client().await;
        // Capture only works once the backend is connected.
        for _ in 0..400 {
            if c.status().await["backend"] == "x11" {
                return this;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("the daemon never reported the x11 backend");
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub async fn client(&self) -> Client {
        Client::connect(&self.dirs.socket()).await
    }

    /// Stops the daemon with SIGTERM, as a session would, and waits for it.
    pub fn stop(mut self) {
        let _ = Command::new("kill")
            .args(["-TERM", &self.child.id().to_string()])
            .status();
        let _ = self.child.wait();
    }

    pub fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for DaemonProc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn status_kib(pid: u32, key: &str) -> u64 {
    std::fs::read_to_string(format!("/proc/{pid}/status"))
        .expect("daemon /proc status")
        .lines()
        .find_map(|l| l.strip_prefix(key))
        .and_then(|v| v.trim().trim_end_matches("kB").trim().parse().ok())
        .unwrap_or_else(|| panic!("{key} in /proc/{pid}/status"))
}

/// Current resident set of the daemon process, KiB.
pub fn rss_kib(pid: u32) -> u64 {
    status_kib(pid, "VmRSS:")
}

/// Peak resident set of the daemon process since it started, KiB.
pub fn peak_rss_kib(pid: u32) -> u64 {
    status_kib(pid, "VmHWM:")
}

/// Per-thread count of times the daemon was woken up: voluntary context switches. A thread that
/// sleeps in epoll or on a timer and is woken counts one; a quiet daemon adds none.
pub fn wakeups(pid: u32) -> std::collections::HashMap<u32, u64> {
    let mut by_tid = std::collections::HashMap::new();
    for task in std::fs::read_dir(format!("/proc/{pid}/task")).expect("tasks") {
        let Ok(task) = task else { continue };
        let Some(tid) = task.file_name().to_str().and_then(|t| t.parse().ok()) else {
            continue;
        };
        // A thread may exit between the listing and the read.
        let Ok(text) = std::fs::read_to_string(task.path().join("status")) else {
            continue;
        };
        let n = text
            .lines()
            .find_map(|l| l.strip_prefix("voluntary_ctxt_switches:"))
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(0);
        by_tid.insert(tid, n);
    }
    by_tid
}

/// Wake-ups between two [`wakeups`] readings; a thread that exits in between is not subtracted.
pub fn woke_since(before: &std::collections::HashMap<u32, u64>, pid: u32) -> u64 {
    wakeups(pid)
        .iter()
        .map(|(tid, n)| n.saturating_sub(before.get(tid).copied().unwrap_or(0)))
        .sum()
}

/// A clipboard owner that can copy again at once: one connection and window keep CLIPBOARD,
/// each [`Copier::copy`] swaps the text and asserts ownership again, which makes the X server
/// announce a new selection owner event to the daemon. Independent of `rstory::backend::x11`.
pub struct Copier {
    client: Arc<XClient>,
    text: Arc<Mutex<String>>,
    clipboard: u32,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Copier {
    pub fn start(display: &str) -> Self {
        let client = Arc::new(XClient::connect(display, None).expect("copier connection"));
        let atom = |n: &str| {
            client
                .conn
                .intern_atom(false, n.as_bytes())
                .unwrap()
                .reply()
                .unwrap()
                .atom
        };
        let (clipboard, targets, utf8) = (atom("CLIPBOARD"), atom("TARGETS"), atom("UTF8_STRING"));
        let text = Arc::new(Mutex::new(String::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (c2, t2, s2) = (client.clone(), text.clone(), stop.clone());
        let thread = std::thread::spawn(move || {
            let conn = &c2.conn;
            while !s2.load(Ordering::SeqCst) {
                let Ok(Some(ev)) = conn.poll_for_event() else {
                    std::thread::sleep(Duration::from_micros(300));
                    continue;
                };
                let Event::SelectionRequest(r) = ev else {
                    continue;
                };
                let prop = if r.property == 0 {
                    r.target
                } else {
                    r.property
                };
                let ok = if r.target == targets {
                    let data: Vec<u8> = [targets, utf8]
                        .iter()
                        .flat_map(|a| a.to_ne_bytes())
                        .collect();
                    conn.change_property(
                        PropMode::REPLACE,
                        r.requestor,
                        prop,
                        AtomEnum::ATOM,
                        32,
                        2,
                        &data,
                    )
                    .is_ok()
                } else if r.target == utf8 {
                    let data = t2.lock().unwrap().clone().into_bytes();
                    conn.change_property8(PropMode::REPLACE, r.requestor, prop, utf8, &data)
                        .is_ok()
                } else {
                    false
                };
                let notify = SelectionNotifyEvent {
                    response_type: SELECTION_NOTIFY_EVENT,
                    sequence: 0,
                    time: r.time,
                    requestor: r.requestor,
                    selection: r.selection,
                    target: r.target,
                    property: if ok { prop } else { 0 },
                };
                let _ = conn.send_event(
                    false,
                    r.requestor,
                    x11rb::protocol::xproto::EventMask::NO_EVENT,
                    notify,
                );
                let _ = conn.flush();
            }
        });
        Self {
            client,
            text,
            clipboard,
            stop,
            thread: Some(thread),
        }
    }

    /// One foreign copy of `text`.
    pub fn copy(&self, text: &str) {
        *self.text.lock().unwrap() = text.to_string();
        let c = &self.client;
        c.conn
            .set_selection_owner(c.win, self.clipboard, CURRENT_TIME)
            .unwrap();
        c.conn.flush().unwrap();
    }
}

impl Drop for Copier {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// Waits until `list` shows `text` on top; returns when it did.
pub async fn wait_listed(c: &mut Client, text: &str, within: Duration) -> Instant {
    let end = Instant::now() + within;
    loop {
        let top = c.list().await.first().map(|e| e["preview"].clone());
        // Previews of long entries are cut, so match the start.
        if top
            .as_ref()
            .and_then(|v| v.as_str())
            .is_some_and(|p| text.starts_with(p))
        {
            return Instant::now();
        }
        assert!(Instant::now() < end, "the copy was never listed");
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
}
