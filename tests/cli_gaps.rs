//! The docs-site gaps: `daemon --help`, the `theme` and `backend` config keys and `--ascii`
//! (specs 000, 004, 006, 011). The binary runs with an empty environment on temp directories.

mod support;

use support::daemon::Dirs;

#[tokio::test]
async fn daemon_help_prints_usage_on_stdout_and_exits_zero() {
    let dirs = Dirs::new();
    for flag in ["--help", "-h"] {
        let r = dirs.rstory(&["daemon", flag]).await;
        assert_eq!(r.code, 0, "{flag}: {}", r.stderr);
        assert_eq!(r.stderr, "", "{flag}");
        let out = r.out();
        assert!(out.contains("Usage: rstory daemon [--backend x11|wayland]"), "{out}");
        assert!(out.contains("--backend"), "{out}");
        assert!(out.contains("Examples:"), "{out}");
        assert!(out.contains("rstory daemon --backend x11"), "{out}");
    }
}

#[tokio::test]
async fn daemon_help_beats_a_bad_config_and_never_starts_the_daemon() {
    let dirs = Dirs::new();
    dirs.write_config("backend = \"xorg\"\n");
    let r = dirs.rstory(&["daemon", "--help"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!dirs.socket().exists());
}

#[tokio::test]
async fn daemon_unknown_argument_is_still_a_usage_error() {
    let dirs = Dirs::new();
    let r = dirs.rstory(&["daemon", "--nope"]).await;
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("unknown argument --nope"), "{}", r.stderr);
}

#[tokio::test]
async fn config_shows_the_default_theme_and_no_backend() {
    let dirs = Dirs::new();
    let out = dirs.rstory(&["config"]).await.out();
    assert!(out.contains("theme = \"terminal\"\n"), "{out}");
    assert!(!out.contains("backend"), "{out}");
}

#[tokio::test]
async fn config_shows_a_configured_theme_and_backend() {
    let dirs = Dirs::new();
    dirs.write_config("theme = \"rstory\"\nbackend = \"wayland\"\n");
    let r = dirs.rstory(&["config"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    let out = r.out();
    assert!(out.contains("theme = \"rstory\"\n"), "{out}");
    assert!(out.contains("backend = \"wayland\"\n"), "{out}");
}

#[tokio::test]
async fn config_rejects_unknown_values_naming_the_key() {
    for (text, key) in [("theme = \"neon\"\n", "theme"), ("backend = \"xorg\"\n", "backend")] {
        let dirs = Dirs::new();
        dirs.write_config(text);
        let r = dirs.rstory(&["config"]).await;
        assert_eq!(r.code, 5, "{text}");
        assert!(r.stderr.contains(key), "{text}: {}", r.stderr);
    }
}

#[tokio::test]
async fn the_daemon_refuses_an_unknown_backend_in_the_config() {
    let dirs = Dirs::new();
    dirs.write_config("backend = \"xorg\"\n");
    let r = dirs.rstory(&["daemon"]).await;
    assert_eq!(r.code, 5, "{}", r.stderr);
    assert!(r.stderr.contains("backend"), "{}", r.stderr);
}

fn check<'a>(doc: &'a serde_json::Value, id: &str) -> Option<&'a serde_json::Value> {
    doc["checks"].as_array().unwrap().iter().find(|c| c["id"] == id)
}

#[tokio::test]
async fn doctor_reports_a_configured_backend() {
    let dirs = Dirs::new();
    dirs.write_config("backend = \"x11\"\n");
    let r = dirs.rstory(&["doctor", "--json"]).await;
    let doc: serde_json::Value = serde_json::from_str(&r.out()).unwrap();
    let msg = check(&doc, "backend").unwrap()["message"].as_str().unwrap();
    assert!(msg.starts_with("x11"), "{msg}");
    assert!(msg.contains("config.toml"), "{msg}");
    assert!(check(&doc, "config").is_none());
}

#[tokio::test]
async fn doctor_reports_an_invalid_config_value() {
    let dirs = Dirs::new();
    dirs.write_config("backend = \"xorg\"\n");
    let r = dirs.rstory(&["doctor", "--json"]).await;
    let doc: serde_json::Value = serde_json::from_str(&r.out()).unwrap();
    let c = check(&doc, "config").expect("a config check");
    assert_eq!(c["status"], "fail");
    assert!(c["message"].as_str().unwrap().contains("backend"));
    assert!(c["hint"].is_string());
}

fn only_ascii(s: &str) -> bool {
    s.is_ascii()
}

#[tokio::test]
async fn ascii_flag_limits_doctor_to_ascii_in_both_positions() {
    let dirs = Dirs::new();
    let plain = dirs.rstory(&["doctor"]).await.out();
    assert!(!only_ascii(&plain), "the UTF-8 report uses glyphs: {plain}");
    for args in [&["--ascii", "doctor"][..], &["doctor", "--ascii"]] {
        let r = dirs.rstory(args).await;
        assert_eq!(r.code, 2, "{args:?}: {}", r.stderr);
        assert_eq!(r.stderr, "", "{args:?}");
        assert!(r.out().contains("Daemon"), "{args:?}: {}", r.out());
        assert!(only_ascii(&r.out()), "{args:?}: {}", r.out());
    }
}

#[tokio::test]
async fn ascii_is_accepted_by_pick_and_by_other_commands() {
    let dirs = Dirs::new();
    let r = dirs.rstory(&["pick", "--ascii"]).await;
    assert_eq!(r.code, 2);
    assert_eq!(r.stderr.trim_end(), "rstory pick needs a terminal");
    let r = dirs.rstory(&["--ascii", "config", "--path"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    let r = dirs.rstory(&["ls", "--ascii"]).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
}
