//! The commands of `contracts/cli.md` that talk to the daemon or the store.
//!
//! Each command writes machine output to `out` and diagnostics to `err`, and returns an
//! error carrying its exit code. Nothing here logs or prints clipboard content except `show`.

use std::io::{self, BufRead, IsTerminal, Write};

use serde_json::{Value, json};
use time::OffsetDateTime;

use super::ExitCode;
use super::env::Env;
use super::error::CliError;
use super::render::{self, Term};
use super::source;
use super::status;
use crate::config::{
    BackendName, Caps, ConfigError, backend_from_toml, picker_command_from_toml, theme_from_toml,
};
use crate::ipc::protocol::Status;
use crate::ui::copy;

/// Everything a command needs from its surroundings.
pub struct Io<'a> {
    /// Paths of the socket, store and config.
    pub env: &'a Env,
    /// What stdout is.
    pub term: &'a Term,
    /// Machine output.
    pub out: &'a mut dyn Write,
    /// Diagnostics.
    pub err: &'a mut dyn Write,
}

/// Writes to a pipe that may close early (`rstory ls | head`): that is not an error.
fn emit(out: &mut dyn Write, bytes: &[u8]) -> Result<(), CliError> {
    match out.write_all(bytes).and_then(|()| out.flush()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(CliError::new(
            ExitCode::Unexpected,
            format!("Cannot write the output: {}", e.kind()),
        )),
    }
}

fn note(err: &mut dyn Write, line: &str) {
    let _ = writeln!(err, "{line}");
}

/// `rstory ls [--json] [--limit N]`.
///
/// # Errors
/// See [`source::list`].
pub fn ls(io: &mut Io<'_>, json: bool, limit: Option<u32>) -> Result<(), CliError> {
    let listing = source::list(io.env, limit)?;
    if listing.read_only {
        note(io.err, copy::CLI_READONLY);
    }
    let text = if json {
        render::ls_json(&listing.entries, listing.read_only)
    } else {
        render::ls_text(&listing.entries, OffsetDateTime::now_utc(), io.term)
    };
    emit(io.out, text.as_bytes())
}

/// `rstory show <ref> [--json]`.
///
/// # Errors
/// See [`source::show`].
pub fn show(io: &mut Io<'_>, reference: &str, json: bool) -> Result<(), CliError> {
    let shown = source::show(io.env, reference)?;
    if json {
        emit(
            io.out,
            render::show_json(&shown.entry, &shown.content).as_bytes(),
        )
    } else {
        emit(io.out, &shown.content)
    }
}

/// `rstory copy <ref>`: silent on success.
///
/// # Errors
/// Exit 3 without a daemon, 7 when inactive, 2 for an unknown reference.
pub fn copy_entry(io: &mut Io<'_>, reference: &str) -> Result<(), CliError> {
    let mut client = source::require_daemon(io.env)?;
    client
        .call("restore", json!({"ref": reference}))
        .map(|_| ())
        .map_err(|e| CliError::from_client(e, Some(reference)))
}

/// `rstory status [--json]`.
///
/// # Errors
/// Exit 3 without a daemon.
pub fn status(io: &mut Io<'_>, json: bool) -> Result<(), CliError> {
    let mut client = source::require_daemon(io.env)?;
    let result = client
        .call("status", json!({}))
        .map_err(|e| CliError::from_client(e, None))?;
    let status: Status = serde_json::from_value(result).map_err(|_| {
        CliError::new(
            ExitCode::Unexpected,
            "The daemon sent a status rstory does not understand.",
        )
    })?;
    let text = if json {
        status::status_json(&status)
    } else {
        status::status_text(&status, OffsetDateTime::now_utc(), io.term)
    };
    emit(io.out, text.as_bytes())
}

