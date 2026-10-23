//! The clap command tree: every subcommand of `contracts/cli.md`, with help and examples.
//!
//! Commands owned by other specs (`daemon`, `pick`, `doctor`, `install-autostart`,
//! `install-gnome-extension`) are declared here so `--help` and completions list them; their
//! arguments are passed through untouched to their owner (see `cli::dispatch`).

use std::io::Write;
use std::str::FromStr;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};

use crate::ipc::protocol::{PAUSE_RANGE, parse_duration};

use super::EntryRef;

/// `rstory`: a small, private clipboard history.
#[derive(Debug, Parser)]
#[command(
    name = "rstory",
    version,
    about = "A small, private clipboard history for Linux",
    arg_required_else_help = true,
    propagate_version = true,
    after_long_help = "Entries are addressed by index (0 is the newest) or by a short id of at least 4 characters.\n\nExit codes: 0 ok, 1 unexpected, 2 usage or not found, 3 daemon not running, 4 daemon already running, 5 environment error, 6 limit reached, 7 inactive."
)]
pub struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub command: Command,
}

/// An entry reference: an index or a short id, checked while parsing.
fn entry_ref(text: &str) -> Result<String, String> {
    EntryRef::from_str(text)
        .map(|_| text.to_string())
        .map_err(|_| "not an index (0, 1, ...) or a short id of at least 4 characters".to_string())
}

/// A pause length (`<n>s|m|h`, 1 second to 24 hours), checked while parsing.
fn pause_length(text: &str) -> Result<String, String> {
    let d =
        parse_duration(text).ok_or_else(|| "use a number and a unit: 50s, 2m or 1h".to_string())?;
    if d < PAUSE_RANGE.0 || d > PAUSE_RANGE.1 {
        return Err("a pause lasts from 1s to 24h".to_string());
    }
    Ok(text.to_string())
}

/// The reference argument shared by the per-entry commands.
#[derive(Debug, Args)]
pub struct RefArg {
    /// Index (0 = newest) or short id (4+ characters)
    #[arg(value_name = "REF", value_parser = entry_ref)]
    pub entry: String,
}

/// Every subcommand.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the clipboard daemon in the foreground
    Daemon {
        /// Passed to the daemon (`--backend x11|wayland`)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// List the history, newest first
    #[command(
        after_long_help = "Examples:\n  rstory ls\n  rstory ls --limit 5\n  rstory ls --json | jq '.entries[].short_id'"
    )]
    Ls {
        /// Print JSON (`schema` 1) instead of lines
        #[arg(long)]
        json: bool,
        /// Show at most N entries
        #[arg(long, value_name = "N", value_parser = clap::value_parser!(u32).range(1..=500))]
        limit: Option<u32>,
    },
    /// Print an entry's content, byte for byte
    #[command(
        after_long_help = "Examples:\n  rstory show 0\n  rstory show 01JB | wc -c\n  rstory show 1 --json"
    )]
    Show {
        #[command(flatten)]
        entry: RefArg,
        /// Print JSON (`schema` 1) instead of the raw content
        #[arg(long)]
        json: bool,
    },
    /// Put an entry back on the clipboard
    #[command(after_long_help = "Examples:\n  rstory copy 1\n  rstory copy 01JBD8X")]
    Copy {
        #[command(flatten)]
        entry: RefArg,
    },
    /// Pin an entry so it survives clearing and expiry
    #[command(after_long_help = "Examples:\n  rstory pin 0")]
    Pin {
        #[command(flatten)]
        entry: RefArg,
    },
    /// Unpin an entry
    #[command(after_long_help = "Examples:\n  rstory unpin 01JBD8X")]
    Unpin {
        #[command(flatten)]
        entry: RefArg,
    },
    /// Delete one entry
    #[command(after_long_help = "Examples:\n  rstory rm 2")]
    Rm {
        #[command(flatten)]
        entry: RefArg,
    },
    /// Delete the unpinned entries (all of them with --all)
    #[command(
        after_long_help = "Examples:\n  rstory clear\n  rstory clear --yes\n  rstory clear --all"
    )]
    Clear {
        /// Also delete pinned entries
        #[arg(long)]
        all: bool,
        /// Do not ask for confirmation (required without a terminal)
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Stop recording, for a while or until `resume`
    #[command(
        after_long_help = "Examples:\n  rstory pause\n  rstory pause --for 50s\n  rstory pause --for 2m && pass -c mysite"
    )]
    Pause {
        /// How long: 50s, 2m or 1h (1s to 24h)
        #[arg(long = "for", value_name = "DURATION", value_parser = pause_length)]
        length: Option<String>,
    },
    /// Start recording again
    #[command(after_long_help = "Examples:\n  rstory resume")]
    Resume,
    /// Show the daemon state and counters
    #[command(after_long_help = "Examples:\n  rstory status\n  rstory status --json | jq .entries")]
    Status {
        /// Print JSON (`schema` 1)
        #[arg(long)]
        json: bool,
    },
    /// Show the effective configuration
    #[command(after_long_help = "Examples:\n  rstory config\n  rstory config --path")]
    Config {
        /// Print only the path of the config file
        #[arg(long)]
        path: bool,
    },
    /// Open the interactive picker
    #[command(disable_help_flag = true)]
    Pick {
        /// Passed to the picker (`--plain`)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// Check the setup and explain what is wrong
    #[command(disable_help_flag = true)]
    Doctor {
        /// Passed to doctor (`--json`)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// Show the tray icon from a separate process (for `tray = false`)
    Tray,
    /// Start the daemon at login
    #[command(name = "install-autostart", disable_help_flag = true)]
    InstallAutostart {
        /// Passed to the installer (`--systemd --now --uninstall`)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// Install the GNOME Shell extension
    #[command(name = "install-gnome-extension", disable_help_flag = true)]
    InstallGnomeExtension {
        /// Passed to the installer (`--enable --uninstall`)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// Print a shell completion script
    #[command(
        after_long_help = "Examples:\n  rstory completions bash > ~/.local/share/bash-completion/completions/rstory\n  rstory completions fish > ~/.config/fish/completions/rstory.fish"
    )]
    Completions {
        /// The shell
        shell: ShellArg,
    },
}

/// The shells completions are generated for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ShellArg {
    /// bash
    Bash,
    /// zsh
    Zsh,
    /// fish
    Fish,
}

