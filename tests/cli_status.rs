//! US4 (T010): `status` and `config`.

mod support;

use serde_json::Value;
use support::daemon::{Dirs, Opts, Rig, copy};

#[tokio::test]
async fn status_text_reports_state_counts_and_counters() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in ["alpha", "beta"] {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
    c.ok("pin", serde_json::json!({"ref": "0"})).await;
    let r = dirs.rstory(&["status"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(r.stderr, "");
    let lines: Vec<String> = r.out().lines().map(str::to_string).collect();
    assert_eq!(lines.len(), 3, "{lines:?}");
    assert_eq!(
        lines[0],
        "Recording · backend x11 · 2 entries (1 pinned) of 20"
    );
    assert_eq!(
        lines[1],
        "Skipped: 0 sensitive · 0 oversize · 0 while paused"
    );
    assert!(lines[2].starts_with("Tray: "), "{}", lines[2]);
    assert!(lines[2].contains(" · Picker: "), "{}", lines[2]);
    rig.stop().await;
}

#[tokio::test]
async fn status_json_is_the_status_schema() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let r = dirs.rstory(&["status", "--json"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    let v: Value = serde_json::from_str(&r.out()).unwrap();
    assert_eq!(v["schema"], 1);
    for k in [
        "state",
        "backend",
        "mode",
        "entries",
        "pinned",
        "max_entries",
        "max_pinned",
        "counters",
        "tray",
        "picker_available",
        "version",
        "protocol",
    ] {
        assert!(v.get(k).is_some(), "{k} in {v}");
    }
    assert_eq!(v["state"], "recording");
    assert_eq!(v["backend"], "x11");
    assert_eq!(v["protocol"], 1);
    rig.stop().await;
}

#[tokio::test]
async fn a_paused_daemon_shows_paused_and_the_end() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    assert_eq!(dirs.rstory(&["pause", "--for", "1h"]).await.code, 0);
    let r = dirs.rstory(&["status"]).await;
    assert!(r.out().starts_with("Paused"), "{}", r.out());
    let j: Value = serde_json::from_str(&dirs.rstory(&["status", "--json"]).await.out()).unwrap();
    assert_eq!(j["state"], "paused");
    assert!(j["paused_until"].is_string());
    rig.stop().await;
}

#[tokio::test]
async fn an_inactive_daemon_still_answers_status() {
    let dirs = Dirs::new();
    let opts = Opts {
        backend: false,
        ..Opts::default()
    };
    let rig = Rig::start(&dirs, opts).await;
    let r = dirs.rstory(&["status"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.out().starts_with("Inactive — "), "{}", r.out());
    let j: Value = serde_json::from_str(&dirs.rstory(&["status", "--json"]).await.out()).unwrap();
    assert_eq!(
        (j["state"].as_str(), j["backend"].as_str()),
        (Some("inactive"), Some("none"))
    );
    rig.stop().await;
}

#[tokio::test]
async fn status_without_a_daemon_exits_3() {
    let dirs = Dirs::new();
    for args in [&["status"][..], &["status", "--json"]] {
        let r = dirs.rstory(args).await;
        assert_eq!(r.code, 3, "{args:?}");
        assert!(r.stdout.is_empty());
        assert!(r.stderr.starts_with("Daemon not running."), "{}", r.stderr);
    }
}

#[tokio::test]
async fn config_without_a_file_prints_the_defaults_and_where_the_file_would_be() {
    let dirs = Dirs::new();
    let r = dirs.rstory(&["config"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    let path = dirs.config_file();
    let expected = format!(
        "# {} (not found, showing the defaults)\nmax_entries = 20\nmax_pinned = 10\nmax_entry_bytes = 1048576\nttl = \"7d\"\ntheme = \"terminal\"\n",
        path.display()
    );
    assert_eq!(r.out(), expected);
}

#[tokio::test]
async fn config_shows_what_the_file_changes() {
    let dirs = Dirs::new();
    dirs.write_config("max_entries = 50\nmax_pinned = 3\npicker_command = \"kitty -e\"\n");
    let r = dirs.rstory(&["config"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    let out = r.out();
    assert!(
        out.starts_with(&format!("# {}\n", dirs.config_file().display())),
        "{out}"
    );
    assert!(out.contains("max_entries = 50\n"), "{out}");
    assert!(out.contains("max_pinned = 3\n"), "{out}");
    assert!(out.contains("picker_command = \"kitty -e\"\n"), "{out}");
}

#[tokio::test]
async fn config_path_prints_only_the_path() {
    let dirs = Dirs::new();
    let r = dirs.rstory(&["config", "--path"]).await;
    assert_eq!(r.code, 0);
    assert_eq!(r.out(), format!("{}\n", dirs.config_file().display()));
    dirs.write_config("max_entries = 50\n");
    assert_eq!(dirs.rstory(&["config", "--path"]).await.out(), r.out());
}

#[tokio::test]
async fn an_invalid_config_is_an_environment_error() {
    let dirs = Dirs::new();
    dirs.write_config("max_entries = 1\n");
    let r = dirs.rstory(&["config"]).await;
    assert_eq!(r.code, 5, "{}", r.stderr);
    assert!(r.stderr.contains("max_entries"), "{}", r.stderr);
    assert!(r.stdout.is_empty());
    // the path alone needs no valid file
    assert_eq!(dirs.rstory(&["config", "--path"]).await.code, 0);
}
