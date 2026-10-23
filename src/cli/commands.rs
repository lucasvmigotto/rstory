//! The commands of `contracts/cli.md` that talk to the daemon or the store.
//!
//! Each command writes machine output to `out` and diagnostics to `err`, and returns an
//! error carrying its exit code. Nothing here logs or prints clipboard content except `show`.

use std::io::{self, Write};

use serde_json::json;
use time::OffsetDateTime;

use super::ExitCode;
use super::env::Env;
use super::error::CliError;
use super::render::{self, Term};
use super::source;
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
