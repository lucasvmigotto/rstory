//! The daemon on a failing store (FaultFs): pause fails closed (charter 005, P2) and failed
//! commits log the I/O error kind, never content (charter 001, S1). Synthetic text only.

#[path = "support/daemon.rs"]
mod rig;

use std::io;
use std::sync::{Arc, Mutex};

use rig::{BASE, Client, Dirs, change_only, copy, no_terminal};
use rstory::backend::fake::{FakeBackend, FakeHandle};
use rstory::backend::{BackendKind, Mode};
use rstory::config::Caps;
use rstory::daemon::state::{BackendInfo, Clock};
use rstory::daemon::{Daemon, run_with_listener};
use rstory::ipc::server;
use rstory::store::Store;
use rstory::store::fs::{Fault, FaultFs};
use serde_json::json;
use tokio::sync::watch;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Default)]
struct Buf(Arc<Mutex<Vec<u8>>>);

impl io::Write for Buf {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Buf {
    type Writer = Buf;
    fn make_writer(&'a self) -> Buf {
        self.clone()
    }
}

/// Mutating calls the store spends on opening.
fn open_cost() -> u64 {
    let d = Dirs::new();
    let fs = FaultFs::recording();
    drop(Store::open_exclusive_at(&d.store, Caps::default(), fs.clone(), BASE).unwrap());
    fs.mutations()
}

/// A daemon whose store fails once, `after` mutating calls past the open, with `kind`.
async fn start(
    dirs: &Dirs,
    after: u64,
    kind: io::ErrorKind,
) -> (FakeHandle, Client, watch::Sender<bool>) {
    let fs = FaultFs::with_kind(open_cost() + after, Fault::Fail, kind);
    let store = Store::open_exclusive_at(&dirs.store, Caps::default(), fs, BASE).unwrap();
    let (backend, fake) = FakeBackend::new();
    let daemon = Daemon::new(
        store,
        Some(backend),
        BackendInfo::active(BackendKind::X11, Mode::Native),
        Caps::default(),
        Clock::at(BASE),
        no_terminal(),
    )
    .unwrap();
    let listener = server::bind(&dirs.runtime).unwrap();
    let (stop, rx) = watch::channel(false);
    tokio::spawn(run_with_listener(daemon, listener, rx));
    let client = Client::connect(&dirs.socket()).await;
    (fake, client, stop)
}

#[tokio::test]
async fn pause_closes_the_gate_even_when_it_cannot_be_persisted() {
    let dirs = Dirs::new();
    let (fake, mut c, stop) = start(&dirs, 0, io::ErrorKind::StorageFull).await;
    let r = c.call("pause", json!({})).await;
    assert_eq!(r["ok"], false, "the failure is reported: {r}");
    assert_eq!(c.status().await["state"], "paused");
    change_only(&fake);
    loop {
        if c.status().await["counters"]["skipped_paused"] == 1 {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert!(c.list().await.is_empty(), "no copy is recorded after pause");
    stop.send(true).unwrap();
}

#[tokio::test]
async fn a_failed_capture_commit_logs_the_io_error_kind() {
    let buf = Buf::default();
    let sub = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(buf.clone())
        .with_ansi(false)
        .finish();
    let _guard = tracing::subscriber::set_default(sub);

    let dirs = Dirs::new();
    let (fake, mut c, stop) = start(&dirs, 0, io::ErrorKind::StorageFull).await;
    copy(&fake, "synthetic-lost");
    loop {
        let log = String::from_utf8_lossy(&buf.0.lock().unwrap()).into_owned();
        if log.contains("capture commit failed") {
            assert!(log.contains("StorageFull"), "{log}");
            assert!(!log.contains("synthetic-lost"));
            break;
        }
        tokio::task::yield_now().await;
    }
    assert!(c.list().await.is_empty());
    stop.send(true).unwrap();
}

#[tokio::test]
async fn a_failed_command_logs_the_io_error_kind() {
    let buf = Buf::default();
    let sub = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(buf.clone())
        .with_ansi(false)
        .finish();
    let _guard = tracing::subscriber::set_default(sub);

    let dirs = Dirs::new();
    let (_fake, mut c, stop) = start(&dirs, 0, io::ErrorKind::StorageFull).await;
    assert_eq!(c.err("clear", json!({"all": true})).await, "internal");
    let log = String::from_utf8_lossy(&buf.0.lock().unwrap()).into_owned();
    assert!(
        log.contains("store error") && log.contains("StorageFull"),
        "{log}"
    );
    stop.send(true).unwrap();
}
