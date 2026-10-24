//! SC-003 (T010): every exit code of `contracts/cli.md` is produced by at least one run.
//! 130 is the shell's code for a process killed by SIGINT; the CLI installs no handler, so
//! the default action applies (covered by the unit test of `ExitCode::Interrupted`).

mod support;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::UnixListener;

use rstory::config::Caps;
use support::daemon::{Dirs, Opts, Rig, copy, run};

/// A socket that answers every request with the given error, like a misbehaving daemon.
fn fake_daemon(dirs: &Dirs, code: &'static str) {
    let dir = dirs.runtime.join("rstory");
    std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
    let listener = UnixListener::bind(dir.join("control.sock")).unwrap();
    std::thread::spawn(move || {
        while let Ok((stream, _)) = listener.accept() {
            let mut out = stream.try_clone().unwrap();
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                let v: serde_json::Value = serde_json::from_str(&line).unwrap();
                let reply = serde_json::json!({
                    "v": 1, "id": v["id"], "ok": false,
                    "error": {"code": code, "message": "synthetic failure"}
                });
                writeln!(out, "{reply}").unwrap();
            }
        }
    });
}

#[tokio::test]
async fn exit_0_ok() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    assert_eq!(dirs.rstory(&["ls"]).await.code, 0);
    assert_eq!(dirs.rstory(&["completions", "bash"]).await.code, 0);
    assert_eq!(dirs.rstory(&["--version"]).await.code, 0);
    rig.stop().await;
}

#[tokio::test]
async fn exit_1_unexpected_error() {
    let dirs = Dirs::new();
    fake_daemon(&dirs, "internal");
    let r = dirs.rstory(&["copy", "0"]).await;
    assert_eq!(r.code, 1, "{}", r.stderr);
    assert!(r.stderr.contains("synthetic failure"));
    let dirs = Dirs::new();
    fake_daemon(&dirs, "backend_error");
    assert_eq!(dirs.rstory(&["copy", "0"]).await.code, 1);
}

#[tokio::test]
async fn exit_2_usage_not_found_and_no_terminal() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    copy(&rig.fake, "alpha");
    rig.client().await.subscribe().await; // let the copy land
    assert_eq!(dirs.rstory(&["nonsense"]).await.code, 2, "unknown command");
    assert_eq!(dirs.rstory(&["show", "abc"]).await.code, 2, "bad ref");
    assert_eq!(dirs.rstory(&["show", "ZZZZ"]).await.code, 2, "not found");
    assert_eq!(
        dirs.rstory(&["pause", "--for", "x"]).await.code,
        2,
        "bad duration"
    );
    assert_eq!(
        dirs.rstory(&["clear"]).await.code,
        2,
        "no terminal and no --yes"
    );
    rig.stop().await;
}

#[tokio::test]
async fn exit_3_daemon_not_running() {
    let dirs = Dirs::new();
    assert_eq!(dirs.rstory(&["status"]).await.code, 3);
    assert_eq!(dirs.rstory(&["copy", "0"]).await.code, 3);
}

#[tokio::test]
async fn exit_4_daemon_already_running() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let r = dirs.rstory(&["daemon"]).await;
    assert_eq!(r.code, 4, "{}", r.stderr);
    rig.stop().await;
}

#[tokio::test]
async fn exit_5_environment_error() {
    let dirs = Dirs::new();
    let mut cmd = dirs.cli(&["ls"]);
    for var in [
        "XDG_RUNTIME_DIR",
        "XDG_STATE_HOME",
        "XDG_CONFIG_HOME",
        "HOME",
    ] {
        cmd.env_remove(var);
    }
    let r = run(cmd).await;
    assert_eq!(r.code, 5, "{}", r.stderr);
    assert!(r.stderr.contains("XDG_RUNTIME_DIR") || r.stderr.contains("XDG_STATE_HOME"));
}

#[tokio::test]
async fn exit_6_limit_reached() {
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
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in ["alpha", "beta"] {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
    assert_eq!(dirs.rstory(&["pin", "0"]).await.code, 0);
    assert_eq!(dirs.rstory(&["pin", "1"]).await.code, 6);
    rig.stop().await;
}

#[tokio::test]
async fn exit_7_inactive() {
    let dirs = Dirs::new();
    let rig = Rig::start(
        &dirs,
        Opts {
            backend: false,
            ..Opts::default()
        },
    )
    .await;
    let r = dirs.rstory(&["copy", "0"]).await;
    assert_eq!(r.code, 7, "{}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        "No clipboard access in this session. Run `rstory doctor` for details."
    );
    rig.stop().await;
}

#[test]
fn install_autostart_is_dispatched_to_the_installer() {
    // A bad flag is rejected by the installer itself (exit 2) before anything is written.
    let out = assert_cmd::Command::cargo_bin("rstory")
        .unwrap()
        .args(["install-autostart", "--no-such-flag"])
        .env("XDG_CONFIG_HOME", "/nonexistent/rstory-test-config")
        .env("HOME", "/nonexistent/rstory-test-home")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("usage: rstory install-autostart"), "{err}");
    assert!(!err.contains("not available in this build"), "{err}");
}

#[test]
fn passthrough_commands_answer_help_without_side_effects() {
    for cmd in ["doctor", "install-autostart", "install-gnome-extension"] {
        let out = assert_cmd::Command::cargo_bin("rstory")
            .unwrap()
            .args([cmd, "--help"])
            .env_clear()
            .env("HOME", "/nonexistent/rstory-test-home")
            .env("XDG_CONFIG_HOME", "/nonexistent/rstory-test-config")
            .env("XDG_DATA_HOME", "/nonexistent/rstory-test-data")
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0), "{cmd}");
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(
            text.contains(&format!("Usage: rstory {cmd}")),
            "{cmd}: {text}"
        );
        assert!(text.contains("Examples:"), "{cmd}");
    }
}
