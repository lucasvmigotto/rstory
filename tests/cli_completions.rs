//! T013: the completion scripts come out for every supported shell and mention the commands.

mod support;

use std::io::Write;
use std::process::{Command, Stdio};

use support::daemon::Dirs;

#[tokio::test]
async fn each_shell_gets_a_script_naming_every_command() {
    let dirs = Dirs::new();
    for shell in ["bash", "zsh", "fish"] {
        let r = dirs.rstory(&["completions", shell]).await;
        assert_eq!(r.code, 0, "{shell}: {}", r.stderr);
        assert_eq!(r.stderr, "", "{shell}");
        let script = r.out();
        for cmd in [
            "ls",
            "show",
            "copy",
            "pin",
            "unpin",
            "rm",
            "clear",
            "pause",
            "resume",
            "status",
            "config",
            "completions",
            "daemon",
            "pick",
            "doctor",
            "install-autostart",
            "install-gnome-extension",
        ] {
            assert!(script.contains(cmd), "{shell} script lacks {cmd}");
        }
        assert!(
            script.contains("json"),
            "{shell} script lacks the json flag"
        );
    }
}

#[tokio::test]
async fn the_bash_script_is_valid_bash() {
    let dirs = Dirs::new();
    let script = dirs.rstory(&["completions", "bash"]).await.stdout;
    let Ok(mut bash) = Command::new("bash")
        .arg("-n")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    else {
        return; // no bash in this environment
    };
    bash.stdin.take().unwrap().write_all(&script).unwrap();
    let out = bash.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[tokio::test]
async fn an_unknown_shell_is_a_usage_error() {
    let dirs = Dirs::new();
    let r = dirs.rstory(&["completions", "powershell"]).await;
    assert_eq!(r.code, 2);
    assert!(r.stdout.is_empty());
    assert_eq!(dirs.rstory(&["completions"]).await.code, 2);
}
