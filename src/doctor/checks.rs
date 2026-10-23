//! The checks: [`Facts`] (what the probes found) turned into [`Check`] results by the pure
//! [`evaluate`]. Nothing here touches the machine, so every row of the failure table runs on
//! a made-up `Facts` (spec 011, SC-001).

use std::path::PathBuf;

use time::OffsetDateTime;

use super::Check;
use crate::domain::entry::PauseState;
use crate::install::autostart::AutostartKind;
use crate::install::gnome::ExtensionInstall;
use crate::ipc::protocol::Status as DaemonStatus;
use crate::launch::LaunchPlan;
use crate::session::BackendSelection;

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
    let _ = facts;
    todo!()
}
