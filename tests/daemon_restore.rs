//! US2 (T006): restore acknowledges only after the backend owns the clipboard.

mod support;

use std::time::Duration;

use rstory::backend::{BackendEvent, ServeError};
use serde_json::json;
use support::daemon::{Dirs, Opts, Rig, copy};

async fn two_entries(rig: &Rig) -> support::daemon::Client {
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in ["alpha", "beta"] {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
    c
}

#[tokio::test(start_paused = true)]
async fn restore_serves_the_content_then_answers_ok_and_moves_it_to_the_top() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = two_entries(&rig).await;
    let before = c.list().await;
    assert_eq!(before[1]["preview"], "alpha");
    tokio::time::advance(Duration::from_secs(10)).await;

    let answer = c.call("restore", json!({"ref": "1"})).await;
    assert_eq!(answer["ok"], true, "{answer}");
    let served = rig.fake.served();
    assert_eq!(served.len(), 1, "served before the answer arrived");
    assert_eq!(&*served[0], b"alpha");

    let ev = c.event("history_changed").await;
    assert_eq!(ev["entries"], 2);
    let after = c.list().await;
    assert_eq!(after.len(), 2);
    assert_eq!(after[0]["preview"], "alpha", "restored entry is at the top");
    assert_ne!(
        after[0]["last_copied"], before[1]["last_copied"],
        "last_copied refreshed"
    );
    assert_eq!(after[0]["id"], before[1]["id"], "same entry, not a new one");
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn the_self_owned_change_creates_no_entry() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = two_entries(&rig).await;
    c.ok("restore", json!({"ref": "1"})).await;
    c.event("history_changed").await;
    // The backend reports the change our own serve caused.
    rig.fake.emit(BackendEvent::SelectionChanged {
        types: vec!["UTF8_STRING".to_string()],
        self_owned: true,
    });
    // A later foreign copy proves the self-owned one was fully processed first.
    copy(&rig.fake, "gamma");
    c.event("history_changed").await;
    let previews: Vec<_> = c
        .list()
        .await
        .iter()
        .map(|e| e["preview"].clone())
        .collect();
    assert_eq!(previews, [json!("gamma"), json!("alpha"), json!("beta")]);
    assert_eq!(
        rig.fake.read_limits().len(),
        3,
        "the self-owned change was not read"
    );
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn restore_by_short_id_and_unknown_ids() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = two_entries(&rig).await;
    let id = c.list().await[1]["id"].as_str().unwrap().to_string();
    c.ok("restore", json!({"ref": id})).await;
    assert_eq!(c.err("restore", json!({"ref": "9"})).await, "not_found");
    assert_eq!(c.err("restore", json!({"ref": "ZZZZ"})).await, "not_found");
    assert_eq!(rig.fake.served().len(), 1, "failed restores serve nothing");
    rig.stop().await;
}

#[tokio::test(start_paused = true)]
async fn a_backend_that_cannot_take_ownership_gives_backend_error_and_changes_nothing() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = two_entries(&rig).await;
    rig.fake.fail_serve(Some(ServeError::OwnershipFailed));
    let before = c.list().await;
    assert_eq!(c.err("restore", json!({"ref": "1"})).await, "backend_error");
    assert_eq!(c.list().await, before, "the order did not change");
    // Retrying after the backend recovers works (idempotent).
    rig.fake.fail_serve(None);
    c.ok("restore", json!({"ref": "1"})).await;
    rig.stop().await;
}

#[tokio::test]
async fn restore_while_inactive_is_the_inactive_error() {
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
    assert_eq!(c.err("restore", json!({"ref": "0"})).await, "inactive");
    rig.stop().await;
}
