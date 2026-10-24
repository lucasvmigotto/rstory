//! F7: capture to listed. From the moment a foreign copy happens on X11 until `list` on the
//! control socket shows it, p95 under 100 ms locally and 200 ms on a CI runner (test-strategy
//! section 6). The daemon is the real binary on a private Xvfb; every capture is fsynced, so the
//! figure includes the disk. All text is synthetic.

#[path = "../support/mod.rs"]
mod support;

mod common;

use std::time::{Duration, Instant};

use common::{Copier, DaemonProc, budget, percentile, wait_listed};

const SAMPLES: usize = 200;

#[tokio::test]
async fn capture_to_listed_p95_is_within_the_budget() {
    let daemon = DaemonProc::start().await;
    let copier = Copier::start(&daemon.x.display);
    let mut sub = daemon.client().await;
    sub.subscribe().await;
    let mut c = daemon.client().await;

    let mut ms = Vec::with_capacity(SAMPLES);
    for i in 0..SAMPLES + 5 {
        let text = format!("latency sample {i}");
        let t0 = Instant::now();
        copier.copy(&text);
        sub.event("history_changed").await;
        let top = c.list().await.first().map(|e| e["preview"].clone());
        let t1 = if top.as_ref().and_then(|v| v.as_str()) == Some(text.as_str()) {
            Instant::now()
        } else {
            wait_listed(&mut c, &text, Duration::from_secs(5)).await
        };
        // The first few copies warm the connection and the page cache.
        if i >= 5 {
            ms.push(t1 - t0);
        }
        // Idle gap, so each sample is a lone capture and not a queue behind the last one.
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let p50 = percentile(&mut ms, 50);
    let p95 = percentile(&mut ms, 95);
    let max = *ms.last().unwrap();
    let limit = budget(100, 200);
    eprintln!("F7 capture to listed: p50 {p50:?}, p95 {p95:?}, max {max:?} (budget {limit:?})");
    assert!(p95 < limit, "F7: p95 {p95:?} is over the {limit:?} budget");
    drop(copier);
    daemon.stop();
}
