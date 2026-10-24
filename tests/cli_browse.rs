//! US1 (T003): `ls`, `show` and `copy` against a daemon with a fake backend.
//! All clipboard text is synthetic.

mod support;

use serde_json::Value;
use support::daemon::{Dirs, Opts, Rig, copy};

/// Three entries, newest last copied: `gamma` (0), `beta` (1), `alpha` (2).
async fn three(rig: &Rig) {
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in ["alpha", "beta", "gamma"] {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
}

/// Replaces the age column of piped `ls` lines: it depends on the wall clock.
fn without_ages(tsv: &str) -> String {
    tsv.lines()
        .map(|l| {
            let mut f: Vec<&str> = l.split('\t').collect();
            assert_eq!(f.len(), 5, "five columns: {l:?}");
            assert!(f[3].parse::<u64>().is_ok(), "age is seconds: {l:?}");
            f[3] = "AGE";
            f.join("\t")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Short ids are random; replace them by their position.
fn without_ids(tsv: &str) -> String {
    tsv.lines()
        .map(|l| {
            let mut f: Vec<&str> = l.split('\t').collect();
            f[1] = "ID";
            f.join("\t")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::test]
async fn ls_piped_is_tab_separated_newest_first() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    three(&rig).await;
    let r = dirs.rstory(&["ls"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(r.stderr, "");
    insta::assert_snapshot!(without_ids(&without_ages(&r.out())), @r"
    0	ID		AGE	gamma
    1	ID		AGE	beta
    2	ID		AGE	alpha
    ");
    rig.stop().await;
}

#[tokio::test]
async fn ls_marks_pinned_entries_and_honors_limit() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    three(&rig).await;
    rig.client()
        .await
        .ok("pin", serde_json::json!({"ref": "2"}))
        .await;
    let r = dirs.rstory(&["ls", "--limit", "2"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    // pinning does not reorder: alpha stays at 2, so it is cut off by the limit
    assert_eq!(r.out().lines().count(), 2);
    let all = dirs.rstory(&["ls"]).await.out();
    let pinned: Vec<&str> = all.lines().filter(|l| l.contains("\tpinned\t")).collect();
    assert_eq!(pinned.len(), 1);
    assert!(pinned[0].ends_with("alpha"), "{pinned:?}");
    rig.stop().await;
}

#[tokio::test]
async fn ls_json_has_the_entry_list_shape() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    three(&rig).await;
    let r = dirs.rstory(&["ls", "--json"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    let v: Value = serde_json::from_str(&r.out()).unwrap();
    assert_eq!(v["schema"], 1);
    assert_eq!(v["read_only"], false);
    let entries = v["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["preview"], "gamma");
    assert_eq!(entries[0]["index"], 0);
    assert_eq!(entries[2]["preview"], "alpha");
    for k in ["id", "short_id", "size", "created", "last_copied", "pinned"] {
        assert!(entries[0].get(k).is_some(), "{k}");
    }
    assert!(r.out().ends_with('\n'));
    rig.stop().await;
}

#[tokio::test]
async fn ls_of_an_empty_history_prints_nothing() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let r = dirs.rstory(&["ls"]).await;
    assert_eq!((r.code, r.out().as_str(), r.stderr.as_str()), (0, "", ""));
    let j = dirs.rstory(&["ls", "--json"]).await;
    let v: Value = serde_json::from_str(&j.out()).unwrap();
    assert_eq!(v["entries"].as_array().unwrap().len(), 0);
    rig.stop().await;
}

#[tokio::test]
async fn show_prints_the_exact_bytes_by_index_and_by_short_id() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    copy(&rig.fake, "line one\nline two\ttabbed");
    c.event("history_changed").await;
    let short = c.list().await[0]["short_id"].as_str().unwrap().to_string();

    let by_index = dirs.rstory(&["show", "0"]).await;
    assert_eq!(by_index.code, 0, "{}", by_index.stderr);
    assert_eq!(
        by_index.stdout, b"line one\nline two\ttabbed",
        "no newline added"
    );
    let by_id = dirs.rstory(&["show", &short.to_lowercase()]).await;
    assert_eq!(by_id.stdout, b"line one\nline two\ttabbed");
    rig.stop().await;
}

#[tokio::test]
async fn show_json_has_the_entry_shape() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    three(&rig).await;
    let r = dirs.rstory(&["show", "1", "--json"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    let v: Value = serde_json::from_str(&r.out()).unwrap();
    assert_eq!(v["schema"], 1);
    assert_eq!(v["content"], "beta");
    assert_eq!(v["entry"]["preview"], "beta");
    assert_eq!(v["entry"]["index"], 1);
    rig.stop().await;
}

#[tokio::test]
async fn copy_restores_silently_and_moves_the_entry_to_the_top() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    three(&rig).await;
    let r = dirs.rstory(&["copy", "2"]).await;
    assert_eq!((r.code, r.stdout.len(), r.stderr.as_str()), (0, 0, ""));
    let served = rig.fake.served();
    assert_eq!(served.len(), 1);
    assert_eq!(&*served[0], b"alpha");
    let mut c = rig.client().await;
    assert_eq!(c.list().await[0]["preview"], "alpha");
    rig.stop().await;
}

#[tokio::test]
async fn an_unknown_reference_exits_2_naming_it() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    three(&rig).await;
    for args in [["show", "ZZZZ"], ["copy", "ZZZZ"], ["show", "7"]] {
        let r = dirs.rstory(&args).await;
        assert_eq!(r.code, 2, "{args:?}: {}", r.stderr);
        assert!(
            r.stderr.starts_with(&format!("No entry {}", args[1])),
            "{}",
            r.stderr
        );
        assert!(r.stdout.is_empty());
    }
    rig.stop().await;
}

#[tokio::test]
async fn an_ambiguous_short_id_lists_the_candidates_and_exits_2() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    three(&rig).await;
    let mut c = rig.client().await;
    let shorts: Vec<String> = c
        .list()
        .await
        .iter()
        .map(|e| e["short_id"].as_str().unwrap().to_string())
        .collect();
    // ids made at the same instant share their leading characters
    let prefix: String = shorts[0].chars().take(4).collect();
    let r = dirs.rstory(&["show", &prefix]).await;
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(
        r.stderr
            .contains(&format!("Short id {prefix} matches 3 entries")),
        "{}",
        r.stderr
    );
    for s in &shorts {
        assert!(r.stderr.contains(s.as_str()), "{} lists {s}", r.stderr);
    }
    rig.stop().await;
}

#[tokio::test]
async fn a_malformed_reference_is_a_usage_error() {
    let dirs = Dirs::new();
    let r = dirs.rstory(&["show", "abc"]).await;
    assert_eq!(r.code, 2);
    assert!(r.stdout.is_empty());
}
