//! US1 (T004): capture into the store, single instance, inactive state, latency, burst.

mod support;

use std::time::{Duration, Instant};

use rstory::backend::fake::FakeBackend;
use rstory::config::Caps;
use rstory::daemon::LaunchEnv;
use rstory::daemon::start::{LOCK_RETRY, Provided, Runtime, StartError, open_store, serve_process};
use rstory::daemon::state::{BackendInfo, Clock};
use rstory::store::readonly::ReadOnlyStore;
use serde_json::json;
use support::daemon::{Dirs, Opts, Rig, copy};
use tokio::sync::watch;

#[tokio::test]
async fn three_changes_make_three_entries_and_three_events() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut sub = rig.client().await;
    sub.subscribe().await;
    for text in ["one", "two", "three"] {
        copy(&rig.fake, text);
        let ev = sub.event("history_changed").await;
        assert!(ev["entries"].as_u64().unwrap() >= 1);
    }
    let mut c = rig.client().await;
    let entries = c.list().await;
    assert_eq!(entries.len(), 3);
    let previews: Vec<_> = entries
        .iter()
        .map(|e| e["preview"].as_str().unwrap())
        .collect();
    assert_eq!(previews, ["three", "two", "one"], "newest first");
    let shown = c.ok("show", json!({"ref": "0"})).await;
    assert_eq!(shown["content"], "three");
    // The event named the new top entry and nothing else.
    copy(&rig.fake, "four");
    let ev = sub.event("history_changed").await;
    assert_eq!(ev["entries"], 4);
    assert_eq!(ev["top"], c.list().await[0]["id"]);
    rig.stop().await;
}

#[tokio::test]
async fn a_repeated_copy_refreshes_instead_of_duplicating() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    copy(&rig.fake, "same");
    c.event("history_changed").await;
    copy(&rig.fake, "other");
    c.event("history_changed").await;
    copy(&rig.fake, "same");
    c.event("history_changed").await;
    let previews: Vec<_> = c
        .list()
        .await
        .iter()
        .map(|e| e["preview"].clone())
        .collect();
    assert_eq!(previews, [json!("same"), json!("other")]);
    rig.stop().await;
}

#[tokio::test]
async fn unreadable_or_oversize_copies_create_nothing_and_oversize_is_counted() {
    let dirs = Dirs::new();
    let caps = Caps {
        max_entry_bytes: 1024,
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
    // No scripted read: the owner offers no text.
    rig.fake
        .emit(rstory::backend::BackendEvent::SelectionChanged {
            types: vec!["image/png".to_string()],
            self_owned: false,
        });
    copy(&rig.fake, &"x".repeat(2000));
    copy(&rig.fake, "fits");
    c.event("history_changed").await;
    let entries = c.list().await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["preview"], "fits");
    let st = c.status().await;
    assert_eq!(st["counters"]["skipped_oversize"], 1);
    rig.stop().await;
}

#[tokio::test]
async fn without_a_backend_the_daemon_is_inactive_but_serves_the_socket() {
    let dirs = Dirs::new();
    let rig = Rig::start(
        &dirs,
        Opts {
            backend: false,
            ..Opts::default()
        },
    )
    .await;
    let mut c = rig.client().await;
    let st = c.status().await;
    assert_eq!(st["state"], "inactive");
    assert_eq!(st["reason"], "no_session");
    assert_eq!(st["backend"], "none");
    assert_eq!(st["mode"], json!(null));
    assert!(c.list().await.is_empty());
    rig.stop().await;
}

fn runtime(dirs: &Dirs, retry: Duration) -> Runtime {
    Runtime {
        store_dir: dirs.store.clone(),
        runtime_dir: dirs.runtime.clone(),
        caps: Caps::default(),
        launch: LaunchEnv::from_env(None),
        lock_retry: retry,
    }
}

async fn inactive() -> Provided<FakeBackend> {
    Provided {
        backend: None,
        info: BackendInfo::none("no_session"),
        connector: None,
    }
}

#[tokio::test]
async fn a_second_daemon_exits_4_naming_the_first_pid() {
    let dirs = Dirs::new();
    let (stop, rx) = watch::channel(false);
    let first = tokio::spawn(serve_process(runtime(&dirs, LOCK_RETRY), inactive, rx));
    support::wait_for_async(|| dirs.socket().exists()).await;
    let err = serve_process(
        runtime(&dirs, LOCK_RETRY),
        inactive,
        watch::channel(false).1,
    )
    .await
    .unwrap_err();
    assert_eq!(err.exit_code(), 4);
    assert_eq!(
        err.to_string(),
        format!("already running (pid {})", std::process::id())
    );
    stop.send(true).unwrap();
    first.await.unwrap().unwrap();
    assert!(
        !dirs.socket().exists(),
        "the socket is removed on a clean stop"
    );
}

#[tokio::test]
async fn a_reader_holding_the_shared_lock_is_waited_for_then_gives_exit_5() {
    let dirs = Dirs::new();
    // Create the store, then hold the read-only (shared) lock like `rstory ls` does.
    drop(
        rstory::store::Store::open_exclusive(
            &dirs.store,
            Caps::default(),
            rstory::store::fs::RealFs,
        )
        .unwrap(),
    );
    let reader = ReadOnlyStore::open(&dirs.store).unwrap();
    let clock = Clock::system();

    let start = Instant::now();
    let err = open_store(
        &dirs.store,
        Caps::default(),
        &dirs.runtime,
        Duration::from_millis(400),
        &clock,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, StartError::Environment(_)));
    assert_eq!(err.exit_code(), 5);
    assert_eq!(err.to_string(), "a client is reading the store; retry");
    assert!(
        start.elapsed() >= Duration::from_millis(400),
        "it retried first"
    );

    // Released within the window: the daemon gets in.
    let release = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        drop(reader);
    });
    open_store(
        &dirs.store,
        Caps::default(),
        &dirs.runtime,
        LOCK_RETRY,
        &clock,
    )
    .await
    .expect("opened after the reader let go");
    release.await.unwrap();
}

#[tokio::test]
async fn capture_to_listed_p95_is_under_100_ms() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    let mut ms = Vec::new();
    for i in 0..60 {
        let t = Instant::now();
        copy(&rig.fake, &format!("sample number {i}"));
        loop {
            let top = c.list().await.first().map(|e| e["preview"].clone());
            if top == Some(json!(format!("sample number {i}"))) {
                break;
            }
            tokio::task::yield_now().await;
        }
        ms.push(t.elapsed());
    }
    ms.sort();
    let p95 = ms[ms.len() * 95 / 100];
    assert!(p95 < Duration::from_millis(100), "p95 was {p95:?}");
    rig.stop().await;
}

#[tokio::test]
async fn a_burst_of_500_is_bounded_counted_and_leaves_the_daemon_responsive() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    // The test runtime is single-threaded: all 500 events are queued before the daemon runs.
    for i in 0..500 {
        copy(&rig.fake, &format!("burst {i:03}"));
    }
    c.event("history_changed").await;
    let st = c.status().await;
    assert_eq!(
        st["counters"]["dropped_burst"],
        500 - 256,
        "queue bound is 256"
    );
    assert_eq!(st["entries"], 20, "the cap still holds");
    assert_eq!(st["state"], "recording");
    // One batch, so one event, and the listing answers at once.
    assert_eq!(c.list().await.len(), 20);
    rig.stop().await;
}
