//! Soak (nightly): the real daemon for 30 minutes of idling with one copy a minute. RSS stays
//! flat within 1 MiB and the daemon wakes up only for the copies (and its own timers).
//!
//! `make test-soak` runs the full 30 minutes; `SOAK_SECS=n` shortens the run (local checks). The
//! readings are `/proc/<pid>` of the child, never of the test process.

#[path = "../support/mod.rs"]
mod support;

mod common;

use std::time::{Duration, Instant};

use common::{Copier, DaemonProc, peak_rss_kib, rss_kib, wakeups, woke_since};

/// Wake-ups allowed per minute beyond the copy itself: reading the selection, the store write
/// and the event fan-out take a handful; an idle minute takes none.
const MAX_WAKEUPS_PER_COPY: u64 = 60;
const MAX_WAKEUPS_PER_IDLE_MINUTE: u64 = 1;

#[tokio::test]
async fn thirty_minutes_of_idle_and_a_copy_a_minute_stay_flat_and_quiet() {
    let total = std::env::var("RSTORY_SOAK_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30 * 60);
    // A "minute" is a minute in the nightly run; shortened runs scale it down.
    let minute = Duration::from_secs(total.min(30 * 60) / (total.min(30 * 60) / 60).max(1))
        .min(Duration::from_secs(60));
    let minutes = (total / minute.as_secs().max(1)).max(3) as usize;

    let daemon = DaemonProc::start().await;
    let pid = daemon.pid();
    let copier = Copier::start(&daemon.x.display);
    let mut c = daemon.client().await;
    let mut sub = daemon.client().await;
    sub.subscribe().await;

    let mut rss = Vec::new();
    let mut idle_wakeups = Vec::new();
    let mut copy_wakeups = Vec::new();
    let started = Instant::now();
    for m in 0..minutes {
        let tick = started + minute * m as u32;
        tokio::time::sleep_until(tokio::time::Instant::from_std(tick)).await;
        let w0 = wakeups(pid);
        copier.copy(&format!("soak copy {m:04}"));
        sub.event("history_changed").await;
        common::wait_listed(&mut c, &format!("soak copy {m:04}"), Duration::from_secs(5)).await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let w1 = wakeups(pid);
        copy_wakeups.push(woke_since(&w0, pid));
        rss.push(rss_kib(pid));
        // The rest of the minute is idle (the status poll above is ours, not the daemon's).
        let idle_from = Instant::now();
        let idle_to = tick + minute - Duration::from_secs(1);
        if idle_to > idle_from {
            tokio::time::sleep(idle_to - idle_from).await;
            idle_wakeups.push(woke_since(&w1, pid));
        }
    }
    // The first minutes warm the allocator; the contract is "flat" after that.
    let settled = &rss[(rss.len() / 5).max(1)..];
    let (lo, hi) = (
        *settled.iter().min().unwrap(),
        *settled.iter().max().unwrap(),
    );
    eprintln!(
        "soak {minutes} x {minute:?}: RSS KiB {rss:?}; peak {} KiB; wake-ups per copy {copy_wakeups:?}; idle {idle_wakeups:?}",
        peak_rss_kib(pid)
    );
    assert!(hi - lo <= 1024, "RSS drifted from {lo} to {hi} KiB");
    assert!(
        copy_wakeups.iter().all(|w| *w <= MAX_WAKEUPS_PER_COPY),
        "wake-ups per copy: {copy_wakeups:?}"
    );
    let idle_total: u64 = idle_wakeups.iter().sum();
    assert!(
        idle_total <= MAX_WAKEUPS_PER_IDLE_MINUTE * idle_wakeups.len() as u64,
        "wake-ups while idle: {idle_wakeups:?}"
    );
    drop(copier);
    daemon.stop();
}
