//! Status-notifier tray icon and menu (spec 009).
//!
//! The tray lives in the daemon process and talks to the daemon through the same [`Link`] the
//! control socket uses, so it is a client and never writes the store. Nothing here logs or
//! formats clipboard content (Constitution III).

pub mod icons;
pub mod menu;

use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::ipc::protocol::Command;
use crate::ipc::server::Link;
use menu::Action;

/// `status.tray`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayStatus {
    /// Registered with a watcher.
    Active,
    /// A session bus, but no `StatusNotifierWatcher` on it.
    NoHost,
    /// No session bus.
    NoBus,
    /// Turned off (`tray = false`) or not started yet.
    Disabled,
}

impl TrayStatus {
    /// The wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::NoHost => "no_host",
            Self::NoBus => "no_bus",
            Self::Disabled => "disabled",
        }
    }
}

/// The protocol request an activated item sends, `None` for the steps that stay in the tray.
#[must_use]
pub fn command_for(_action: &Action) -> Option<Command> {
    todo!()
}

/// Starts the tray on the current runtime. The receiver follows `status.tray`; the task ends
/// when `shutdown` turns true.
#[must_use]
pub fn spawn(_link: Link, _shutdown: watch::Receiver<bool>) -> (watch::Receiver<TrayStatus>, JoinHandle<()>) {
    todo!()
}

#[cfg(test)]
mod tests;
