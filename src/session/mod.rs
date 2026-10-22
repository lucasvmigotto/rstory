//! Session detection and backend selection (BR5b, ADR-0005).
//!
//! `select` is a pure decision over the environment, an optional override and a [`Probe`] that
//! answers the two questions the environment cannot: which data-control global the Wayland
//! registry offers, and whether the X display accepts a connection. The probe is only asked what
//! the decision needs, so a plain X11 session never opens a Wayland connection.

use crate::backend::{BackendKind, Mode};

/// The environment variables that describe the session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionEnv {
    /// `WAYLAND_DISPLAY`: a socket name in `xdg_runtime_dir`, or an absolute path.
    pub wayland_display: Option<String>,
    /// `DISPLAY`.
    pub display: Option<String>,
    /// `XDG_SESSION_TYPE`.
    pub session_type: Option<String>,
    /// `XDG_RUNTIME_DIR`.
    pub xdg_runtime_dir: Option<String>,
}

/// Which data-control protocol the compositor offers (`ext` is preferred over `zwlr`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataControl {
    None,
    Ext,
    Zwlr,
}

/// Why a selection came out the way it did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Override,
    X11Session,
    DataControl,
    NoDataControl,
    NoDataControlNoXwayland,
    NoSession,
    X11Unreachable,
}

impl Reason {
    /// Stable name for `status` and `doctor`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Override => "override",
            Self::X11Session => "x11_session",
            Self::DataControl => "data_control",
            Self::NoDataControl => "no_data_control",
            Self::NoDataControlNoXwayland => "no_data_control_no_xwayland",
            Self::NoSession => "no_session",
            Self::X11Unreachable => "x11_unreachable",
        }
    }
}

/// The result of backend selection (FR-008).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendSelection {
    pub backend: BackendKind,
    pub mode: Mode,
    pub reason: Reason,
}

/// What the environment cannot tell.
pub trait Probe {
    /// Ask the Wayland registry for the data-control globals.
    fn data_control(&mut self) -> DataControl;
    /// Whether the X display accepts a connection (waits for on-demand XWayland).
    fn x11_reachable(&mut self) -> bool;
}

/// Pick the data-control protocol out of a registry's interface names.
#[must_use]
pub fn data_control_from(_interfaces: &[String]) -> DataControl {
    DataControl::None
}

/// Parse the `backend` config key / `--backend` flag; `auto` means no override.
#[must_use]
pub fn parse_override(_value: &str) -> Option<BackendKind> {
    None
}

/// Decide the backend. `override_backend` (config or `--backend`) wins over detection.
#[must_use]
pub fn select(
    _env: &SessionEnv,
    _probe: &mut dyn Probe,
    _override_backend: Option<BackendKind>,
) -> BackendSelection {
    BackendSelection {
        backend: BackendKind::None,
        mode: Mode::None,
        reason: Reason::NoSession,
    }
}

/// The real probe: a Wayland registry round trip and an X connection attempt.
pub struct RealProbe;

impl RealProbe {
    /// A probe for `env`; `xauthority` overrides `$XAUTHORITY`.
    #[must_use]
    pub fn new(_env: &SessionEnv, _xauthority: Option<std::path::PathBuf>) -> Self {
        Self
    }
}

impl Probe for RealProbe {
    fn data_control(&mut self) -> DataControl {
        DataControl::None
    }

    fn x11_reachable(&mut self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests;
