//! 007 T009: the terminal is restored on a panic and on SIGTERM/SIGHUP/SIGINT, with a fake
//! screen: no real terminal is touched. Its own test binary, since the panic hook and the
//! signal handlers are process-wide.

use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use rstory::picker::term::{Restorer, Screen, install_panic_hook, watch_signals};

/// Serializes the tests: they share the process's panic hook and signal dispositions.
static SERIAL: Mutex<()> = Mutex::new(());

struct Fake(Arc<AtomicUsize>);

impl Screen for Fake {
    fn enter(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn leave(&mut self) -> io::Result<()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn entered() -> (Restorer, Arc<AtomicUsize>) {
    let leaves = Arc::new(AtomicUsize::new(0));
    let r = Restorer::enter(Box::new(Fake(leaves.clone()))).unwrap();
    (r, leaves)
}

#[test]
fn a_panic_restores_the_terminal_once() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (r, leaves) = entered();
    install_panic_hook(r.clone());
    let result = std::panic::catch_unwind(|| panic!("synthetic panic"));
    assert!(result.is_err());
    assert!(!r.active(), "raw mode left by the panic hook");
    assert_eq!(leaves.load(Ordering::SeqCst), 1);
    // The guard's own restore afterwards is a no-op.
    assert!(!r.restore());
    assert_eq!(leaves.load(Ordering::SeqCst), 1);
}

#[test]
fn a_signal_restores_the_terminal_then_exits_with_128_plus_n() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    use rustix::process::{Signal, getpid, kill_process};
    for (sig, code) in [(Signal::HUP, 129), (Signal::TERM, 143), (Signal::INT, 130)] {
        let (r, leaves) = entered();
        let (tx, rx) = mpsc::channel();
        watch_signals(r.clone(), move |c| tx.send(c).unwrap()).unwrap();
        kill_process(getpid(), sig).unwrap();
        let got = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("no signal seen");
        assert_eq!(got, code, "{sig:?}");
        assert!(!r.active(), "{sig:?}");
        assert_eq!(leaves.load(Ordering::SeqCst), 1, "{sig:?}");
    }
}
