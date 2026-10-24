//! 007 T017: the picker's data sources against a real daemon on a fake backend, and the
//! read-only fallback once it stops. All clipboard text is synthetic.

mod support;

use std::sync::mpsc;
use std::time::Duration;

use rstory::picker::data::{DaemonSource, Paths, ReadOnlySource, Source, open, subscribe};
use rstory::picker::state::{Action, Failure};
use support::daemon::{Dirs, Opts, Rig, copy};

async fn with_entries(rig: &Rig, texts: &[&str]) {
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in texts {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    tokio::task::spawn_blocking(f).await.unwrap()
}

#[tokio::test]
async fn the_daemon_source_lists_shows_and_acts() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    with_entries(&rig, &["alpha", "beta"]).await;
    let socket = rig.socket.clone();
    let (snap, content, pin, unpin, restore, rm, after) = blocking(move || {
        let mut s = DaemonSource::connect(&socket).unwrap();
        assert!(!s.read_only());
        let snap = s.list().unwrap();
        let alpha = snap.entries[1].id.clone();
        let content = s.show(&alpha).unwrap();
        let pin = s.act(Action::Pin, &alpha);
        let unpin = s.act(Action::Unpin, &alpha);
        let restore = s.act(Action::Restore, &alpha);
        let rm = s.act(Action::Remove, &alpha);
        let after = s.list().unwrap();
        (snap, content, pin, unpin, restore, rm, after)
    })
    .await;
    let previews: Vec<&str> = snap.entries.iter().map(|e| e.preview.as_str()).collect();
    assert_eq!(previews, ["beta", "alpha"]);
    assert!(snap.rec.is_some(), "the recording state comes from status");
    assert_eq!(content, "alpha");
    assert_eq!(pin, Ok(()));
    assert_eq!(unpin, Ok(()));
    assert_eq!(restore, Ok(()));
    assert_eq!(rm, Ok(()));
    assert_eq!(after.entries.len(), 1);
    assert_eq!(rig.fake.served().len(), 1);
    rig.stop().await;
}

#[tokio::test]
async fn the_pin_cap_is_a_pin_limit_failure() {
    let dirs = Dirs::new();
    let mut opts = Opts::default();
    opts.caps.max_pinned = 1;
    let rig = Rig::start(&dirs, opts).await;
    with_entries(&rig, &["alpha", "beta"]).await;
    let socket = rig.socket.clone();
    let second = blocking(move || {
        let mut s = DaemonSource::connect(&socket).unwrap();
        let snap = s.list().unwrap();
        s.act(Action::Pin, &snap.entries[0].id).unwrap();
        s.act(Action::Pin, &snap.entries[1].id)
    })
    .await;
    assert_eq!(second, Err(Failure::PinLimit(Some("1".into()))));
    rig.stop().await;
}

#[tokio::test]
async fn subscribe_reports_history_changes() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let (tx, rx) = mpsc::channel();
    let socket = rig.socket.clone();
    blocking(move || subscribe(&socket, move || tx.send(()).is_ok()).unwrap()).await;
    copy(&rig.fake, "gamma");
    let got = blocking(move || rx.recv_timeout(Duration::from_secs(5))).await;
    assert!(got.is_ok(), "no history_changed event");
    rig.stop().await;
}

#[tokio::test]
async fn without_a_daemon_open_falls_back_to_the_read_only_store() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    with_entries(&rig, &["alpha", "beta"]).await;
    rig.stop().await;
    let paths = Paths {
        socket: Some(dirs.socket()),
        store: Some(dirs.store.clone()),
    };
    let store = dirs.store.clone();
    let (ro, previews, content, act, reopened) = blocking(move || {
        let mut s = open(&paths).unwrap();
        let snap = s.list().unwrap();
        let previews: Vec<String> = snap.entries.iter().map(|e| e.preview.clone()).collect();
        let content = s.show(&snap.entries[0].id).unwrap();
        let act = s.act(Action::Restore, &snap.entries[0].id);
        // The handle was dropped: a writer could take the store now.
        let reopened = rstory::store::Store::open_exclusive_at(
            &store,
            rstory::config::Caps::default(),
            rstory::store::fs::RealFs,
            time::OffsetDateTime::now_utc(),
        )
        .is_ok();
        (s.read_only(), previews, content, act, reopened)
    })
    .await;
    assert!(ro);
    assert_eq!(previews, ["beta", "alpha"]);
    assert_eq!(content, "beta");
    assert_eq!(act, Err(Failure::DaemonDown));
    assert!(reopened, "the read-only source kept the store open");
}

#[test]
fn a_missing_store_is_an_empty_read_only_history() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = ReadOnlySource::new(dir.path().join("none"));
    let snap = s.list().unwrap();
    assert!(snap.entries.is_empty());
    assert!(snap.rec.is_none(), "the state is unknown without a daemon");
}

#[test]
fn nothing_reachable_is_an_error() {
    assert!(open(&Paths::default()).is_err());
}
