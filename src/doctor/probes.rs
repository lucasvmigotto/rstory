//! The probes: the only doctor code that looks at the machine. Every input is a field of
//! [`ProbeEnv`] (environment values, directories, the `PATH` string, the socket path, the
//! `gnome-shell` and `gnome-extensions` programs), so tests point it at temp dirs, fakes and a
//! stub socket server. Probes only read: nothing is written, enabled or started. External
//! programs get `ProbeEnv::timeout` each (backend.md: 300 ms).

use std::io::Read;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use time::OffsetDateTime;

use super::checks::{DaemonFact, Facts, GnomeFacts, SessionFacts, SessionKind, StoreFact};
use crate::config;
use crate::domain::entry::PauseState;
use crate::install::autostart::{self, AutostartKind};
use crate::install::gnome::{self, ExtensionInstall};
use crate::ipc::client::{Client, ClientError};
use crate::ipc::server;
use crate::launch::{self, LaunchPlan};
use crate::session::{self, SessionEnv};
use crate::store::StoreError;
use crate::store::fs::RealFs;
use crate::store::perms;
use crate::store::readonly::ReadOnlyStore;

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
        let var = |k: &str| get(k).filter(|v| !v.is_empty());
        let home = var("HOME").map(PathBuf::from);
        let under_home = |rel: &str| home.as_ref().map(|h| h.join(rel));
        let socket = server::runtime_dir(var("XDG_RUNTIME_DIR").as_deref(), uid, run_user_root)
            .ok()
            .map(|dir| server::socket_path(&dir));
        let store_dir = var("XDG_STATE_HOME")
            .map(|s| PathBuf::from(s).join("rstory"))
            .or_else(|| under_home(".local/state/rstory"));
        let config_home = var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| under_home(".config"));
        let data_home = var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| under_home(".local/share"));
        Self {
            now,
            session: SessionEnv {
                wayland_display: var("WAYLAND_DISPLAY"),
                display: var("DISPLAY"),
                session_type: var("XDG_SESSION_TYPE"),
                xdg_runtime_dir: var("XDG_RUNTIME_DIR"),
            },
            xauthority: var("XAUTHORITY").map(PathBuf::from),
            current_desktop: var("XDG_CURRENT_DESKTOP"),
            terminal: var("TERMINAL"),
            path: var("PATH").unwrap_or_default(),
            socket,
            store_dir,
            config_path: config_home.as_ref().map(|c| c.join("rstory/config.toml")),
            config_home,
            data_home,
            gnome_shell: PathBuf::from("gnome-shell"),
            gnome_extensions: PathBuf::from("gnome-extensions"),
            timeout: PROBE_TIMEOUT,
        }
    }

    /// The real process environment. Only [`super::run`] calls this.
    #[must_use]
    pub fn from_process() -> Self {
        Self::from_lookup(
            &|k| std::env::var(k).ok(),
            rustix::process::getuid().as_raw(),
            Path::new("/run/user"),
            OffsetDateTime::now_utc(),
        )
    }
}

/// The kind of session an environment describes.
#[must_use]
pub fn session_kind(env: &SessionEnv) -> SessionKind {
    let set = |v: &Option<String>| v.as_deref().is_some_and(|s| !s.is_empty());
    match env
        .session_type
        .as_deref()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("wayland") => SessionKind::Wayland,
        Some("x11") => SessionKind::X11,
        _ if set(&env.wayland_display) => SessionKind::Wayland,
        _ if set(&env.display) => SessionKind::X11,
        _ => SessionKind::None,
    }
}

/// `48.7` out of `GNOME Shell 48.7`.
#[must_use]
pub fn parse_shell_version(output: &str) -> Option<String> {
    let rest = output
        .lines()
        .find_map(|l| l.trim().strip_prefix("GNOME Shell "))?;
    let version = rest.split_whitespace().next()?;
    version
        .starts_with(|c: char| c.is_ascii_digit())
        .then(|| version.to_string())
}

/// Runs `program args`, returns its stdout when it exits successfully within `timeout`; the
/// child is killed on timeout. `None` for a missing program, a failure or a timeout.
#[must_use]
pub fn run_capture(program: &Path, args: &[&str], timeout: Duration) -> Option<String> {
    let deadline = Instant::now() + timeout;
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout.read_to_string(&mut text);
        let _ = tx.send(text);
    });
    let left = deadline.saturating_duration_since(Instant::now());
    let output = rx.recv_timeout(left).ok();
    let status = output.as_ref().and_then(|_| {
        loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                _ => break None,
            }
        }
    });
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }
    output.filter(|_| status.is_some_and(|s| s.success()))
}

