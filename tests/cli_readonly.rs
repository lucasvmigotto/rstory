//! US3 (T008): `ls` and `show` read the store while the daemon is down; writes exit 3.
//! All clipboard text is synthetic.

mod support;

use rstory::backend::BackendEvent;
use serde_json::Value;
use support::daemon::{Dirs, Opts, Rig, copy};

/// A store with `alpha`, `beta` (newest) written by a daemon that is then stopped.
async fn prepared(dirs: &Dirs, texts: &[&str]) -> Vec<Value> {
    let rig = Rig::start(dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in texts {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
    let list = c.list().await;
    drop(c);
    rig.stop().await;
    list
}

const HINT: &str = "(daemon not running, read-only)";

#[tokio::test]
async fn ls_reads_the_store_and_says_so_on_stderr() {
    let dirs = Dirs::new();
    prepared(&dirs, &["alpha", "beta"]).await;
    let r = dirs.rstory(&["ls"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(r.stderr.trim_end(), HINT);
    let text = r.stdout_text();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(
        lines[0].starts_with("0\t") && lines[0].ends_with("\tbeta"),
        "{lines:?}"
    );
    assert!(
        lines[1].starts_with("1\t") && lines[1].ends_with("\talpha"),
        "{lines:?}"
    );
}

#[tokio::test]
async fn ls_json_flags_the_read_only_listing() {
    let dirs = Dirs::new();
    let before = prepared(&dirs, &["alpha", "beta"]).await;
    let r = dirs.rstory(&["ls", "--json", "--limit", "1"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    let v: Value = serde_json::from_str(&r.stdout_text()).unwrap();
    assert_eq!(v["schema"], 1);
    assert_eq!(v["read_only"], true);
    assert_eq!(v["entries"].as_array().unwrap().len(), 1);
    assert_eq!(v["entries"][0]["id"], before[0]["id"]);
    assert_eq!(v["entries"][0]["short_id"], before[0]["short_id"]);
    assert_eq!(v["entries"][0]["preview"], "beta");
}

#[tokio::test]
async fn show_prints_the_stored_bytes_by_index_and_by_short_id() {
    let dirs = Dirs::new();
    let before = prepared(&dirs, &["alpha\nline two", "beta"]).await;
    let by_index = dirs.rstory(&["show", "1"]).await;
    assert_eq!(by_index.code, 0, "{}", by_index.stderr);
    assert_eq!(by_index.stdout, b"alpha\nline two");
    assert_eq!(by_index.stderr, "", "show adds no hint");
    let short = before[1]["short_id"].as_str().unwrap();
    assert_eq!(
        dirs.rstory(&["show", short]).await.stdout,
        b"alpha\nline two"
    );
    let json = dirs.rstory(&["show", "0", "--json"]).await;
    let v: Value = serde_json::from_str(&json.stdout_text()).unwrap();
    assert_eq!(
        (v["content"].as_str(), v["entry"]["index"].as_u64()),
        (Some("beta"), Some(0))
    );
}

#[tokio::test]
async fn show_writes_non_utf8_content_as_stored() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    rig.fake.push_read(Ok(vec![b'a', 0xff, b'b']));
    rig.fake.emit(BackendEvent::SelectionChanged {
        types: vec!["UTF8_STRING".to_string()],
        self_owned: false,
    });
    c.event("history_changed").await;
    drop(c);
    rig.stop().await;
    let r = dirs.rstory(&["show", "0"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(r.stdout, [b'a', 0xff, b'b']);
}

#[tokio::test]
async fn unknown_and_ambiguous_references_exit_2() {
    let dirs = Dirs::new();
    let before = prepared(&dirs, &["alpha", "beta", "gamma"]).await;
    let r = dirs.rstory(&["show", "ZZZZ"]).await;
    assert_eq!(r.code, 2);
    assert!(r.stderr.starts_with("No entry ZZZZ."), "{}", r.stderr);
    let r = dirs.rstory(&["show", "9"]).await;
    assert_eq!((r.code, r.stdout.len()), (2, 0));
    let prefix: String = before[0]["short_id"]
        .as_str()
        .unwrap()
        .chars()
        .take(4)
        .collect();
    let r = dirs.rstory(&["show", &prefix]).await;
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("matches 3 entries"), "{}", r.stderr);
    for e in &before {
        assert!(
            r.stderr.contains(e["short_id"].as_str().unwrap()),
            "{}",
            r.stderr
        );
    }
}

#[tokio::test]
async fn no_store_at_all_is_an_empty_history() {
    let dirs = Dirs::new();
    let r = dirs.rstory(&["ls"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(r.stdout.len(), 0);
    assert_eq!(r.stderr.trim_end(), HINT);
    assert_eq!(dirs.rstory(&["show", "0"]).await.code, 2);
    assert!(!dirs.store.exists(), "reading created nothing");
}

#[tokio::test]
async fn writes_exit_3_and_change_nothing() {
    let dirs = Dirs::new();
    prepared(&dirs, &["alpha", "beta"]).await;
    for args in [
        &["copy", "0"][..],
        &["pin", "0"],
        &["rm", "0"],
        &["clear", "--yes"],
        &["pause"],
        &["resume"],
    ] {
        let r = dirs.rstory(args).await;
        assert_eq!(r.code, 3, "{args:?}: {}", r.stderr);
        assert!(r.stderr.starts_with("Daemon not running."), "{}", r.stderr);
    }
    assert_eq!(dirs.rstory(&["ls"]).await.stdout_text().lines().count(), 2);
}

#[tokio::test]
async fn a_finished_read_does_not_block_the_daemon_from_starting() {
    let dirs = Dirs::new();
    prepared(&dirs, &["alpha"]).await;
    assert_eq!(dirs.rstory(&["ls"]).await.code, 0);
    let rig = Rig::start(&dirs, Opts::default()).await; // takes the exclusive lock
    let r = dirs.rstory(&["ls"]).await;
    assert_eq!(r.code, 0);
    assert_eq!(r.stderr, "", "the daemon answers now: no read-only hint");
    rig.stop().await;
}
