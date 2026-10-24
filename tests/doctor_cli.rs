//! `rstory doctor` as a process: the report against a real daemon on temp dirs, the exit scale,
//! read-only behavior, and the canary (T007): clipboard content never appears in its output
//! (Constitution III). The binary runs with an empty environment pointing only at temp dirs,
//! so nothing of the host session or `$HOME` is visible to it.

mod support;

use std::path::{Path, PathBuf};

use serde_json::Value;
use support::daemon::{Dirs, Opts, Rig, copy};

const CANARY: &str = "CANARY-5e1b27-do-not-print";

/// Every path under `dir` with its size, to prove nothing changed.
fn tree(dir: &Path) -> Vec<(PathBuf, u64)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let m = std::fs::symlink_metadata(e.path()).unwrap();
            out.push((e.path(), m.len()));
            if m.is_dir() {
                stack.push(e.path());
            }
        }
    }
    out.sort();
    out
}

fn check<'a>(doc: &'a Value, id: &str) -> &'a Value {
    doc["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == id)
        .unwrap_or_else(|| panic!("no {id} in {doc}"))
}

#[tokio::test]
async fn without_a_session_or_daemon_the_report_names_both_and_exits_two() {
    let dirs = Dirs::new();
    let r = dirs.rstory(&["doctor", "--json"]).await;
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert_eq!(r.stderr, "");
    let doc: Value = serde_json::from_str(&r.out()).unwrap();
    assert_eq!(doc["schema"], 1);
    assert_eq!(check(&doc, "session")["status"], "fail");
    assert!(
        check(&doc, "session")["message"]
            .as_str()
            .unwrap()
            .contains("no graphical session")
    );
    assert_eq!(check(&doc, "daemon")["status"], "fail");
    assert_eq!(check(&doc, "store")["status"], "ok");
}

#[tokio::test]
async fn text_output_has_the_summary_line() {
    let dirs = Dirs::new();
    let r = dirs.rstory(&["doctor"]).await;
    assert_eq!(r.code, 2);
    let text = r.out();
    assert!(
        text.contains("Session") && text.contains("Daemon"),
        "{text}"
    );
    assert!(
        text.trim_end().ends_with("failures."),
        "summary line in {text}"
    );
    assert!(!text.contains('\x1b'), "NO_COLOR is honored");
}

#[tokio::test]
async fn a_running_daemon_is_reported_and_the_report_is_read_only() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    copy(&rig.fake, "synthetic entry one");
    c.event("history_changed").await;
    let root = dirs._tmp.path().to_path_buf();
    let before = tree(&root);
    let r = dirs.rstory(&["doctor", "--json"]).await;
    let after = tree(&root);
    assert_eq!(before, after, "doctor writes nothing");
    let doc: Value = serde_json::from_str(&r.out()).unwrap();
    assert_eq!(check(&doc, "daemon")["status"], "ok");
    assert_eq!(check(&doc, "daemon")["message"], "running, recording");
    assert_eq!(
        check(&doc, "store")["message"],
        "private (0700/0600), 1 entry"
    );
    rig.stop().await;
}

#[tokio::test]
async fn a_pause_is_information_not_a_problem() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    assert_eq!(dirs.rstory(&["pause"]).await.code, 0);
    let doc: Value = serde_json::from_str(&dirs.rstory(&["doctor", "--json"]).await.out()).unwrap();
    let pause = check(&doc, "pause");
    assert_eq!(pause["status"], "info");
    assert!(pause["hint"].as_str().unwrap().contains("rstory resume"));
    rig.stop().await;
}

#[tokio::test]
async fn the_canary_is_never_printed_with_or_without_a_daemon() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    copy(&rig.fake, CANARY);
    c.event("history_changed").await;
    for args in [&["doctor"][..], &["doctor", "--json"]] {
        let r = dirs.rstory(args).await;
        assert!(!r.out().contains(CANARY), "{args:?}: daemon up");
        assert!(!r.stderr.contains(CANARY), "{args:?}: daemon up");
    }
    drop(c);
    rig.stop().await;
    // The daemon is gone: doctor reads the store itself.
    for args in [&["doctor"][..], &["doctor", "--json"]] {
        let r = dirs.rstory(args).await;
        assert!(!r.out().contains(CANARY), "{args:?}: daemon down");
        assert!(!r.stderr.contains(CANARY), "{args:?}: daemon down");
        assert!(r.out().contains("1 entry"), "the entry was counted");
    }
}

#[tokio::test]
async fn a_store_open_to_others_fails_with_the_chmod_hint() {
    use std::os::unix::fs::PermissionsExt;
    let dirs = Dirs::new();
    std::fs::create_dir_all(&dirs.store).unwrap();
    std::fs::set_permissions(&dirs.store, std::fs::Permissions::from_mode(0o755)).unwrap();
    let doc: Value = serde_json::from_str(&dirs.rstory(&["doctor", "--json"]).await.out()).unwrap();
    let store = check(&doc, "store");
    assert_eq!(store["status"], "fail");
    let dir = dirs.store.display();
    assert_eq!(
        store["hint"].as_str().unwrap(),
        format!("chmod 700 {dir} && chmod 600 {dir}/*")
    );
}

#[tokio::test]
async fn an_unknown_argument_is_a_usage_error() {
    let dirs = Dirs::new();
    let r = dirs.rstory(&["doctor", "--nope"]).await;
    assert_eq!(r.code, 2);
    assert!(r.out().is_empty());
    assert!(r.stderr.contains("--nope"), "{}", r.stderr);
}
