//! SC-002: the daemon process's own resident memory after 10 000 captures with the default caps,
//! read from `/proc/<pid>/status` of the real binary running as a child on Xvfb (not the test
//! process). Its own binary, so it runs after the latency tests: 10 000 fsynced captures leave
//! the disk busy for a while, which would inflate F7.

#[path = "../support/mod.rs"]
mod support;

mod common;

use std::time::Duration;

use common::{Copier, DaemonProc, MIB, peak_rss_kib};

/// SC-002: 10 000 captures of 600-byte entries with the default caps, daemon RSS absolute.
#[tokio::test]
async fn ten_thousand_captures_keep_the_daemon_process_under_20_mib() {
    let daemon = DaemonProc::start().await;
    let pid = daemon.pid();
    let copier = Copier::start(&daemon.x.display);
    let mut c = daemon.client().await;
    let payload = "r".repeat(600);
    for round in 0..100 {
        for i in 0..100 {
            copier.copy(&format!("{round:02}-{i:02} {payload}"));
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        // Let the daemon catch up between rounds: this is sustained load, not a burst.
        c.status().await;
    }
    let last = format!("99-99 {payload}");
    common::wait_listed(&mut c, &last, Duration::from_secs(10)).await;
    let st = c.status().await;
    let peak = peak_rss_kib(pid);
    eprintln!("sustained: peak RSS {peak} KiB; entries {}", st["entries"]);
    assert_eq!(st["entries"], 20, "the cap held");
    assert!(
        peak * 1024 <= 20 * MIB,
        "SC-002: daemon peak RSS {peak} KiB is over 20 MiB"
    );
    drop(copier);
    daemon.stop();
}
