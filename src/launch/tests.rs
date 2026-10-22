use super::*;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

fn avail(names: &[&str]) -> impl Fn(&str) -> bool {
    let set: HashSet<String> = names.iter().map(|s| (*s).to_string()).collect();
    move |n| set.contains(n)
}

fn argv(plan: LaunchPlan) -> (Source, Vec<String>) {
    match plan {
        LaunchPlan::Found { source, argv } => (source, argv),
        other => panic!("expected Found, got {other:?}"),
    }
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn table_rows_match_research() {
    let expected: &[(&str, &[&str])] = &[
        ("ghostty", &["-e"]),
        ("kitty", &[]),
        ("alacritty", &["-e"]),
        ("foot", &[]),
        ("wezterm", &["start", "--"]),
        ("ptyxis", &["--"]),
        ("kgx", &["--"]),
        ("gnome-terminal", &["--"]),
        ("konsole", &["-e"]),
        ("xfce4-terminal", &["-x"]),
        ("xterm", &["-e"]),
    ];
    assert_eq!(TERMINALS, expected);
}

/// One argv test per row (SC-001), through the fixed list.
fn row(name: &str, prefix: &[&str]) {
    let (source, got) = argv(plan(None, None, &avail(&[name])));
    let mut want = vec![name.to_string()];
    want.extend(strs(prefix));
    want.extend(strs(&["rstory", "pick"]));
    assert_eq!(source, Source::List, "{name}");
    assert_eq!(got, want, "{name}");
}

#[test]
fn row_ghostty() {
    row("ghostty", &["-e"]);
}
#[test]
fn row_kitty() {
    row("kitty", &[]);
}
#[test]
fn row_alacritty() {
    row("alacritty", &["-e"]);
}
#[test]
fn row_foot() {
    row("foot", &[]);
}
#[test]
fn row_wezterm() {
    row("wezterm", &["start", "--"]);
}
#[test]
fn row_ptyxis() {
    row("ptyxis", &["--"]);
}
#[test]
fn row_kgx() {
    row("kgx", &["--"]);
}
#[test]
fn row_gnome_terminal() {
    row("gnome-terminal", &["--"]);
}
#[test]
fn row_konsole() {
    row("konsole", &["-e"]);
}
#[test]
fn row_xfce4_terminal() {
    row("xfce4-terminal", &["-x"]);
}
#[test]
fn row_xterm() {
    row("xterm", &["-e"]);
}

#[test]
fn every_table_row_has_a_test() {
    // Guards the list above: adding a row without a test above fails here.
    assert_eq!(TERMINALS.len(), 11);
}

#[test]
fn list_order_prefers_earlier_rows() {
    let (_, got) = argv(plan(None, None, &avail(&["xterm", "kitty", "foot"])));
    assert_eq!(got[0], "kitty");
}

#[test]
fn config_wins_over_everything() {
    let on = avail(&["kitty", "alacritty", XDG_TERMINAL_EXEC]);
    let (source, got) = argv(plan(Some("foot -e rstory pick"), Some("alacritty"), &on));
    assert_eq!(source, Source::Config);
    assert_eq!(got, strs(&["foot", "-e", "rstory", "pick"]));
}

#[test]
fn config_is_split_like_shell_words_without_a_shell() {
    let (_, got) = argv(plan(
        Some(r#"my-term --title "a b" -e 'rstory pick' $HOME; ls"#),
        None,
        &avail(&[]),
    ));
    assert_eq!(
        got,
        strs(&[
            "my-term",
            "--title",
            "a b",
            "-e",
            "rstory pick",
            "$HOME;",
            "ls"
        ])
    );
}

#[test]
fn config_is_used_as_is_even_if_not_on_path() {
    let (source, got) = argv(plan(Some("/opt/x/term -e rstory pick"), None, &avail(&[])));
    assert_eq!(source, Source::Config);
    assert_eq!(got[0], "/opt/x/term");
}

#[test]
fn unusable_config_falls_through() {
    for bad in ["", "   ", "foot \"unterminated"] {
        let (source, _) = argv(plan(Some(bad), None, &avail(&["kitty"])));
        assert_eq!(source, Source::List, "{bad:?}");
    }
}

#[test]
fn env_known_terminal_uses_the_table() {
    let (source, got) = argv(plan(
        None,
        Some("alacritty"),
        &avail(&["alacritty", "kitty"]),
    ));
    assert_eq!(source, Source::Env);
    assert_eq!(got, strs(&["alacritty", "-e", "rstory", "pick"]));
    let (_, got) = argv(plan(None, Some("wezterm"), &avail(&["wezterm"])));
    assert_eq!(got, strs(&["wezterm", "start", "--", "rstory", "pick"]));
}

#[test]
fn env_path_to_known_terminal_uses_the_table_by_basename() {
    let (source, got) = argv(plan(
        None,
        Some("/usr/local/bin/foot"),
        &avail(&["/usr/local/bin/foot"]),
    ));
    assert_eq!(source, Source::Env);
    assert_eq!(got, strs(&["/usr/local/bin/foot", "rstory", "pick"]));
}

#[test]
fn env_unknown_terminal_gets_dash_e() {
    let (source, got) = argv(plan(None, Some("mytty"), &avail(&["mytty"])));
    assert_eq!(source, Source::Env);
    assert_eq!(got, strs(&["mytty", "-e", "rstory", "pick"]));
}

#[test]
fn env_not_on_path_or_empty_is_skipped() {
    let (source, _) = argv(plan(None, Some("ghost"), &avail(&["kitty"])));
    assert_eq!(source, Source::List);
    let (source, _) = argv(plan(None, Some(""), &avail(&["kitty"])));
    assert_eq!(source, Source::List);
}

#[test]
fn xdg_terminal_exec_beats_the_list() {
    let (source, got) = argv(plan(None, None, &avail(&[XDG_TERMINAL_EXEC, "kitty"])));
    assert_eq!(source, Source::Xdg);
    assert_eq!(got, strs(&["xdg-terminal-exec", "rstory", "pick"]));
}

#[test]
fn env_beats_xdg() {
    let (source, _) = argv(plan(
        None,
        Some("foot"),
        &avail(&["foot", XDG_TERMINAL_EXEC]),
    ));
    assert_eq!(source, Source::Env);
}

#[test]
fn not_found_lists_everything_searched() {
    match plan(None, Some("ghost"), &avail(&[])) {
        LaunchPlan::NotFound { searched } => {
            assert!(searched.iter().any(|s| s.contains("ghost")));
            assert!(searched.iter().any(|s| s == XDG_TERMINAL_EXEC));
            for (name, _) in TERMINALS {
                assert!(searched.iter().any(|s| s == name), "missing {name}");
            }
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
    match plan(None, None, &avail(&[])) {
        LaunchPlan::NotFound { searched } => {
            assert_eq!(searched.len(), TERMINALS.len() + 1);
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

fn stub(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    p
}

use std::path::{Path, PathBuf};

#[test]
fn which_in_finds_executables_on_a_path_string() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    stub(b.path(), "fake-term", "exit 0");
    fs::write(a.path().join("plain"), "x").unwrap();
    let path = format!("{}:{}", a.path().display(), b.path().display());
    assert!(which_in(&path, "fake-term"));
    assert!(!which_in(&path, "plain"), "not executable");
    assert!(!which_in(&path, "nope"));
    assert!(!which_in("", "nope"));
    let abs = b.path().join("fake-term");
    assert!(
        which_in("", abs.to_str().unwrap()),
        "slash path is checked directly"
    );
}

fn wait_for(path: &Path) -> String {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(s) = fs::read_to_string(path)
            && s.ends_with("done\n")
        {
            return s;
        }
        assert!(Instant::now() < end, "stub never wrote {path:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn spawn_runs_the_argv_detached_without_a_shell() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("argv.txt");
    let term = stub(
        dir.path(),
        "stub-term",
        &format!(
            "for a in \"$@\"; do printf '%s\\n' \"$a\" >> '{o}'; done; \
             printf 'pgid=%s pid=%s\\n' \"$(ps -o pgid= -p $$ | tr -d ' ')\" $$ >> '{o}'; \
             [ -t 0 ] && echo tty >> '{o}'; echo done >> '{o}'",
            o = out.display()
        ),
    );
    let plan = LaunchPlan::Found {
        source: Source::Config,
        argv: vec![
            term.to_str().unwrap().to_string(),
            "-e".into(),
            "rstory pick".into(),
            "$HOME; touch pwned".into(),
        ],
    };
    spawn(&plan).unwrap();
    let got = wait_for(&out);
    let lines: Vec<&str> = got.lines().collect();
    assert_eq!(&lines[..3], ["-e", "rstory pick", "$HOME; touch pwned"]);
    assert!(!dir.path().join("pwned").exists(), "no shell involved");
    assert!(!got.contains("tty"), "stdin is not a terminal");
    // Own process group: pgid == pid.
    let ids = lines[3].strip_prefix("pgid=").unwrap();
    let (pgid, pid) = ids.split_once(" pid=").unwrap();
    assert_eq!(pgid, pid, "child leads its own process group");
}

#[test]
fn spawn_missing_program_is_an_error_naming_only_the_program() {
    let plan = LaunchPlan::Found {
        source: Source::Config,
        argv: strs(&["/nonexistent/rstory-term", "--secret-arg", "rstory", "pick"]),
    };
    let err = spawn(&plan).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("/nonexistent/rstory-term"), "{msg}");
    assert!(!msg.contains("secret-arg"), "{msg}");
}

#[test]
fn spawn_not_found_plan_is_an_error() {
    let plan = LaunchPlan::NotFound {
        searched: strs(&["kitty"]),
    };
    assert!(spawn(&plan).is_err());
}
