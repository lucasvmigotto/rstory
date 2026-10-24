//! E2E-PEER Intruder is rejected (specs/004 qa.md): a connection from another uid is closed
//! with no bytes sent back, and the daemon logs `peer_rejected{uid}`.
//!
//! A second uid needs privilege (`setpriv --reuid`), which the unprivileged tools container does
//! not have: a user namespace cannot help, because the intruder and the owner are the same kernel
//! uid and the daemon sees both as the same mapped uid. So there are two forms:
//! - with privilege to switch uid (root, CAP_SETUID): the real `rstory daemon` as the owner and a
//!   `socat` client as uid 65534 on the socket, permissions loosened for the test;
//! - without: the daemon's own accept loop (`ipc::server::serve`, the code `rstory daemon` runs)
//!   in this process, told to expect another uid, so every peer is a stranger.

use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use rstory::ipc::server::{self, Link};
use tokio::sync::watch;

use crate::session::wait_for;
use crate::support::{SECS5, SECS15, World};

const STATUS: &[u8] = b"{\"v\":1,\"id\":1,\"op\":\"status\",\"args\":{}}\n";

fn can_switch_uid() -> bool {
    Command::new("setpriv")
        .args([
            "--reuid=65534",
            "--regid=65534",
            "--clear-groups",
            "--",
            "true",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[test]
fn peer_connection_from_another_uid_is_closed_without_a_byte_and_logged() {
    if can_switch_uid() {
        real_daemon_real_intruder();
    } else {
        eprintln!("no privilege to switch uid: checking the daemon's accept loop in-process");
        accept_loop_with_a_stranger();
    }
}

fn real_daemon_real_intruder() {
    let w = World::xvfb();
    let mut daemon = w.start_daemon();
    // Loosen the permissions so only the uid check stands between the intruder and the daemon.
    let run = w.runtime_dir();
    std::fs::set_permissions(&run, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::set_permissions(w.socket(), std::fs::Permissions::from_mode(0o666)).unwrap();

    // Control: the owner is served.
    assert_eq!(w.status()["state"], "recording");

    let mut intruder = Command::new("setpriv")
        .args(["--reuid=65534", "--regid=65534", "--clear-groups", "--"])
        .args([
            "socat",
            "-",
            &format!("UNIX-CONNECT:{}", w.socket().display()),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    intruder.stdin.take().unwrap().write_all(STATUS).unwrap();
    let mut got = Vec::new();
    intruder
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut got)
        .unwrap();
    let _ = intruder.wait();
    assert!(got.is_empty(), "{} bytes came back", got.len());

    let logged = wait_for(SECS15, || {
        let t = daemon.log_text();
        t.contains("peer_rejected").then_some(t)
    })
    .expect("the daemon logs peer_rejected");
    assert!(logged.contains("uid=65534"), "{logged}");
    daemon.stop();
}

/// A `MakeWriter` into a shared buffer, to read the log back.
#[derive(Clone, Default)]
struct Sink(Arc<Mutex<Vec<u8>>>);

impl Write for Sink {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn accept_loop_with_a_stranger() {
    let sink = Sink::default();
    let sub = tracing_subscriber::fmt()
        .with_writer({
            let s = sink.clone();
            move || s.clone()
        })
        .with_ansi(false)
        .finish();
    let _guard = tracing::subscriber::set_default(sub);

    let me = rustix::process::getuid().as_raw();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let listener = server::bind(dir.path()).unwrap();
        let path = server::socket_path(dir.path());
        let (link, _calls) = Link::new();
        let (stop, rx) = watch::channel(false);
        // Every peer, including us, is not uid `me + 1`.
        let serving = tokio::spawn(server::serve(listener, link, me + 1, rx));

        let got = tokio::task::spawn_blocking(move || {
            let mut s = UnixStream::connect(&path).expect("the socket accepts the connection");
            s.set_read_timeout(Some(SECS5)).unwrap();
            let _ = s.write_all(STATUS);
            let mut got = Vec::new();
            let _ = s.read_to_end(&mut got);
            got
        })
        .await
        .unwrap();
        assert!(got.is_empty(), "{} bytes came back", got.len());
        let _ = stop.send(true);
        let _ = serving.await;
    });
    let log = String::from_utf8(sink.0.lock().unwrap().clone()).unwrap();
    assert!(log.contains("peer_rejected"), "{log}");
    assert!(log.contains(&format!("uid={me}")), "{log}");
}
