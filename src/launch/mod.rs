//! Picker command and terminal detection.
//!
//! `plan` resolves which command opens the picker (pure, with an injectable
//! PATH lookup); `spawn` starts it detached, without a shell.

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
/// name or a path) is runnable.
pub fn plan(
    picker_command: Option<&str>,
    terminal: Option<&str>,
    on_path: &dyn Fn(&str) -> bool,
) -> LaunchPlan {
    let _ = (picker_command, terminal, on_path);
    LaunchPlan::NotFound { searched: vec![] }
}

/// Look `name` up in a `PATH`-style string (or check it directly if it
/// contains a slash); true when it is an executable regular file.
pub fn which_in(path: &str, name: &str) -> bool {
    let _ = (path, name);
    false
}

/// `which_in` against the process's own `PATH`.
pub fn on_path(name: &str) -> bool {
    let _ = name;
    false
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

/// Start the planned command detached; fire and forget.
pub fn spawn(plan: &LaunchPlan) -> Result<(), SpawnError> {
    let _ = plan;
    Ok(())
}

#[cfg(test)]
mod tests;
