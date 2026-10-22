//! Picker command and terminal detection.
//!
//! `plan` resolves which command opens the picker (pure, with an injectable
//! PATH lookup); `spawn` starts it detached, without a shell.

use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

/// The picker's own argv, appended after a terminal's exec prefix.
pub const PICKER_ARGV: &[&str] = &["rstory", "pick"];

/// Fixed terminal list: (program, args placed before the picker argv), in
/// detection order. One row per terminal, one test per row.
pub const TERMINALS: &[(&str, &[&str])] = &[
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

/// The freedesktop `xdg-terminal-exec` launcher; takes the command directly.
pub const XDG_TERMINAL_EXEC: &str = "xdg-terminal-exec";

/// Where a resolved command came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Config,
    Env,
    Xdg,
    List,
}

/// The outcome of resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchPlan {
    Found { source: Source, argv: Vec<String> },
    NotFound { searched: Vec<String> },
}

/// Resolve the picker command. `on_path` answers whether a program (a bare
/// name or a path) is runnable. An empty or unparsable `picker_command` is
/// skipped; a configured command is used as is (no lookup).
pub fn plan(
    picker_command: Option<&str>,
    terminal: Option<&str>,
    on_path: &dyn Fn(&str) -> bool,
) -> LaunchPlan {
    let mut searched = Vec::new();

    if let Some(cmd) = picker_command
        && let Ok(argv) = shell_words::split(cmd)
        && !argv.is_empty()
    {
        return LaunchPlan::Found {
            source: Source::Config,
            argv,
        };
    }

    if let Some(term) = terminal.filter(|t| !t.is_empty()) {
        if on_path(term) {
            let base = term.rsplit('/').next().unwrap_or(term);
            let prefix: &[&str] = TERMINALS
                .iter()
                .find(|(n, _)| *n == base)
                .map_or(&["-e"], |(_, p)| p);
            return found(Source::Env, term, prefix);
        }
        searched.push(format!("$TERMINAL={term}"));
    }

    if on_path(XDG_TERMINAL_EXEC) {
        return found(Source::Xdg, XDG_TERMINAL_EXEC, &[]);
    }
    searched.push(XDG_TERMINAL_EXEC.to_string());

    for (name, prefix) in TERMINALS {
        if on_path(name) {
            return found(Source::List, name, prefix);
        }
        searched.push((*name).to_string());
    }
    LaunchPlan::NotFound { searched }
}

fn found(source: Source, program: &str, prefix: &[&str]) -> LaunchPlan {
    let argv = std::iter::once(program)
        .chain(prefix.iter().copied())
        .chain(PICKER_ARGV.iter().copied())
        .map(str::to_string)
        .collect();
    LaunchPlan::Found { source, argv }
}

/// Look `name` up in a `PATH`-style string (or check it directly if it
/// contains a slash); true when it is an executable regular file.
pub fn which_in(path: &str, name: &str) -> bool {
    if name.contains('/') {
        return is_exec(Path::new(name));
    }
    path.split(':')
        .filter(|d| !d.is_empty())
        .any(|d| is_exec(&Path::new(d).join(name)))
}

fn is_exec(p: &Path) -> bool {
    std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// `which_in` against the process's own `PATH`.
pub fn on_path(name: &str) -> bool {
    which_in(&std::env::var("PATH").unwrap_or_default(), name)
}

/// Why a launch failed. Carries the program name only, never arguments.
#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("empty command")]
    Empty,
    #[error("could not start {program}: {source}")]
    Start {
        program: String,
        #[source]
        source: std::io::Error,
    },
}

/// Start the planned command detached (own process group, null stdio, no
/// shell, caller's environment); fire and forget.
pub fn spawn(plan: &LaunchPlan) -> Result<(), SpawnError> {
    let LaunchPlan::Found { source, argv } = plan else {
        return Err(SpawnError::Empty);
    };
    let (program, args) = argv.split_first().ok_or(SpawnError::Empty)?;
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|source| SpawnError::Start {
            program: program.clone(),
            source,
        })?;
    tracing::info!(source = ?source, program = %program, "launch.spawn");
    // Reap the child so it never lingers as a zombie.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests;
