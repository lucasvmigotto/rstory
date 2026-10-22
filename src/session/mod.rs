//! Session detection and backend selection (BR5b, ADR-0005).
//!
//! `select` is a pure decision over the environment, an optional override and a [`Probe`] that
//! answers the two questions the environment cannot: which data-control global the Wayland
//! registry offers, and whether the X display accepts a connection. The probe is only asked what
//! the decision needs, so a plain X11 session never opens a Wayland connection.

use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, QueueHandle};

use crate::backend::x11::X11Options;
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
pub fn data_control_from(interfaces: &[String]) -> DataControl {
    let has = |name: &str| interfaces.iter().any(|i| i == name);
    if has("ext_data_control_manager_v1") {
        DataControl::Ext
    } else if has("zwlr_data_control_manager_v1") {
        DataControl::Zwlr
    } else {
        DataControl::None
    }
}

/// Parse the `backend` config key / `--backend` flag; `auto` (or anything else) means no override.
#[must_use]
pub fn parse_override(value: &str) -> Option<BackendKind> {
    match value {
        "x11" => Some(BackendKind::X11),
        "wayland" => Some(BackendKind::Wayland),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Wayland,
    X11,
    Unknown,
}

fn set(v: &Option<String>) -> Option<&str> {
    v.as_deref().filter(|s| !s.is_empty())
}

fn kind_of(env: &SessionEnv) -> Kind {
    match set(&env.session_type)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("wayland") => Kind::Wayland,
        Some("x11") => Kind::X11,
        _ if set(&env.wayland_display).is_some() => Kind::Wayland,
        _ if set(&env.display).is_some() => Kind::X11,
        _ => Kind::Unknown,
    }
}

/// Decide the backend. `override_backend` (config or `--backend`) wins over detection.
#[must_use]
pub fn select(
    env: &SessionEnv,
    probe: &mut dyn Probe,
    override_backend: Option<BackendKind>,
) -> BackendSelection {
    let kind = kind_of(env);
    let sel = |backend, mode, reason| BackendSelection {
        backend,
        mode,
        reason,
    };
    match override_backend {
        Some(BackendKind::X11) => {
            let mode = if kind == Kind::Wayland {
                Mode::XwaylandBridge
            } else {
                Mode::Native
            };
            return sel(BackendKind::X11, mode, Reason::Override);
        }
        Some(BackendKind::Wayland) => {
            return sel(BackendKind::Wayland, Mode::Native, Reason::Override);
        }
        Some(BackendKind::None) | None => {}
    }
    let none = |reason| sel(BackendKind::None, Mode::None, reason);
    let has_display = set(&env.display).is_some();
    match kind {
        Kind::Unknown => none(Reason::NoSession),
        Kind::X11 if !has_display => none(Reason::NoSession),
        Kind::X11 if probe.x11_reachable() => {
            sel(BackendKind::X11, Mode::Native, Reason::X11Session)
        }
        Kind::X11 => none(Reason::X11Unreachable),
        Kind::Wayland => {
            let has_wayland = set(&env.wayland_display).is_some();
            if has_wayland && probe.data_control() != DataControl::None {
                return sel(BackendKind::Wayland, Mode::Native, Reason::DataControl);
            }
            if !has_display {
                return none(if has_wayland {
                    Reason::NoDataControlNoXwayland
                } else {
                    Reason::NoSession
                });
            }
            if probe.x11_reachable() {
                sel(
                    BackendKind::X11,
                    Mode::XwaylandBridge,
                    Reason::NoDataControl,
                )
            } else {
                none(Reason::X11Unreachable)
            }
        }
    }
}

/// The real probe: a Wayland registry round trip and an X connection attempt.
pub struct RealProbe {
    wayland_socket: Option<PathBuf>,
    x11: X11Options,
}

impl RealProbe {
    /// A probe for `env`; `xauthority` overrides `$XAUTHORITY`. On a Wayland session the X
    /// connection is retried for up to 3 s, because XWayland starts on demand.
    #[must_use]
    pub fn new(env: &SessionEnv, xauthority: Option<PathBuf>) -> Self {
        let wayland_socket = set(&env.wayland_display).map(|name| {
            let p = Path::new(name);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                Path::new(set(&env.xdg_runtime_dir).unwrap_or("/run/user/1000")).join(name)
            }
        });
        let mut x11 = X11Options::new(set(&env.display).unwrap_or(""));
        x11.xauthority = xauthority;
        if kind_of(env) == Kind::Wayland {
            x11.mode = Mode::XwaylandBridge;
        }
        Self {
            wayland_socket,
            x11,
        }
    }
}

#[derive(Default)]
struct Registry {
    interfaces: Vec<String>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for Registry {
    fn event(
        state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { interface, .. } = event {
            state.interfaces.push(interface);
        }
    }
}

/// Interface names the compositor at `socket` advertises.
fn registry_interfaces(socket: &Path) -> Option<Vec<String>> {
    let stream = UnixStream::connect(socket).ok()?;
    let conn = Connection::from_socket(stream).ok()?;
    let mut queue = conn.new_event_queue::<Registry>();
    let _registry = conn.display().get_registry(&queue.handle(), ());
    let mut state = Registry::default();
    queue.roundtrip(&mut state).ok()?;
    Some(state.interfaces)
}

impl Probe for RealProbe {
    fn data_control(&mut self) -> DataControl {
        let Some(socket) = self.wayland_socket.clone() else {
            return DataControl::None;
        };
        // A wedged compositor must not wedge startup: give up after 3 s.
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(registry_interfaces(&socket));
        });
        match rx.recv_timeout(Duration::from_secs(3)) {
            Ok(Some(interfaces)) => data_control_from(&interfaces),
            _ => DataControl::None,
        }
    }

    fn x11_reachable(&mut self) -> bool {
        crate::backend::x11::reachable(&self.x11)
    }
}

#[cfg(test)]
mod tests;
