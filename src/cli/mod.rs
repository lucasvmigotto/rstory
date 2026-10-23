//! Command-line interface: clap commands, text and JSON output, exit codes.
//!
//! `run` is the whole CLI: it parses the arguments and dispatches. `src/main.rs` calls it for
//! everything except `daemon`, which it starts itself (architecture 5.3: cli does not use daemon).
//! To add a subcommand: declare it in `args::Command` and add one arm to `dispatch`.

pub mod args;
pub mod commands;
pub mod env;
pub mod error;
pub mod render;
pub mod source;
pub mod status;

use clap::Parser;

use crate::ipc::error::ErrorCode;

pub use crate::domain::ids::EntryRef;

/// Process exit codes (`contracts/cli.md`, stable within a major version).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// 0: done.
    Ok,
    /// 1: unexpected error (the picker also uses it for "dismissed").
    Unexpected,
    /// 2: usage error, entry not found or ambiguous, no terminal where one is needed.
    Usage,
    /// 3: the daemon is not running.
    DaemonDown,
    /// 4: a daemon is already running.
    AlreadyRunning,
    /// 5: environment error (runtime directory, store, config).
    Environment,
    /// 6: a limit was reached (the pin cap).
    LimitReached,
    /// 7: inactive, there is no capture backend.
    Inactive,
    /// 130: interrupted.
    Interrupted,
}

impl ExitCode {
    /// The number the process exits with.
    #[must_use]
    pub fn code(self) -> u8 {
        match self {
            Self::Ok => 0,
            Self::Unexpected => 1,
            Self::Usage => 2,
            Self::DaemonDown => 3,
            Self::AlreadyRunning => 4,
            Self::Environment => 5,
            Self::LimitReached => 6,
            Self::Inactive => 7,
            Self::Interrupted => 130,
        }
    }

    /// The exit code the error catalog assigns to a protocol error; 1 where it assigns none.
    #[must_use]
    pub fn from_proto(code: ErrorCode) -> Self {
        match code.cli_exit() {
            Some(2) => Self::Usage,
            Some(6) => Self::LimitReached,
            Some(7) => Self::Inactive,
            _ => Self::Unexpected,
        }
    }
}

/// Parses `args` (without the program name), runs the command and returns the exit status.
#[must_use]
pub fn run(args: &[String]) -> u8 {
    let argv = std::iter::once("rstory".to_string()).chain(args.iter().cloned());
    let cli = match args::Cli::try_parse_from(argv) {
        Ok(cli) => cli,
        Err(e) => {
            let code = e.exit_code();
            let _ = e.print();
            return u8::try_from(code).unwrap_or(2);
        }
    };
    // Owned by spec 012: it prints its own lines and picks its own exit code.
    if let args::Command::InstallAutostart { args } = &cli.command {
        return crate::install::autostart::run(args);
    }
    let env = env::Env::process();
    let term = render::Term::detect();
    let (mut out, mut err) = (std::io::stdout().lock(), std::io::stderr().lock());
    let mut io = commands::Io {
        env: &env,
        term: &term,
        out: &mut out,
        err: &mut err,
    };
    match dispatch(cli.command, &mut io) {
        Ok(()) => ExitCode::Ok.code(),
        Err(e) => {
            let _ = writeln!(io.err, "{}", e.message);
            e.exit.code()
        }
    }
}

/// Runs one parsed command. To add a subcommand, add an arm here.
fn dispatch(command: args::Command, io: &mut commands::Io<'_>) -> Result<(), error::CliError> {
    use args::Command;
    match command {
        Command::Ls { json, limit } => commands::ls(io, json, limit),
        Command::Show { entry, json } => commands::show(io, &entry.entry, json),
        Command::Copy { entry } => commands::copy_entry(io, &entry.entry),
        Command::Pin { entry } => commands::pin(io, &entry.entry, true),
        Command::Unpin { entry } => commands::pin(io, &entry.entry, false),
        Command::Rm { entry } => commands::rm(io, &entry.entry),
        Command::Clear { all, yes } => commands::clear(io, all, yes),
        Command::Pause { length } => commands::pause(io, length.as_deref()),
        Command::Resume => commands::resume(io),
        Command::Status { json } => commands::status(io, json),
        Command::Config { path } => commands::config(io, path),
        Command::Completions { shell } => {
            args::completions(shell, io.out);
            Ok(())
        }
        // `src/main.rs` runs the daemon itself: the cli module may not depend on it.
        Command::Daemon { .. } => Err(unavailable("daemon")),
        Command::Pick { .. } => Err(unavailable("pick")),
        Command::Doctor { .. } => Err(unavailable("doctor")),
        Command::InstallAutostart { .. } => unreachable!("handled in `run`"),
        Command::InstallGnomeExtension { .. } => Err(unavailable("install-gnome-extension")),
    }
}

/// A command another spec has not delivered yet.
fn unavailable(name: &str) -> error::CliError {
    error::CliError::new(
        ExitCode::Unexpected,
        format!("rstory: {name} is not available in this build"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_match_the_contract() {
        let table = [
            (ExitCode::Ok, 0),
            (ExitCode::Unexpected, 1),
            (ExitCode::Usage, 2),
            (ExitCode::DaemonDown, 3),
            (ExitCode::AlreadyRunning, 4),
            (ExitCode::Environment, 5),
            (ExitCode::LimitReached, 6),
            (ExitCode::Inactive, 7),
            (ExitCode::Interrupted, 130),
        ];
        for (code, n) in table {
            assert_eq!(code.code(), n);
        }
    }

    #[test]
    fn protocol_errors_map_to_the_catalog_exit_codes() {
        use crate::ipc::error::ErrorCode as E;
        assert_eq!(ExitCode::from_proto(E::NotFound), ExitCode::Usage);
        assert_eq!(ExitCode::from_proto(E::Ambiguous), ExitCode::Usage);
        assert_eq!(ExitCode::from_proto(E::BadRequest), ExitCode::Usage);
        assert_eq!(ExitCode::from_proto(E::PinLimit), ExitCode::LimitReached);
        assert_eq!(ExitCode::from_proto(E::Inactive), ExitCode::Inactive);
        assert_eq!(ExitCode::from_proto(E::Internal), ExitCode::Unexpected);
        assert_eq!(ExitCode::from_proto(E::BackendError), ExitCode::Unexpected);
        assert_eq!(ExitCode::from_proto(E::NoTerminal), ExitCode::Unexpected);
    }

    #[test]
    fn a_ref_is_an_index_or_a_short_id() {
        assert_eq!("0".parse::<EntryRef>(), Ok(EntryRef::Index(0)));
        assert_eq!("12".parse::<EntryRef>(), Ok(EntryRef::Index(12)));
        assert_eq!(
            "01jb".parse::<EntryRef>(),
            Ok(EntryRef::ShortId("01JB".into()))
        );
        assert_eq!(
            "ab12".parse::<EntryRef>(),
            Ok(EntryRef::ShortId("AB12".into()))
        );
    }

    #[test]
    fn bad_refs_are_rejected() {
        for bad in ["", "abc", "-1", "ab 12", "ab!2", "99999999999"] {
            assert!(bad.parse::<EntryRef>().is_err(), "{bad:?}");
        }
    }
}