impl FromStr for ShellArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        <Self as ValueEnum>::from_str(s, true)
    }
}

/// Writes the completion script of `shell` for the whole command tree.
pub fn completions(shell: ShellArg, out: &mut dyn Write) {
    use clap_complete::Shell;
    let shell = match shell {
        ShellArg::Bash => Shell::Bash,
        ShellArg::Zsh => Shell::Zsh,
        ShellArg::Fish => Shell::Fish,
    };
    clap_complete::generate(shell, &mut Cli::command(), "rstory", out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("rstory").chain(args.iter().copied()))
    }

    #[test]
    fn every_command_of_the_contract_parses() {
        for args in [
            &["daemon"][..],
            &["daemon", "--backend", "x11"],
            &["ls"],
            &["ls", "--json", "--limit", "5"],
            &["show", "0"],
            &["show", "ab12", "--json"],
            &["copy", "1"],
            &["pin", "1"],
            &["unpin", "1"],
            &["rm", "1"],
            &["clear"],
            &["clear", "--all", "--yes"],
            &["pause"],
            &["pause", "--for", "50s"],
            &["resume"],
            &["status", "--json"],
            &["config"],
            &["config", "--path"],
            &["pick", "--plain"],
            &["doctor", "--json"],
            &["install-autostart", "--systemd", "--now"],
            &["install-gnome-extension", "--enable"],
            &["completions", "bash"],
            &["completions", "zsh"],
            &["completions", "fish"],
        ] {
            assert!(parse(args).is_ok(), "{args:?}");
        }
    }

    #[test]
    fn bad_input_is_a_usage_error() {
        for args in [
            &["show"][..],
            &["show", "abc"],
            &["pause", "--for", "soon"],
            &["pause", "--for", "0s"],
            &["pause", "--for", "25h"],
            &["ls", "--limit", "0"],
            &["completions", "powershell"],
            &["nope"],
        ] {
            let err = parse(args).expect_err(&format!("{args:?}"));
            assert_eq!(err.exit_code(), 2, "{args:?}");
        }
    }

    #[test]
    fn every_command_has_help_with_an_example() {
        use clap::CommandFactory;
        let cmd = Cli::command();
        for sub in cmd.get_subcommands() {
            let name = sub.get_name();
            if [
                "daemon",
                "pick",
                "doctor",
                "install-autostart",
                "install-gnome-extension",
            ]
            .contains(&name)
            {
                continue; // owned by other specs
            }
            let help = sub.clone().render_long_help().to_string();
            assert!(help.contains("rstory "), "{name} has no example");
            assert!(sub.get_about().is_some(), "{name} has no about");
        }
    }

    #[test]
    fn completions_scripts_are_generated() {
        for shell in ["bash", "zsh", "fish"] {
            let mut out = Vec::new();
            completions(shell.parse().unwrap(), &mut out);
            let text = String::from_utf8(out).unwrap();
            assert!(text.contains("rstory"), "{shell}");
            assert!(text.contains("clear"), "{shell}");
        }
    }
}
