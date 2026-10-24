//! Burst and sustained load on the real daemon (specs/004-daemon/qa.md "Load profile", SC-002).
//!
//! The daemon is the release-shaped binary as a child process on a private Xvfb; RSS is read from
//! `/proc/<pid>/status` of the child, not of the test process. Open model: the copier fires on a
//! fixed schedule whether or not the daemon keeps up, as a real burst would.

#[path = "../support/mod.rs"]
mod support;

mod common;

use std::time::{Duration, Instant};

use common::{Copier, DaemonProc, MIB, budget, peak_rss_kib, percentile};

const BURST: usize = 500;
const LIST_CLIENTS: usize = 5;

/// 500 copies in 1 s while 5 clients list at 10 requests a second.
#[tokio::test]
async fn burst_of_500_copies_in_a_second_keeps_the_daemon_responsive() {
    let daemon = DaemonProc::start().await;
    let pid = daemon.pid();
    let copier = Copier::start(&daemon.x.display);
    let mut sub = daemon.client().await;
    sub.subscribe().await;

    let start = Instant::now();
    let end = start + Duration::from_secs(1);
    let mut clients = Vec::new();
    for _ in 0..LIST_CLIENTS {
        let mut c = daemon.client().await;
        clients.push(tokio::spawn(async move {
            let mut lat = Vec::new();
            let mut tick = tokio::time::interval(Duration::from_millis(100));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            while Instant::now() < end {
                tick.tick().await;
                let t = Instant::now();
                c.list().await;
                lat.push(t.elapsed());
            }
            lat
        }));
    }
    // The copier runs on its own thread against the schedule: copy i at i * 2 ms.
    let pace = std::thread::spawn(move || {
        for i in 0..BURST {
            let due = start + Duration::from_micros(2000 * i as u64);
            while Instant::now() < due {
                std::thread::sleep(Duration::from_micros(200));
            }
            copier.copy(&format!("burst copy {i:04}"));
        }
        copier
    });
    let mut lat = Vec::new();
    for c in clients {
        lat.extend(c.await.unwrap());
    }
    let copier = pace.join().unwrap();
    assert!(lat.len() >= LIST_CLIENTS * 8, "{} list samples", lat.len());

    // Let the daemon drain, then read the verdicts.
    sub.event("history_changed").await;
    let mut c = daemon.client().await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let st = c.status().await;
    let p95 = percentile(&mut lat, 95);
    let peak = peak_rss_kib(pid);
    let dropped = st["counters"]["dropped_burst"]
        .as_u64()
        .expect("dropped_burst is reported");
    eprintln!(
        "burst: list p95 {p95:?} over {} requests; dropped_burst {dropped}; peak RSS {peak} KiB; entries {}",
        lat.len(),
        st["entries"]
    );
    assert!(p95 < budget(50, 100), "list p95 {p95:?} during the burst");
    // The queue holds at most 256 changes, so at most 500 - 256 can ever be dropped by a daemon
    // that never reads; anything more would mean events vanished somewhere else.
    assert!(dropped <= (BURST - 256) as u64, "dropped_burst {dropped}");
    assert_eq!(st["state"], "recording");
    assert!(st["entries"].as_u64().unwrap() <= 20, "the cap held");
    assert!(
        peak * 1024 <= 20 * MIB,
        "daemon peak RSS {peak} KiB is over 20 MiB"
    );
    // No crash: the process is the same one and still answers.
    let mut daemon = daemon;
    assert!(daemon.alive());
    copier.copy("after the burst");
    common::wait_listed(&mut c, "after the burst", Duration::from_secs(5)).await;
    daemon.stop();
}

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
