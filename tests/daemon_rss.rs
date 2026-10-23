//! SC-002 (T013): memory after 10 000 captures with the default caps. The test process hosts
//! the daemon, so it checks the growth over the 10 000 captures (a leak or an unbounded queue
//! would show) and the absolute figure of the whole process.

mod support;

use support::daemon::{Dirs, Opts, Rig, copy};

fn rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap()
        .lines()
        .find_map(|l| l.strip_prefix("VmRSS:"))
        .and_then(|v| v.trim().trim_end_matches("kB").trim().parse().ok())
        .expect("VmRSS")
}

#[tokio::test]
async fn ten_thousand_captures_stay_within_the_memory_budget() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    let payload = "r".repeat(600);

    // Warm up so allocator and buffers reach steady state before the baseline.
    for round in 0..5 {
        for i in 0..50 {
            copy(&rig.fake, &format!("{payload} warm {round} {i}"));
        }
        c.event("history_changed").await;
    }
    let baseline = rss_kib();
    let mut peak = baseline;
    for round in 0..100 {
        for i in 0..100 {
            copy(&rig.fake, &format!("{payload} {round} {i}"));
        }
        c.event("history_changed").await;
        peak = peak.max(rss_kib());
    }
    // Everything has been processed: the last batch ends with the last copy.
    let st = c.status().await;
    assert_eq!(st["entries"], 20, "the cap held");
    assert_eq!(st["counters"]["dropped_burst"], 0);
    let growth_mib = (peak.saturating_sub(baseline)) / 1024;
    assert!(
        growth_mib <= 8,
        "RSS grew by {growth_mib} MiB over 10 000 captures"
    );
    assert!(
        peak / 1024 <= 64,
        "whole test process (daemon + harness) at {} MiB",
        peak / 1024
    );
    rig.stop().await;
}
