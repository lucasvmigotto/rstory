//! The checks: [`Facts`] (what the probes found) turned into [`Check`] results by the pure
//! [`evaluate`]. Nothing here touches the machine, so every row of the failure table runs on
//! a made-up `Facts` (spec 011, SC-001).

use std::path::PathBuf;

use time::OffsetDateTime;

use super::{Check, Status};
use crate::backend::{BackendKind, Mode};
use crate::domain::entry::PauseState;
use crate::install::autostart::AutostartKind;
use crate::install::gnome::ExtensionInstall;
use crate::install::gnome::UUID;
use crate::ipc::protocol::{StateKind, Status as DaemonStatus};
use crate::launch::{LaunchPlan, Source};
use crate::session::{BackendSelection, Reason};

/// The GNOME Shell majors the panel icon supports.
pub const SHELL_SUPPORTED: std::ops::RangeInclusive<u32> = 48..=51;

/// What kind of graphical session the environment describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    Wayland,
    X11,
    None,
}

/// The session and the backend rstory would pick in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionFacts {
    pub kind: SessionKind,
    pub selection: BackendSelection,
}

/// The store directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreFact {
    /// No `HOME` or `XDG_STATE_HOME`, so no directory to look at.
    Unknown,
    /// The directory does not exist yet.
    Missing(PathBuf),
    /// Permissions are 0700/0600; `entries` when it could be counted.
    Private { entries: Option<usize> },
    /// A file or directory allows group or other access.
    TooOpen { dir: PathBuf },
    /// The directory could not be read.
    Unreadable { dir: PathBuf },
}

/// The daemon behind the control socket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonFact {
    /// Nothing listens.
    Down,
    /// Something is there but did not give a `status` answer in time.
    NotAnswering,
    /// The `status` result.
    Up(Box<DaemonStatus>),
}

/// What is known when the desktop is GNOME.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GnomeFacts {
    /// `48.7` from `gnome-shell --version`; `None` when the program is missing or too slow.
    pub shell_version: Option<String>,
    /// `None` when the extension directory could not be located.
    pub extension: Option<ExtensionInstall>,
    /// The version this binary ships.
    pub shipped_version: Option<String>,
}

/// Everything doctor found out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub now: OffsetDateTime,
    pub session: SessionFacts,
    pub store: StoreFact,
    pub daemon: DaemonFact,
    /// The pause persisted in the store, read only when no daemon answers.
    pub store_pause: Option<PauseState>,
    pub gnome: Option<GnomeFacts>,
    pub picker: LaunchPlan,
    pub autostart: Vec<(AutostartKind, PathBuf)>,
}

/// Turns the facts into the report, in display order.
#[must_use]
pub fn evaluate(facts: &Facts) -> Vec<Check> {
    let mut out = vec![session(facts), backend(facts), store(facts), daemon(facts)];
    out.extend(pause(facts));
    out.extend(tray(facts));
    if let Some(g) = &facts.gnome {
        out.extend(shell(g));
        out.push(extension(g));
    }
    out.push(picker(&facts.picker));
    out.push(autostart(&facts.autostart));
    out
}

fn session(f: &Facts) -> Check {
    let version = f
        .gnome
        .as_ref()
        .and_then(|g| g.shell_version.as_deref())
        .map(|v| format!(" (GNOME Shell {v})"))
        .unwrap_or_default();
    match f.session.kind {
        SessionKind::Wayland => Check::ok("session", format!("Wayland{version}")),
        SessionKind::X11 => Check::ok("session", format!("X11{version}")),
        SessionKind::None => Check::with_hint(
            "session",
            Status::Fail,
            "no graphical session: no Wayland or X display was found",
            "run rstory from a terminal inside your desktop session",
        ),
    }
}