/// `rstory config [--path]`: needs no daemon.
///
/// # Errors
/// Exit 5 when the config file cannot be located, read or is invalid.
pub fn config(io: &mut Io<'_>, path_only: bool) -> Result<(), CliError> {
    let path = io.env.config_path()?;
    if path_only {
        return emit(io.out, format!("{}\n", path.display()).as_bytes());
    }
    let (text, exists) = match std::fs::read_to_string(&path) {
        Ok(t) => (t, true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => (String::new(), false),
        Err(e) => {
            return Err(CliError::environment(format!(
                "Cannot read {}: {}",
                path.display(),
                e.kind()
            )));
        }
    };
    let invalid = |e: ConfigError| CliError::environment(format!("{}: {e}", path.display()));
    let caps = Caps::from_toml(&text).map_err(invalid)?;
    let picker = picker_command_from_toml(&text).map_err(invalid)?;
    let theme = theme_from_toml(&text).map_err(invalid)?;
    let backend = backend_from_toml(&text).map_err(invalid)?;
    let view = status::ConfigView {
        theme: theme.as_str(),
        backend: backend.map(BackendName::as_str),
        picker_command: picker.as_deref(),
    };
    emit(
        io.out,
        status::config_text(&path, exists, &caps, &view).as_bytes(),
    )
}

/// `rstory pin <ref>` or `rstory unpin <ref>`: silent on success.
///
/// # Errors
/// Exit 3 without a daemon, 6 at the pin cap, 2 for an unknown reference.
pub fn pin(io: &mut Io<'_>, reference: &str, pin: bool) -> Result<(), CliError> {
    simple_call(io, if pin { "pin" } else { "unpin" }, reference)
}

/// `rstory rm <ref>`: silent on success; deleting one entry asks nothing.
///
/// # Errors
/// Exit 3 without a daemon, 2 for an unknown reference.
pub fn rm(io: &mut Io<'_>, reference: &str) -> Result<(), CliError> {
    simple_call(io, "rm", reference)
}

fn simple_call(io: &mut Io<'_>, op: &str, reference: &str) -> Result<(), CliError> {
    let mut client = source::require_daemon(io.env)?;
    client
        .call(op, json!({"ref": reference}))
        .map(|_| ())
        .map_err(|e| CliError::from_client(e, Some(reference)))
}

/// `rstory pause [--for <duration>]`: silent on success. The length was checked by the parser.
///
/// # Errors
/// Exit 3 without a daemon.
pub fn pause(io: &mut Io<'_>, length: Option<&str>) -> Result<(), CliError> {
    let mut client = source::require_daemon(io.env)?;
    let args = length.map_or_else(|| json!({}), |l| json!({"for": l}));
    client
        .call("pause", args)
        .map(|_| ())
        .map_err(|e| CliError::from_client(e, None))
}

/// `rstory resume`: silent on success.
///
/// # Errors
/// Exit 3 without a daemon.
pub fn resume(io: &mut Io<'_>) -> Result<(), CliError> {
    let mut client = source::require_daemon(io.env)?;
    client
        .call("resume", json!({}))
        .map(|_| ())
        .map_err(|e| CliError::from_client(e, None))
}

/// The question `clear` asks: `unpinned` and `pinned` are the current counts.
#[must_use]
pub fn clear_prompt(unpinned: u32, pinned: u32, all: bool) -> String {
    if all {
        copy::format(
            copy::CLI_CLEAR_ALL_PROMPT,
            &[
                ("n", &(unpinned + pinned).to_string()),
                ("m", &pinned.to_string()),
            ],
        )
    } else {
        copy::format(copy::CLI_CLEAR_PROMPT, &[("n", &unpinned.to_string())])
    }
}

/// Writes `prompt` and reads one line: `y` or `yes` (any case) confirm, anything else, EOF
/// included, declines.
pub fn confirm(prompt: &str, input: &mut dyn BufRead, err: &mut dyn Write) -> bool {
    let _ = write!(err, "{prompt} ");
    let _ = err.flush();
    let mut line = String::new();
    if input.read_line(&mut line).is_err() {
        return false;
    }
    matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// Asks on a terminal: the answer comes from stdin when that is a terminal, else from
/// `/dev/tty`. `None` when there is no terminal to ask on (stderr is not one, or `/dev/tty`
/// cannot be opened).
fn ask_on_terminal(prompt: &str, err: &mut dyn Write) -> Option<bool> {
    if !std::io::stderr().is_terminal() {
        return None;
    }
    if std::io::stdin().is_terminal() {
        return Some(confirm(prompt, &mut std::io::stdin().lock(), err));
    }
    let tty = std::fs::File::open("/dev/tty").ok()?;
    Some(confirm(prompt, &mut io::BufReader::new(tty), err))
}

/// `rstory clear [--all] [--yes]`: asks first on a terminal; prints `Removed N entries.` on stderr.
///
/// # Errors
/// Exit 3 without a daemon; exit 2 when it would have to ask and there is no terminal;
/// exit 1 when the answer is no.
pub fn clear(io: &mut Io<'_>, all: bool, yes: bool) -> Result<(), CliError> {
    let mut client = source::require_daemon(io.env)?;
    if !yes {
        let status = client
            .call("status", json!({}))
            .map_err(|e| CliError::from_client(e, None))?;
        let status: Status = serde_json::from_value(status).map_err(|_| {
            CliError::new(
                ExitCode::Unexpected,
                "The daemon sent a status rstory does not understand.",
            )
        })?;
        let unpinned = status.entries.saturating_sub(status.pinned);
        let nothing_to_delete = if all {
            status.entries == 0
        } else {
            unpinned == 0
        };
        if !nothing_to_delete {
            let prompt = clear_prompt(unpinned, status.pinned, all);
            match ask_on_terminal(&prompt, io.err) {
                None => return Err(CliError::usage(copy::CLI_NEEDS_YES)),
                Some(true) => {}
                Some(false) => return Err(CliError::new(ExitCode::Unexpected, "Cancelled.")),
            }
        }
    }
    let result = client
        .call("clear", json!({"all": all}))
        .map_err(|e| CliError::from_client(e, None))?;
    let removed = result.get("removed").and_then(Value::as_u64).unwrap_or(0);
    note(
        io.err,
        &copy::format(copy::CLI_CLEAR_DONE, &[("n", &removed.to_string())]),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer(input: &str) -> bool {
        let mut err = Vec::new();
        confirm("Delete? [y/N]", &mut input.as_bytes(), &mut err)
    }

    #[test]
    fn only_y_and_yes_confirm_in_any_case() {
        for yes in ["y\n", "Y\n", "yes\n", "YES\n", " y \n", "y", "Yes\r\n"] {
            assert!(answer(yes), "{yes:?}");
        }
        for no in ["", "\n", "n\n", "no\n", "maybe\n", "yy\n", "yes please\n"] {
            assert!(!answer(no), "{no:?}");
        }
    }

    #[test]
    fn the_prompt_is_written_before_reading_and_without_a_newline() {
        let mut err = Vec::new();
        confirm("Delete? [y/N]", &mut "n\n".as_bytes(), &mut err);
        assert_eq!(String::from_utf8(err).unwrap(), "Delete? [y/N] ");
    }

    #[test]
    fn clear_prompts_follow_the_copy_keys() {
        assert_eq!(
            clear_prompt(12, 3, false),
            "Delete 12 unpinned entries? This can't be undone. [y/N]"
        );
        assert_eq!(
            clear_prompt(12, 3, true),
            "Delete ALL 15 entries, including 3 pinned? This can't be undone. [y/N]"
        );
    }
}
