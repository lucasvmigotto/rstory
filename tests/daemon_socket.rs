//! US4 (T011): the real `rstory daemon` process: stale socket, malformed and oversize lines,
//! SIGTERM cleanup, exit codes. The uid check is covered in `ipc_server.rs` (a stranger uid
//! cannot be created without privileges, so the server is told to expect another uid).

use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener as StdListener, UnixStream as StdStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use tempfile::TempDir;

struct Proc {
    child: Child,
    _tmp: TempDir,
    runtime: PathBuf,
    state: PathBuf,
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !ok() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn command(tmp: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_rstory"));
    c.arg("daemon")
        .env_clear()
        .env("XDG_RUNTIME_DIR", tmp.join("run"))
        .env("XDG_STATE_HOME", tmp.join("state"))
        .env("XDG_CONFIG_HOME", tmp.join("config"))
        .env("HOME", tmp)
        .env("PATH", "/nonexistent")
        .env("RUST_LOG", "info")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    c
}

fn start() -> Proc {
    let tmp = tempfile::tempdir().unwrap();
    let runtime = tmp.path().join("run");
    std::fs::create_dir(&runtime).unwrap();
    std::fs::set_permissions(&runtime, std::fs::Permissions::from_mode(0o700)).unwrap();
    start_in(tmp, runtime)
}

fn start_in(tmp: TempDir, runtime: PathBuf) -> Proc {
    let child = command(tmp.path()).spawn().unwrap();
    let state = tmp.path().join("state/rstory");
    Proc {
        child,
        _tmp: tmp,
        runtime,
        state,
    }
}

impl Proc {
    fn socket(&self) -> PathBuf {
        self.runtime.join("rstory/control.sock")
    }

    fn wait_ready(&self) {
        wait_until("the socket", || StdStream::connect(self.socket()).is_ok());
    }

    fn stderr(&mut self) -> String {
        let mut s = String::new();
        if let Some(mut e) = self.child.stderr.take() {
            let _ = e.read_to_string(&mut s);
        }
        s
    }

    fn term(&self) {
        let pid = rustix::process::Pid::from_raw(i32::try_from(self.child.id()).unwrap()).unwrap();
        rustix::process::kill_process(pid, rustix::process::Signal::TERM).unwrap();
    }

    fn exit(&mut self) -> i32 {
        wait_until("exit", || self.child.try_wait().unwrap().is_some());
        self.child.wait().unwrap().code().unwrap_or(-1)
    }
}

impl Drop for Proc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn request(path: &Path, line: &str) -> String {
    use std::io::{BufRead, BufReader, Write};
    let mut s = StdStream::connect(path).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    s.write_all(line.as_bytes()).unwrap();
    s.write_all(b"\n").unwrap();
    let mut out = String::new();
    BufReader::new(s).read_line(&mut out).unwrap();
    out
}

#[test]
fn sigterm_stops_cleanly_removes_the_socket_and_exits_0() {
    let mut p = start();
    p.wait_ready();
    let dir = p.runtime.join("rstory");
    assert_eq!(
        std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let answer = request(&p.socket(), r#"{"v":1,"id":1,"op":"status"}"#);
    assert!(
        answer.contains(r#""state":"inactive""#) && answer.contains("no_session"),
        "{answer}"
    );
    p.term();
    assert_eq!(p.exit(), 0);
    assert!(!p.socket().exists(), "socket removed");
    // The lock is free again.
    drop(
        rstory::store::Store::open_exclusive(
            &p.state,
            rstory::config::Caps::default(),
            rstory::store::fs::RealFs,
        )
        .unwrap(),
    );
}

#[test]
fn a_second_daemon_exits_4_with_the_first_pid() {
    let p = start();
    p.wait_ready();
    let mut second = command(p._tmp.path()).spawn().unwrap();
    wait_until("second exit", || second.try_wait().unwrap().is_some());
    let mut err = String::new();
    second
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut err)
        .unwrap();
    assert_eq!(second.wait().unwrap().code(), Some(4));
    assert!(
        err.contains(&format!("already running (pid {})", p.child.id())),
        "{err}"
    );
}

#[test]
fn a_stale_socket_from_a_crash_is_replaced() {
    let tmp = tempfile::tempdir().unwrap();
    let runtime = tmp.path().join("run");
    std::fs::create_dir(&runtime).unwrap();
    std::fs::create_dir(runtime.join("rstory")).unwrap();
    drop(StdListener::bind(runtime.join("rstory/control.sock")).unwrap()); // file stays
    let p = start_in(tmp, runtime);
    p.wait_ready();
}

#[test]
fn malformed_and_oversize_lines_are_answered_without_a_panic() {
    use std::io::{Read, Write};
    let mut p = start();
    p.wait_ready();
    let bad = request(&p.socket(), "{nope");
    assert!(bad.contains(r#""code":"bad_request""#), "{bad}");

    let mut s = StdStream::connect(p.socket()).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let _ = s.write_all(&vec![b'x'; 2 * 1024 * 1024 + 8192]);
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    assert!(out.contains(r#""code":"too_large""#), "{out}");
    // Still alive afterwards.
    let ok = request(&p.socket(), r#"{"v":1,"id":2,"op":"status"}"#);
    assert!(ok.contains(r#""ok":true"#));
    assert!(p.child.try_wait().unwrap().is_none());
}

#[test]
fn a_store_that_is_too_open_exits_5() {
    let tmp = tempfile::tempdir().unwrap();
    let runtime = tmp.path().join("run");
    std::fs::create_dir(&runtime).unwrap();
    let store = tmp.path().join("state/rstory");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::set_permissions(&store, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut p = start_in(tmp, runtime);
    assert_eq!(p.exit(), 5);
    assert!(p.stderr().contains("too open"));
}

#[test]
fn a_bad_backend_flag_is_a_usage_error() {
    let tmp = tempfile::tempdir().unwrap();
    let out = command(tmp.path())
        .args(["--backend", "gopher"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}
