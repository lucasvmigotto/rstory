//! US2 (T006): pin, unpin, rm, clear, pause and resume against a daemon with a fake backend.
//! All clipboard text is synthetic.

mod support;

use rstory::config::Caps;
use serde_json::json;
use support::daemon::{Client, Dirs, Opts, Rig, copy};

async fn with(rig: &Rig, texts: &[&str]) -> Client {
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in texts {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
    c
}

fn previews(list: &[serde_json::Value]) -> Vec<&str> {
    list.iter()
        .map(|e| e["preview"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn pin_and_unpin_by_index_and_by_short_id() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = with(&rig, &["alpha", "beta"]).await;
    let r = dirs.rstory(&["pin", "1"]).await;
    assert_eq!((r.code, r.stdout.len(), r.stderr.as_str()), (0, 0, ""));
    let list = c.list().await;
    assert_eq!(list[1]["pinned"], true, "{list:?}");
    let short = list[1]["short_id"].as_str().unwrap().to_string();
    let r = dirs.rstory(&["unpin", &short]).await;
    assert_eq!((r.code, r.stderr.as_str()), (0, ""));
    assert_eq!(c.list().await[1]["pinned"], false);
    rig.stop().await;
}

#[tokio::test]
async fn pinning_past_the_cap_exits_6() {
    let dirs = Dirs::new();
    let caps = Caps::from_toml("max_pinned = 1").unwrap();
    let rig = Rig::start(
        &dirs,
        Opts {
            caps,
            ..Opts::default()
        },
    )
    .await;
    with(&rig, &["alpha", "beta"]).await;
    assert_eq!(dirs.rstory(&["pin", "0"]).await.code, 0);
    let r = dirs.rstory(&["pin", "1"]).await;
    assert_eq!(r.code, 6, "{}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        "Pin limit reached (1). Unpin an entry first."
    );
    rig.stop().await;
}

#[tokio::test]
async fn rm_removes_one_entry_and_a_second_rm_is_not_found() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = with(&rig, &["alpha", "beta", "gamma"]).await;
    let short = c.list().await[1]["short_id"].as_str().unwrap().to_string();
    let r = dirs.rstory(&["rm", &short]).await;
    assert_eq!((r.code, r.stderr.as_str()), (0, ""));
    assert_eq!(previews(&c.list().await), ["gamma", "alpha"]);
    let again = dirs.rstory(&["rm", &short]).await;
    assert_eq!(again.code, 2);
    assert!(again.stderr.starts_with("No entry"), "{}", again.stderr);
    rig.stop().await;
}

#[tokio::test]
async fn clear_with_yes_keeps_pinned_entries_and_reports_the_count() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = with(&rig, &["alpha", "beta", "gamma"]).await;
    c.ok("pin", json!({"ref": "2"})).await;
    let r = dirs.rstory(&["clear", "--yes"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.is_empty());
    assert_eq!(r.stderr.trim_end(), "Removed 2 entries.");
    assert_eq!(previews(&c.list().await), ["alpha"]);
    rig.stop().await;
}

#[tokio::test]
async fn clear_all_with_yes_removes_pinned_entries_too() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = with(&rig, &["alpha", "beta"]).await;
    c.ok("pin", json!({"ref": "0"})).await;
    let r = dirs.rstory(&["clear", "--all", "--yes"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(r.stderr.trim_end(), "Removed 2 entries.");
    assert!(c.list().await.is_empty());
    rig.stop().await;
}

#[tokio::test]
async fn clear_without_a_terminal_needs_yes() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = with(&rig, &["alpha", "beta"]).await;
    for args in [&["clear"][..], &["clear", "--all"]] {
        let r = dirs.rstory(args).await;
        assert_eq!(r.code, 2, "{args:?}");
        assert_eq!(
            r.stderr.trim_end(),
            "Refusing to clear without a terminal. Add `--yes`."
        );
    }
    assert_eq!(c.list().await.len(), 2, "nothing was deleted");
    rig.stop().await;
}

#[tokio::test]
async fn clearing_an_empty_history_removes_nothing_without_asking() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let r = dirs.rstory(&["clear"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(r.stderr.trim_end(), "Removed 0 entries.");
    rig.stop().await;
}

#[tokio::test]
async fn pause_for_sets_an_end_time_and_resume_ends_it() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    let r = dirs.rstory(&["pause", "--for", "50s"]).await;
    assert_eq!((r.code, r.stdout.len(), r.stderr.as_str()), (0, 0, ""));
    let st = c.status().await;
    assert_eq!(st["state"], "paused");
    // the fake clock starts at 14:00:00 and advances with real time
    let until = st["paused_until"].as_str().unwrap();
    assert!(until.starts_with("2026-10-21T14:00:5"), "{until}");
    let r = dirs.rstory(&["resume"]).await;
    assert_eq!((r.code, r.stderr.as_str()), (0, ""));
    assert_eq!(c.status().await["state"], "recording");
    let r = dirs.rstory(&["pause"]).await;
    assert_eq!(r.code, 0);
    let st = c.status().await;
    assert_eq!(
        (st["state"].as_str(), st["paused_until"].is_null()),
        (Some("paused"), true)
    );
    rig.stop().await;
}

#[tokio::test]
async fn a_bad_duration_is_a_usage_error() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    for bad in ["soon", "0s", "25h", "5"] {
        let r = dirs.rstory(&["pause", "--for", bad]).await;
        assert_eq!(r.code, 2, "{bad}");
    }
    assert_eq!(rig.client().await.status().await["state"], "recording");
    rig.stop().await;
}

#[tokio::test]
async fn write_commands_without_a_daemon_exit_3_with_the_hint() {
    let dirs = Dirs::new();
    for args in [
        &["pause"][..],
        &["pause", "--for", "1m"],
        &["resume"],
        &["pin", "0"],
        &["unpin", "0"],
        &["rm", "0"],
        &["copy", "0"],
        &["clear", "--yes"],
    ] {
        let r = dirs.rstory(args).await;
        assert_eq!(r.code, 3, "{args:?}: {}", r.stderr);
        assert_eq!(
            r.stderr.trim_end(),
            "Daemon not running. Start it with `rstory daemon` or run `rstory install-autostart`."
        );
        assert!(r.stdout.is_empty());
    }
}
