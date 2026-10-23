//! US4 (T010): status, the single deadline timer, expiry, reconnect, open_picker.

mod support;

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use jsonschema::Validator;
use rstory::backend::BackendEvent;
use rstory::backend::fake::{FakeBackend, FakeHandle};
use rstory::config::Caps;
use rstory::daemon::{Connector, LaunchEnv};
use serde_json::{Value, json};
use support::daemon::{Dirs, Opts, Rig, copy};

fn status_validator() -> Validator {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("contracts/protocol.schema.json");
    let s: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let wrapped = json!({"$schema": s["$schema"], "$defs": s["$defs"], "$ref": "#/$defs/Status"});
    jsonschema::validator_for(&wrapped).unwrap()
}

#[tokio::test]
async fn status_has_every_field_and_validates_against_the_schema() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    let st = c.status().await;
    assert!(status_validator().is_valid(&st), "{st}");
    assert_eq!(st["state"], "recording");
    assert_eq!(st["backend"], "x11");
    assert_eq!(st["mode"], "native");
    assert_eq!(st["entries"], 0);
    assert_eq!(st["pinned"], 0);
    assert_eq!(st["max_entries"], 20);
    assert_eq!(st["max_pinned"], 10);
    assert_eq!(st["protocol"], 1);
    assert_eq!(st["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(st["picker_available"], false);
    for k in [
        "skipped_sensitive",
        "skipped_oversize",
        "skipped_paused",
        "dropped_burst",
    ] {
        assert_eq!(st["counters"][k], 0, "{k}");
    }
    rig.stop().await;
}

#[tokio::test]
async fn status_counts_entries_and_pins() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in ["a1", "b2"] {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
    c.ok("pin", json!({"ref": "0"})).await;
    let st = c.status().await;
    assert_eq!(
        (st["entries"].as_u64(), st["pinned"].as_u64()),
        (Some(2), Some(1))
    );
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn entries_expire_exactly_at_their_instant_through_one_timer_without_idle_wakeups() {
    let dirs = Dirs::new();
    let caps = Caps {
        ttl: Duration::from_secs(3 * 3600),
        ..Caps::default()
    };
    let rig = Rig::start(
        &dirs,
        Opts {
            caps,
            ..Opts::default()
        },
    )
    .await;
    let mut c = rig.client().await;
    c.subscribe().await;
    copy(&rig.fake, "short lived");
    c.event("history_changed").await;
    let at_rest = rig.probe.wakeups.load(Ordering::Relaxed);

    // Idle for just under 3 hours: only the armed timer exists, so nothing wakes the loop.
    tokio::time::advance(Duration::from_secs(3 * 3600 - 1)).await;
    tokio::task::yield_now().await;
    assert_eq!(
        rig.probe.wakeups.load(Ordering::Relaxed),
        at_rest,
        "no wake-up before the deadline"
    );

    tokio::time::advance(Duration::from_secs(1)).await;
    let ev = c.event("history_changed").await;
    assert_eq!(ev["entries"], 0);
    assert_eq!(ev["top"], json!(null));
    assert_eq!(
        rig.probe.wakeups.load(Ordering::Relaxed),
        at_rest + 1,
        "exactly one wake-up for the expiry"
    );
    assert!(c.list().await.is_empty());
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn an_idle_daemon_with_no_deadlines_never_wakes() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.status().await;
    let before = rig.probe.wakeups.load(Ordering::Relaxed);
    tokio::time::advance(Duration::from_secs(60)).await;
    tokio::task::yield_now().await;
    assert_eq!(
        rig.probe.wakeups.load(Ordering::Relaxed),
        before,
        "F10: no wake-ups in 60 s"
    );
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn a_lost_display_makes_the_daemon_inactive_and_it_reconnects_with_backoff() {
    let dirs = Dirs::new();
    let spare: Arc<Mutex<Vec<(FakeBackend, FakeHandle)>>> = Arc::new(Mutex::new(Vec::new()));
    let (b2, h2) = FakeBackend::new();
    spare.lock().unwrap().push((b2, h2.clone()));
    let attempts = Arc::new(Mutex::new(0u32));
    let (spare2, attempts2) = (Arc::clone(&spare), Arc::clone(&attempts));
    let connector: Connector<FakeBackend> = Box::new(move || {
        let (spare, attempts) = (Arc::clone(&spare2), Arc::clone(&attempts2));
        Box::pin(async move {
            let mut n = attempts.lock().unwrap();
            *n += 1;
            if *n < 3 {
                return None; // the display is not back yet
            }
            spare.lock().unwrap().pop().map(|(b, _)| b)
        })
    });
    let rig = Rig::start(
        &dirs,
        Opts {
            connector: Some(connector),
            ..Opts::default()
        },
    )
    .await;
    let mut c = rig.client().await;
    c.subscribe().await;

    rig.fake.emit(BackendEvent::Disconnected);
    assert_eq!(c.event("pause_changed").await["state"], "inactive");
    let st = c.status().await;
    assert_eq!(st["state"], "inactive");
    assert_eq!(st["reason"], "disconnected");

    // 100 ms (fail), then 200 ms (fail), then 400 ms (success).
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;
    assert_eq!(c.status().await["state"], "inactive");
    tokio::time::advance(Duration::from_millis(200)).await;
    tokio::task::yield_now().await;
    assert_eq!(c.status().await["state"], "inactive");
    tokio::time::advance(Duration::from_millis(400)).await;
    assert_eq!(c.event("pause_changed").await["state"], "recording");
    assert_eq!(*attempts.lock().unwrap(), 3);

    copy(&h2, "after reconnect");
    c.event("history_changed").await;
    assert_eq!(c.list().await[0]["preview"], "after reconnect");
    rig.stop().await;
}

fn stub_launch(picker_command: Option<&str>, terminals: &'static [&'static str]) -> LaunchEnv {
    LaunchEnv {
        picker_command: picker_command.map(str::to_string),
        terminal: None,
        on_path: Arc::new(move |n| terminals.contains(&n)),
    }
}

#[tokio::test]
async fn open_picker_spawns_the_configured_command_and_status_reports_it_available() {
    let dirs = Dirs::new();
    let launch = stub_launch(Some("/bin/true"), &[]);
    let rig = Rig::start(
        &dirs,
        Opts {
            launch,
            ..Opts::default()
        },
    )
    .await;
    let mut c = rig.client().await;
    assert_eq!(c.status().await["picker_available"], true);
    let r = c.ok("open_picker", json!({})).await;
    assert_eq!(r["argv"], json!(["/bin/true"]));
    rig.stop().await;
}

#[tokio::test]
async fn open_picker_without_a_terminal_is_no_terminal_and_status_hides_the_picker() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    assert_eq!(c.status().await["picker_available"], false);
    assert_eq!(c.err("open_picker", json!({})).await, "no_terminal");
    rig.stop().await;
}

#[tokio::test]
async fn a_terminal_from_the_list_makes_the_picker_available() {
    // Resolution only: no `open_picker` here, so no test can start a terminal installed on the
    // machine running it (the spawn path is covered with `/bin/true` and a missing command).
    let dirs = Dirs::new();
    let launch = LaunchEnv {
        on_path: Arc::new(|n| n == "kitty"),
        ..stub_launch(None, &[])
    };
    let rig = Rig::start(
        &dirs,
        Opts {
            launch,
            ..Opts::default()
        },
    )
    .await;
    let mut c = rig.client().await;
    assert_eq!(c.status().await["picker_available"], true);
    rig.stop().await;
}

#[tokio::test]
async fn a_picker_command_that_cannot_start_is_picker_command_failed() {
    let dirs = Dirs::new();
    let launch = stub_launch(Some("/nonexistent/picker --flag secret-arg"), &[]);
    let rig = Rig::start(
        &dirs,
        Opts {
            launch,
            ..Opts::default()
        },
    )
    .await;
    let mut c = rig.client().await;
    let r = c.call("open_picker", json!({})).await;
    assert_eq!(r["error"]["code"], "picker_command_failed");
    let msg = r["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("/nonexistent/picker") && !msg.contains("secret-arg"),
        "{msg}"
    );
    rig.stop().await;
}

#[tokio::test]
async fn list_pin_unpin_rm_clear_follow_the_contract() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in ["a1", "b2", "c3"] {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
    assert_eq!(
        c.ok("list", json!({"limit": 2})).await["entries"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let pinned = c.ok("pin", json!({"ref": "2"})).await;
    assert_eq!(pinned["entry"]["pinned"], true);
    c.event("history_changed").await;
    c.ok("pin", json!({"ref": "2"})).await; // idempotent: no second event is required
    let un = c.ok("unpin", json!({"ref": "2"})).await;
    assert_eq!(un["entry"]["pinned"], false);
    c.event("history_changed").await;
    c.ok("pin", json!({"ref": "2"})).await;
    c.event("history_changed").await;
    c.ok("rm", json!({"ref": "0"})).await;
    c.event("history_changed").await;
    assert_eq!(c.err("rm", json!({"ref": "9"})).await, "not_found");
    let cleared = c.ok("clear", json!({})).await;
    assert_eq!(cleared["removed"], 1, "the pinned entry stays");
    c.event("history_changed").await;
    let all = c.ok("clear", json!({"all": true})).await;
    assert_eq!(all["removed"], 1);
    assert!(c.list().await.is_empty());
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn the_pin_cap_gives_pin_limit_and_a_shared_prefix_gives_ambiguous_with_candidates() {
    let dirs = Dirs::new();
    let caps = Caps {
        max_entries: 20,
        max_pinned: 2,
        ..Caps::default()
    };
    let rig = Rig::start(
        &dirs,
        Opts {
            caps,
            ..Opts::default()
        },
    )
    .await;
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in ["a1", "b2", "c3"] {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
    c.ok("pin", json!({"ref": "0"})).await;
    c.ok("pin", json!({"ref": "1"})).await;
    assert_eq!(c.err("pin", json!({"ref": "2"})).await, "pin_limit");

    // The clock is frozen, so all three ids share their timestamp prefix.
    let entries = c.list().await;
    let prefix = entries[0]["id"].as_str().unwrap()[..4].to_string();
    let r = c.call("show", json!({"ref": prefix})).await;
    assert_eq!(r["error"]["code"], "ambiguous");
    let msg = r["error"]["message"].as_str().unwrap();
    for e in &entries {
        assert!(msg.contains(e["short_id"].as_str().unwrap()), "{msg}");
    }
    rig.stop().await;
}
