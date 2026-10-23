//! US3 (T008): pause, timed pause and their persistence, on tokio's paused clock.

mod support;

use std::time::Duration;

use serde_json::json;
use support::daemon::{BASE, Client, Dirs, Opts, Rig, change_only, copy};
use time::format_description::well_known::Rfc3339;

async fn sub(rig: &Rig) -> Client {
    let mut c = rig.client().await;
    c.subscribe().await;
    c
}

#[tokio::test(start_paused = true)]
async fn an_indefinite_pause_skips_and_counts_until_resume() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = sub(&rig).await;
    let r = c.ok("pause", json!({})).await;
    assert_eq!(r["state"]["state"], "paused");
    assert_eq!(r["state"]["paused_until"], json!(null));
    let ev = c.event("pause_changed").await;
    assert_eq!(ev["state"], "paused");

    change_only(&rig.fake);
    change_only(&rig.fake);
    let st = loop {
        let st = c.status().await;
        if st["counters"]["skipped_paused"] == 2 {
            break st;
        }
        tokio::task::yield_now().await;
    };
    assert_eq!(st["state"], "paused");
    assert!(c.list().await.is_empty());

    let r = c.ok("resume", json!({})).await;
    assert_eq!(r["state"]["state"], "recording");
    assert_eq!(c.event("pause_changed").await["state"], "recording");
    copy(&rig.fake, "visible");
    c.event("history_changed").await;
    assert_eq!(c.list().await.len(), 1);
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn a_timed_pause_ends_by_itself_at_its_instant() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = sub(&rig).await;
    let r = c.ok("pause", json!({"for": "50s"})).await;
    let until = r["state"]["paused_until"].as_str().unwrap().to_string();
    assert_eq!(
        until,
        (BASE + Duration::from_secs(50))
            .format(&Rfc3339)
            .unwrap()
            .replace("+00:00", "Z")
    );
    c.event("pause_changed").await;

    change_only(&rig.fake);
    tokio::time::advance(Duration::from_secs(49)).await;
    assert_eq!(c.status().await["state"], "paused", "one second early");

    tokio::time::advance(Duration::from_secs(1)).await;
    let ev = c.event("pause_changed").await;
    assert_eq!(ev["state"], "recording", "no client action was needed");
    copy(&rig.fake, "after");
    c.event("history_changed").await;
    let entries = c.list().await;
    assert_eq!(entries.len(), 1, "the copy during the pause was skipped");
    assert_eq!(entries[0]["preview"], "after");
    assert_eq!(c.status().await["counters"]["skipped_paused"], 1);
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn resume_ends_a_timed_pause_at_once() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = sub(&rig).await;
    c.ok("pause", json!({"for": "1h"})).await;
    let r = c.ok("resume", json!({})).await;
    assert_eq!(r["state"]["state"], "recording");
    assert_eq!(r["state"]["paused_until"], json!(null));
    copy(&rig.fake, "now");
    c.event("history_changed").await;
    // Resume while recording is a no-op success.
    c.ok("resume", json!({})).await;
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn pause_durations_outside_one_second_to_a_day_are_bad_requests() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    for bad in ["0s", "25h", "1441m", "soon", "5d"] {
        assert_eq!(
            c.err("pause", json!({"for": bad})).await,
            "bad_request",
            "{bad}"
        );
    }
    assert_eq!(c.status().await["state"], "recording");
    c.ok("pause", json!({"for": "1s"})).await;
    c.ok("pause", json!({"for": "24h"})).await;
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn an_indefinite_pause_survives_a_restart() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    rig.client().await.ok("pause", json!({})).await;
    rig.stop().await;

    let rig = Rig::start(
        &dirs,
        Opts {
            clock_base: BASE + Duration::from_secs(3600),
            ..Opts::default()
        },
    )
    .await;
    let mut c = rig.client().await;
    let st = c.status().await;
    assert_eq!(st["state"], "paused");
    change_only(&rig.fake);
    c.ok("resume", json!({})).await;
    assert!(c.list().await.is_empty());
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn a_timed_pause_keeps_its_original_end_across_a_restart() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let first = rig.client().await.ok("pause", json!({"for": "10m"})).await;
    rig.stop().await;

    let rig = Rig::start(
        &dirs,
        Opts {
            clock_base: BASE + Duration::from_secs(60),
            ..Opts::default()
        },
    )
    .await;
    let mut c = sub(&rig).await;
    let st = c.status().await;
    assert_eq!(st["state"], "paused");
    assert_eq!(
        st["paused_until"], first["state"]["paused_until"],
        "not extended"
    );
    // 9 more minutes: the original end is 10 minutes after BASE, i.e. 9 after the restart.
    tokio::time::advance(Duration::from_secs(9 * 60)).await;
    assert_eq!(c.event("pause_changed").await["state"], "recording");
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn a_pause_that_ended_while_the_daemon_was_down_resumes_on_start() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    rig.client().await.ok("pause", json!({"for": "50s"})).await;
    rig.stop().await;

    let rig = Rig::start(
        &dirs,
        Opts {
            clock_base: BASE + Duration::from_secs(120),
            ..Opts::default()
        },
    )
    .await;
    let mut c = rig.client().await;
    assert_eq!(c.status().await["state"], "recording");
    copy(&rig.fake, "recorded");
    let mut s = rig.client().await;
    s.subscribe().await;
    drop(s);
    loop {
        if !c.list().await.is_empty() {
            break;
        }
        tokio::task::yield_now().await;
    }
    rig.stop().await;
    // And it was written back: a third start sees recording, not a stale pause.
    let rig = Rig::start(
        &dirs,
        Opts {
            clock_base: BASE + Duration::from_secs(130),
            ..Opts::default()
        },
    )
    .await;
    assert_eq!(rig.client().await.status().await["state"], "recording");
    rig.stop().await;
}
