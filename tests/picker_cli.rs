//! 007: `rstory pick` from the CLI without a terminal (FR-009 exit 2). Never opens a real
//! terminal: assert_cmd pipes stdout. Directories are temporary.

use assert_cmd::Command;

fn pick(args: &[&str]) -> assert_cmd::assert::Assert {
    let tmp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("rstory").unwrap();
    cmd.env_clear()
        .env("XDG_RUNTIME_DIR", tmp.path().join("run"))
        .env("XDG_STATE_HOME", tmp.path().join("state"))
        .env("XDG_CONFIG_HOME", tmp.path().join("config"))
        .env("HOME", tmp.path())
        .arg("pick")
        .args(args);
    cmd.assert()
}

#[test]
fn without_a_terminal_pick_exits_2_with_the_message() {
    let out = pick(&[]).code(2).get_output().clone();
    assert_eq!(
        String::from_utf8_lossy(&out.stderr).trim_end(),
        "rstory pick needs a terminal"
    );
    assert!(out.stdout.is_empty());
}

#[test]
fn plain_needs_a_terminal_too() {
    pick(&["--plain"]).code(2);
}

#[test]
fn an_unknown_argument_is_a_usage_error() {
    let out = pick(&["--fancy"]).code(2).get_output().clone();
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("Usage: rstory pick [--plain] [--ascii]")
    );
}

#[test]
fn help_prints_the_usage() {
    let out = pick(&["--help"]).code(0).get_output().clone();
    assert!(String::from_utf8_lossy(&out.stdout).contains("rstory pick [--plain] [--ascii]"));
}