fn backend(f: &Facts) -> Check {
    let sel = f.session.selection;
    let how = if sel.reason == Reason::Override {
        " (chosen in the settings)"
    } else {
        ""
    };
    match (sel.backend, sel.mode) {
        (BackendKind::Wayland, _) => Check::ok("backend", format!("wayland (data-control){how}")),
        (BackendKind::X11, Mode::XwaylandBridge) => {
            let why = if sel.reason == Reason::NoDataControl {
                " (no data-control on this compositor)"
            } else {
                how
            };
            Check::ok("backend", format!("x11 via XWayland bridge{why}"))
        }
        (BackendKind::X11, _) => Check::ok("backend", format!("x11{how}")),
        (BackendKind::None, _) => match sel.reason {
            Reason::NoDataControlNoXwayland => Check::with_hint(
                "backend",
                Status::Fail,
                "This compositor offers no clipboard-monitoring protocol and no XWayland, \
                 so rstory cannot see what you copy.",
                "enable XWayland in your compositor, or use a compositor with data-control",
            ),
            Reason::X11Unreachable => Check::with_hint(
                "backend",
                Status::Fail,
                "The X display does not accept connections, so rstory cannot see what you copy.",
                "check that DISPLAY and XAUTHORITY point at a running X server or XWayland",
            ),
            _ => Check::with_hint(
                "backend",
                Status::Fail,
                "No clipboard access: there is no graphical session to watch.",
                "run rstory from a terminal inside your desktop session",
            ),
        },
    }
}

fn store(f: &Facts) -> Check {
    match &f.store {
        StoreFact::Unknown => Check::with_hint(
            "store",
            Status::Fail,
            "the store location is unknown: neither XDG_STATE_HOME nor HOME is set",
            "set HOME (or XDG_STATE_HOME) before running rstory",
        ),
        StoreFact::Missing(_) => {
            Check::ok("store", "no history yet (created private on first copy)")
        }
        StoreFact::Private { entries } => {
            let n = entries.map_or(String::new(), |n| {
                format!(", {n} {}", if n == 1 { "entry" } else { "entries" })
            });
            Check::ok("store", format!("private (0700/0600){n}"))
        }
        StoreFact::TooOpen { dir } => {
            let d = dir.display();
            Check::with_hint(
                "store",
                Status::Fail,
                format!("permissions are too open: other users could read your history in {d}"),
                format!("chmod 700 {d} && chmod 600 {d}/*"),
            )
        }
        StoreFact::Unreadable { dir } => Check::with_hint(
            "store",
            Status::Fail,
            format!("cannot read the store in {}", dir.display()),
            "check that the directory exists and belongs to you",
        ),
    }
}

fn daemon(f: &Facts) -> Check {
    match &f.daemon {
        DaemonFact::Down => Check::with_hint(
            "daemon",
            Status::Fail,
            "not running",
            "run `rstory daemon` or `rstory install-autostart`",
        ),
        DaemonFact::NotAnswering => Check::with_hint(
            "daemon",
            Status::Fail,
            "not answering on its socket",
            "stop it (`pkill -x rstory`) and run `rstory daemon` again",
        ),
        DaemonFact::Up(s) => {
            let c = &s.counters;
            let skipped: Vec<String> = [
                (c.skipped_sensitive, "sensitive"),
                (c.skipped_oversize, "oversize"),
                (c.skipped_paused, "while paused"),
                (c.dropped_burst, "dropped in bursts"),
            ]
            .iter()
            .filter(|(n, _)| *n > 0)
            .map(|(n, what)| format!("{n} {what}"))
            .collect();
            let skipped = if skipped.is_empty() {
                String::new()
            } else {
                format!(" (skipped: {})", skipped.join(", "))
            };
            match s.state.state {
                StateKind::Recording => Check::ok("daemon", format!("running, recording{skipped}")),
                StateKind::Paused => Check::ok("daemon", format!("running, paused{skipped}")),
                StateKind::Inactive => Check::with_hint(
                    "daemon",
                    Status::Warn,
                    format!(
                        "running, but not recording: {}",
                        s.state.reason.as_deref().unwrap_or("unknown reason")
                    ),
                    "fix the Backend line above, then restart `rstory daemon`",
                ),
            }
        }
    }
}

fn pause(f: &Facts) -> Option<Check> {
    let until = match &f.daemon {
        DaemonFact::Up(s) if s.state.state == StateKind::Paused => Some(s.state.paused_until),
        DaemonFact::Up(_) => None,
        _ => match f.store_pause {
            Some(PauseState::Paused { until }) => Some(until),
            _ => None,
        },
    }?;
    let message = match until {
        Some(end) if end <= f.now => return None,
        Some(end) => {
            let secs = (end - f.now).whole_seconds();
            let (m, s) = (secs / 60, secs % 60);
            let left = match (m, s) {
                (0, s) => format!("{s}s"),
                (m, 0) => format!("{m}m"),
                (m, s) => format!("{m}m {s}s"),
            };
            format!("recording paused (resumes in {left})")
        }
        None => "recording paused (until you resume it)".to_string(),
    };
    Some(Check::with_hint(
        "pause",
        Status::Info,
        message,
        "run `rstory resume` to record again",
    ))
}