/// The daemon behind the control socket, asked for `status`.
#[must_use]
pub fn daemon_fact(env: &ProbeEnv) -> DaemonFact {
    let Some(path) = &env.socket else {
        return DaemonFact::Down;
    };
    let mut client = match Client::connect(path) {
        Ok(c) => c,
        // Connecting to a file that is not a socket is refused like a stale socket is.
        Err(ClientError::NotRunning)
            if std::fs::symlink_metadata(path).is_ok_and(|m| !m.file_type().is_socket()) =>
        {
            return DaemonFact::NotAnswering;
        }
        Err(ClientError::NotRunning) => return DaemonFact::Down,
        Err(_) => return DaemonFact::NotAnswering,
    };
    client.set_timeout(Some(env.timeout));
    client
        .call("status", serde_json::json!({}))
        .ok()
        .and_then(|v| serde_json::from_value(v).ok())
        .map_or(DaemonFact::NotAnswering, |s| DaemonFact::Up(Box::new(s)))
}

/// The store's permissions, entry count and persisted pause. Opens it read-only, and only
/// when no daemon answered (the daemon knows the count itself).
#[must_use]
pub fn store_fact(env: &ProbeEnv, daemon: &DaemonFact) -> (StoreFact, Option<PauseState>) {
    let Some(dir) = &env.store_dir else {
        return (StoreFact::Unknown, None);
    };
    let unreadable = || StoreFact::Unreadable { dir: dir.clone() };
    if matches!(std::fs::metadata(dir), Err(e) if e.kind() == std::io::ErrorKind::NotFound) {
        return (StoreFact::Missing(dir.clone()), None);
    }
    match perms::first_too_open(&RealFs, dir) {
        Err(_) => return (unreadable(), None),
        Ok(Some(_)) => return (StoreFact::TooOpen { dir: dir.clone() }, None),
        Ok(None) => {}
    }
    if let DaemonFact::Up(status) = daemon {
        let entries = Some(status.entries as usize);
        return (StoreFact::Private { entries }, None);
    }
    match ReadOnlyStore::open(dir) {
        Ok(store) => (
            StoreFact::Private {
                entries: Some(store.entries().len()),
            },
            Some(store.pause()),
        ),
        // The daemon holds the store but did not answer: the daemon check says so.
        Err(StoreError::DaemonRunning) => (StoreFact::Private { entries: None }, None),
        Err(_) => (unreadable(), None),
    }
}

/// The GNOME details: Shell version, extension install, the version this binary ships.
#[must_use]
pub fn gnome_facts(env: &ProbeEnv) -> GnomeFacts {
    let shell_version = run_capture(&env.gnome_shell, &["--version"], env.timeout)
        .and_then(|out| parse_shell_version(&out));
    let extension = env.data_home.as_ref().map(|data_home| {
        let gnome_env = gnome::Env {
            data_home: data_home.clone(),
            gnome_extensions: env.gnome_extensions.clone(),
            current_desktop: env.current_desktop.clone(),
        };
        // `inspect` runs `gnome-extensions` without a limit: wait for it on a helper thread.
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(gnome::inspect(&gnome_env));
        });
        rx.recv_timeout(env.timeout).unwrap_or_else(|_| {
            let path = gnome::extension_dir(data_home);
            ExtensionInstall {
                installed_version: installed_version(&path),
                path,
                enabled: None,
            }
        })
    });
    GnomeFacts {
        shell_version,
        extension,
        shipped_version: gnome::embedded_version(),
    }
}

/// How the picker would be launched (`picker_command` from the config file, `$TERMINAL`, the
/// terminal list searched in `env.path`).
#[must_use]
pub fn picker_plan(env: &ProbeEnv) -> LaunchPlan {
    let configured = env
        .config_path
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|text| config::picker_command_from_toml(&text).ok().flatten());
    launch::plan(configured.as_deref(), env.terminal.as_deref(), &|name| {
        launch::which_in(&env.path, name)
    })
}

/// The autostart entries that exist.
#[must_use]
pub fn autostart_facts(env: &ProbeEnv) -> Vec<(AutostartKind, PathBuf)> {
    let Some(home) = &env.config_home else {
        return Vec::new();
    };
    autostart::detect(home)
        .into_iter()
        .map(|kind| {
            let path = match kind {
                AutostartKind::Xdg => autostart::xdg_path(home),
                AutostartKind::Systemd => autostart::unit_path(home),
            };
            (kind, path)
        })
        .collect()
}

/// Runs every probe. `probe` answers the two questions the environment cannot (the Wayland
/// registry and the X display), as for the daemon.
#[must_use]
pub fn gather(env: &ProbeEnv, probe: &mut dyn session::Probe) -> Facts {
    let selection = session::select(&env.session, probe, None);
    let daemon = daemon_fact(env);
    let (store, store_pause) = store_fact(env, &daemon);
    let on_gnome = env
        .current_desktop
        .as_deref()
        .is_some_and(|d| d.split(':').any(|p| p.eq_ignore_ascii_case("gnome")));
    Facts {
        now: env.now,
        session: SessionFacts {
            kind: session_kind(&env.session),
            selection,
        },
        store,
        daemon,
        store_pause,
        gnome: on_gnome.then(|| gnome_facts(env)),
        picker: picker_plan(env),
        autostart: autostart_facts(env),
    }
}

/// `version-name` of an installed extension's `metadata.json`.
fn installed_version(dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(dir.join("metadata.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("version-name")?.as_str().map(str::to_string)
}
