//! F10: idle quietness of the real daemon. With no copies and no deadline armed it must not wake
//! up at all beyond a small bound, measured from `/proc/<pid>` of the child process (voluntary
//! context switches over all threads). Also records the idle RSS of the process (SC-002).

#[path = "../support/mod.rs"]
mod support;

mod common;

use std::time::Duration;

use common::{DaemonProc, MIB, peak_rss_kib, rss_kib, wakeups, woke_since};

/// Idle window. The fitness function names 60 s.
const IDLE: Duration = Duration::from_secs(60);
/// Wake-ups tolerated over the whole window: a housekeeping tick or two, nothing periodic.
const MAX_WAKEUPS: u64 = 5;

#[tokio::test]
async fn an_idle_daemon_does_not_wake_up_and_stays_small() {
    let daemon = DaemonProc::start().await;
    let pid = daemon.pid();
    // Let startup settle, and make one request so the socket path is warm.
    daemon.client().await.status().await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    let before = wakeups(pid);
    tokio::time::sleep(IDLE).await;
    let woke = woke_since(&before, pid);
    let rss = rss_kib(pid);
    eprintln!(
        "F10 idle {IDLE:?}: {woke} wake-ups; RSS {} KiB, peak {} KiB",
        rss,
        peak_rss_kib(pid)
    );
    assert!(woke <= MAX_WAKEUPS, "F10: {woke} wake-ups in {IDLE:?}");
    assert!(
        peak_rss_kib(pid) * 1024 <= 20 * MIB,
        "idle daemon peak RSS {} KiB is over 20 MiB",
        peak_rss_kib(pid)
    );
    daemon.stop();
}