fn tray(f: &Facts) -> Option<Check> {
    // On GNOME the panel icon comes from the extension, not from a tray host.
    let DaemonFact::Up(s) = &f.daemon else {
        return None;
    };
    let on_gnome = f.gnome.is_some();
    Some(match s.tray.as_str() {
        "active" => Check::ok("tray", "tray icon shown"),
        _ if on_gnome => return None,
        "no_host" => Check::with_hint(
            "tray",
            Status::Warn,
            "no StatusNotifierItem host: the tray is unavailable; CLI and picker work",
            "start a panel or tray that supports StatusNotifierItem",
        ),
        "no_bus" => Check::with_hint(
            "tray",
            Status::Warn,
            "no session D-Bus: the tray is unavailable; CLI and picker work",
            "start rstory from inside a desktop session that provides D-Bus",
        ),
        "disabled" => Check::with_hint(
            "tray",
            Status::Info,
            "tray icon turned off in config.toml",
            "set tray = true in config.toml to show it",
        ),
        other => Check::with_hint(
            "tray",
            Status::Info,
            format!("tray state `{other}`"),
            "update rstory so the daemon and this command agree",
        ),
    })
}

fn shell(g: &GnomeFacts) -> Option<Check> {
    let version = g.shell_version.as_deref()?;
    let major: u32 = version.split('.').next()?.parse().ok()?;
    if SHELL_SUPPORTED.contains(&major) {
        return None;
    }
    Some(Check::with_hint(
        "gnome_shell",
        Status::Warn,
        format!("panel icon not supported on GNOME Shell {major}; CLI and picker work"),
        "the panel icon needs GNOME Shell 48 to 51",
    ))
}

fn extension(g: &GnomeFacts) -> Check {
    let Some(ext) = &g.extension else {
        return Check::with_hint(
            "gnome_extension",
            Status::Info,
            "cannot tell: the data directory is unknown",
            "set HOME (or XDG_DATA_HOME) and run `rstory doctor` again",
        );
    };
    let install = "run `rstory install-gnome-extension`, then log out and back in";
    let Some(installed) = &ext.installed_version else {
        return Check::with_hint("gnome_extension", Status::Warn, "not installed", install);
    };
    if let Some(shipped) = g.shipped_version.as_ref().filter(|s| *s != installed) {
        return Check::with_hint(
            "gnome_extension",
            Status::Warn,
            format!("installed {installed}, this rstory ships {shipped}"),
            install,
        );
    }
    match ext.enabled {
        Some(true) => Check::ok("gnome_extension", format!("installed {installed}, enabled")),
        Some(false) => Check::with_hint(
            "gnome_extension",
            Status::Warn,
            format!("installed {installed}, not enabled"),
            format!("gnome-extensions enable {UUID}"),
        ),
        None => Check::with_hint(
            "gnome_extension",
            Status::Info,
            format!("installed {installed}, enabled state unknown"),
            format!("run `gnome-extensions info {UUID}` to see it"),
        ),
    }
}

fn picker(plan: &LaunchPlan) -> Check {
    match plan {
        LaunchPlan::Found { source, argv } => {
            let program = argv.first().map_or("", String::as_str);
            let name = program.rsplit('/').next().unwrap_or(program);
            if *source == Source::Config {
                Check::ok("picker", format!("{name} (picker_command in config.toml)"))
            } else {
                Check::ok("picker", name)
            }
        }
        LaunchPlan::NotFound { searched } => Check::with_hint(
            "picker",
            Status::Warn,
            format!(
                "no terminal found for the picker (searched: {})",
                searched.join(", ")
            ),
            "set picker_command in config.toml, for example picker_command = \"xterm -e rstory pick\"",
        ),
    }
}

fn autostart(found: &[(AutostartKind, PathBuf)]) -> Check {
    if found.is_empty() {
        return Check::with_hint(
            "autostart",
            Status::Info,
            "rstory does not start at login",
            "run `rstory install-autostart` to start it at login",
        );
    }
    let parts: Vec<String> = found
        .iter()
        .map(|(kind, path)| match kind {
            AutostartKind::Xdg => path.display().to_string(),
            AutostartKind::Systemd => format!("systemd unit {}", path.display()),
        })
        .collect();
    Check::ok("autostart", parts.join(", "))
}
