//! The probes: the only doctor code that looks at the machine. Every input is a field of
//! [`ProbeEnv`] (environment values, directories, the `PATH` string, the socket path, the
//! `gnome-shell` and `gnome-extensions` programs), so tests point it at temp dirs, fakes and a
//! stub socket server. Probes only read: nothing is written, enabled or started. External
//! programs get `ProbeEnv::timeout` each (backend.md: 300 ms).

use std::path::{Path, PathBuf};
use std::time::Duration;

use time::OffsetDateTime;

use super::checks::{DaemonFact, Facts, GnomeFacts, SessionKind, StoreFact};
use crate::domain::entry::PauseState;
use crate::install::autostart::AutostartKind;
use crate::launch::LaunchPlan;
use crate::session::{self, SessionEnv};

/// How long one external probe may take.
pub const PROBE_TIMEOUT: Duration = Duration::from_millis(300);

/// Everything the probes read, injected.
#[derive(Debug, Clone)]
pub struct ProbeEnv {
    pub now: OffsetDateTime,
    pub session: SessionEnv,
    pub xauthority: Option<PathBuf>,
    /// `XDG_CURRENT_DESKTOP`.
    pub current_desktop: Option<String>,
    /// `TERMINAL`.
    pub terminal: Option<String>,
    /// The `PATH` string the terminal search looks in.
    pub path: String,
    pub socket: Option<PathBuf>,
    pub store_dir: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    /// `$XDG_CONFIG_HOME`: holds `autostart/` and `systemd/user/`.
    pub config_home: Option<PathBuf>,
    /// `$XDG_DATA_HOME`: holds `gnome-shell/extensions/`.
    pub data_home: Option<PathBuf>,
    pub gnome_shell: PathBuf,
    pub gnome_extensions: PathBuf,
    pub timeout: Duration,
}

impl ProbeEnv {
    /// An environment read through `get` (empty values count as unset). `uid` and
    /// `run_user_root` are the fallback for a missing `XDG_RUNTIME_DIR`.
    #[must_use]
    pub fn from_lookup(
        get: &dyn Fn(&str) -> Option<String>,
        uid: u32,
        run_user_root: &Path,
        now: OffsetDateTime,
    ) -> Self {
        let _ = (get, uid, run_user_root, now);
        todo!()
    }

    /// The real process environment. Only [`super::run`] calls this.
    #[must_use]
    pub fn from_process() -> Self {
        todo!()
    }
}

/// The kind of session an environment describes.
#[must_use]
pub fn session_kind(env: &SessionEnv) -> SessionKind {
    let _ = env;
    todo!()
}

/// `48.7` out of `GNOME Shell 48.7`.
#[must_use]
pub fn parse_shell_version(output: &str) -> Option<String> {
    let _ = output;
    todo!()
}

/// Runs `program args`, returns its stdout when it exits successfully within `timeout`; the
/// child is killed on timeout. `None` for a missing program, a failure or a timeout.
#[must_use]
pub fn run_capture(program: &Path, args: &[&str], timeout: Duration) -> Option<String> {
    let _ = (program, args, timeout);
    todo!()
}

/// The daemon behind the control socket, asked for `status`.
#[must_use]
pub fn daemon_fact(env: &ProbeEnv) -> DaemonFact {
    let _ = env;
    todo!()
}

/// The store's permissions, entry count and persisted pause. Opens it read-only, and only
/// when no daemon answered (the daemon knows the count itself).
#[must_use]
pub fn store_fact(env: &ProbeEnv, daemon: &DaemonFact) -> (StoreFact, Option<PauseState>) {
    let _ = (env, daemon);
    todo!()
}

/// The GNOME details: Shell version, extension install, the version this binary ships.
#[must_use]
pub fn gnome_facts(env: &ProbeEnv) -> GnomeFacts {
    let _ = env;
    todo!()
}

/// How the picker would be launched (`picker_command` from the config file, `$TERMINAL`, the
/// terminal list searched in `env.path`).
#[must_use]
pub fn picker_plan(env: &ProbeEnv) -> LaunchPlan {
    let _ = env;
    todo!()
}

/// The autostart entries that exist.
#[must_use]
pub fn autostart_facts(env: &ProbeEnv) -> Vec<(AutostartKind, PathBuf)> {
    let _ = env;
    todo!()
}

/// Runs every probe. `probe` answers the two questions the environment cannot (the Wayland
/// registry and the X display), as for the daemon.
#[must_use]
pub fn gather(env: &ProbeEnv, probe: &mut dyn session::Probe) -> Facts {
    let _ = (env, probe);
    todo!()
}
