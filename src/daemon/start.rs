//! Process start: single instance, socket, backend selection, signals, exit codes (FR-011).

use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;
use tokio::net::UnixStream;
use tokio::sync::watch;

use crate::backend::x11::{X11Backend, X11Options};
use crate::backend::{Backend, BackendKind, Mode};
use crate::config::Caps;
use crate::ipc::peercred::peer_cred;
use crate::ipc::server;
use crate::session::{self, RealProbe, SessionEnv};
use crate::store::fs::RealFs;
use crate::store::{Store, StoreError};

use super::state::{BackendInfo, Clock};
use super::{Connector, Daemon, LaunchEnv, error_kind, run_with_listener};

/// How long a daemon waits for a read-only client to release the store's shared lock.
pub const LOCK_RETRY: Duration = Duration::from_secs(2);
const LOCK_POLL: Duration = Duration::from_millis(100);

/// Why the daemon did not run, with its exit code.
#[derive(Debug, Error)]
pub enum StartError {
    /// Another daemon holds the store.
    #[error("already running (pid {})", .0.map_or_else(|| "unknown".to_string(), |p| p.to_string()))]
    AlreadyRunning(Option<u32>),
    /// The environment cannot host the daemon (runtime dir, store permissions, a client reading).
    #[error("{0}")]
    Environment(String),
    /// Anything else.
    #[error("{0}")]
    Unexpected(String),
}

impl StartError {
    /// 4 already running, 5 environment, 1 unexpected.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::AlreadyRunning(_) => 4,
            Self::Environment(_) => 5,
            Self::Unexpected(_) => 1,
        }
    }
}

/// Where the daemon lives and how it behaves.
pub struct Runtime {
    /// The store directory.
    pub store_dir: PathBuf,
    /// `$XDG_RUNTIME_DIR` (the socket goes in `rstory/` below it).
    pub runtime_dir: PathBuf,
    /// History caps.
    pub caps: Caps,
    /// Picker launch environment.
    pub launch: LaunchEnv,
    /// How long to wait for a read-only client holding the shared lock.
    pub lock_retry: Duration,
}

/// Opens the store exclusively. A held lock means a daemon when its socket answers, else a
/// read-only client: retry every 100 ms for `retry`, then give up with an environment error.
///
/// # Errors
/// [`StartError`].
pub async fn open_store(
    dir: &Path,
    caps: Caps,
    runtime_dir: &Path,
    retry: Duration,
    clock: &Clock,
) -> Result<Store<RealFs>, StartError> {
    let end = tokio::time::Instant::now() + retry;
    loop {
        match Store::open_exclusive_at(dir, caps, RealFs, clock.now()) {
            Ok(s) => return Ok(s),
            Err(StoreError::Locked) => {
                if let Ok(stream) = UnixStream::connect(server::socket_path(runtime_dir)).await {
                    return Err(StartError::AlreadyRunning(
                        peer_cred(&stream).ok().map(|c| c.pid),
                    ));
                }
                if tokio::time::Instant::now() >= end {
                    return Err(StartError::Environment(
                        "a client is reading the store; retry".to_string(),
                    ));
                }
                tokio::time::sleep(LOCK_POLL).await;
            }
            Err(e @ StoreError::StoreTooOpen { .. }) => {
                return Err(StartError::Environment(e.to_string()));
            }
            Err(e) => {
                return Err(StartError::Unexpected(format!(
                    "could not open the store ({})",
                    error_kind(&e)
                )));
            }
        }
    }
}

/// What a backend provider hands the daemon.
pub struct Provided<B> {
    /// The connected backend, if any.
    pub backend: Option<B>,
    /// What `status` reports.
    pub info: BackendInfo,
    /// Reconnect policy.
    pub connector: Option<Connector<B>>,
}

/// Locks the store, binds the socket, asks `provide` for the backend, and serves until
/// `shutdown`. The socket file is removed on the way out.
///
/// # Errors
/// [`StartError`] before serving starts.
pub async fn serve_process<B, P, Fut>(
    rt: Runtime,
    provide: P,
    shutdown: watch::Receiver<bool>,
) -> Result<(), StartError>
where
    B: Backend,
    P: FnOnce() -> Fut,
    Fut: Future<Output = Provided<B>>,
{
    let clock = Clock::system();
    let store = open_store(
        &rt.store_dir,
        rt.caps,
        &rt.runtime_dir,
        rt.lock_retry,
        &clock,
    )
    .await?;
    let listener = server::bind(&rt.runtime_dir).map_err(|e| {
        StartError::Environment(format!("could not bind the control socket: {}", e.kind()))
    })?;
    let provided = provide().await;
    tracing::info!(
        backend = provided.info.kind.as_str(),
        mode = provided.info.mode.as_str(),
        version = env!("CARGO_PKG_VERSION"),
        "daemon.start"
    );
    let mut daemon = Daemon::new(
        store,
        provided.backend,
        provided.info,
        rt.caps,
        clock,
        rt.launch,
    )
    .map_err(|e| StartError::Unexpected(format!("could not start ({})", error_kind(&e))))?;
    if let Some(c) = provided.connector {
        daemon.set_connector(c);
    }
    let store = run_with_listener(daemon, listener, shutdown).await;
    let _ = std::fs::remove_file(server::socket_path(&rt.runtime_dir));
    drop(store);
    Ok(())
}

/// Command-line options of `rstory daemon`.
#[derive(Debug, Clone, Default)]
pub struct DaemonArgs {
    /// `--backend x11|wayland`.
    pub backend: Option<BackendKind>,
}

/// Parses the arguments after `daemon`.
///
/// # Errors
/// A usage message.
pub fn parse_args(args: &[String]) -> Result<DaemonArgs, String> {
    let mut out = DaemonArgs::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let value = if let Some(v) = a.strip_prefix("--backend=") {
            v.to_string()
        } else if a == "--backend" {
            it.next()
                .ok_or("--backend needs a value (x11 or wayland)")?
                .clone()
        } else {
            return Err(format!(
                "unknown argument {a}; usage: rstory daemon [--backend x11|wayland]"
            ));
        };
        out.backend = Some(match value.as_str() {
            "x11" => BackendKind::X11,
            "wayland" => BackendKind::Wayland,
            _ => return Err("--backend takes x11 or wayland".to_string()),
        });
    }
    Ok(out)
}

fn env_var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn store_dir() -> Result<PathBuf, StartError> {
    if let Some(state) = env_var("XDG_STATE_HOME") {
        return Ok(PathBuf::from(state).join("rstory"));
    }
    let home = env_var("HOME")
        .ok_or_else(|| StartError::Environment("neither XDG_STATE_HOME nor HOME is set".into()))?;
    Ok(PathBuf::from(home).join(".local/state/rstory"))
}

fn load_caps() -> Result<Caps, StartError> {
    let dir = match env_var("XDG_CONFIG_HOME") {
        Some(c) => PathBuf::from(c),
        None => match env_var("HOME") {
            Some(h) => PathBuf::from(h).join(".config"),
            None => return Ok(Caps::default()),
        },
    };
    match std::fs::read_to_string(dir.join("rstory/config.toml")) {
        Ok(text) => Caps::from_toml(&text).map_err(|e| StartError::Environment(e.to_string())),
        Err(_) => Ok(Caps::default()),
    }
}

/// Connects the X11 backend for `info`; `None` when unreachable.
async fn connect_x11(display: String, bridge: bool) -> Option<X11Backend> {
    let opts = X11Options::new(&display);
    let opts = if bridge { opts.bridge() } else { opts };
    X11Backend::connect(opts).await.ok()
}

async fn provide_backend(args: DaemonArgs, env: SessionEnv) -> Provided<X11Backend> {
    let for_select = env.clone();
    let selection = tokio::task::spawn_blocking(move || {
        let mut probe = RealProbe::new(&for_select, None);
        session::select(&for_select, &mut probe, args.backend)
    })
    .await
    .ok();
    let Some(sel) = selection else {
        return Provided {
            backend: None,
            info: BackendInfo::none("selection_failed"),
            connector: None,
        };
    };
    match sel.backend {
        BackendKind::X11 => {
            let display = env.display.clone().unwrap_or_default();
            let bridge = sel.mode == Mode::XwaylandBridge;
            let backend = connect_x11(display.clone(), bridge).await;
            let info = if backend.is_some() {
                BackendInfo::active(BackendKind::X11, sel.mode)
            } else {
                BackendInfo::none(session::Reason::X11Unreachable.as_str())
            };
            let connector: Connector<X11Backend> = Box::new(move || {
                let display = display.clone();
                Box::pin(connect_x11(display, bridge))
            });
            let kind_mode = BackendInfo::active(BackendKind::X11, sel.mode);
            let mut p = Provided {
                backend,
                info,
                connector: Some(connector),
            };
            if p.backend.is_none() {
                // Reconnect restores this identity once the display answers.
                p.info = BackendInfo {
                    reason: p.info.reason,
                    ..kind_mode
                };
            }
            p
        }
        BackendKind::Wayland => Provided {
            backend: None,
            info: BackendInfo::none("wayland_backend_unavailable"),
            connector: None,
        },
        BackendKind::None => Provided {
            backend: None,
            info: BackendInfo::none(sel.reason.as_str()),
            connector: None,
        },
    }
}

/// `rstory daemon`: returns the process exit code.
pub async fn run_main(args: &[String]) -> u8 {
    super::log::init_logging();
    let args = match parse_args(args) {
        Ok(a) => a,
        Err(m) => {
            eprintln!("rstory: {m}");
            return 2;
        }
    };
    let result = run_inner(args).await;
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("rstory daemon: {e}");
            e.exit_code()
        }
    }
}

async fn run_inner(args: DaemonArgs) -> Result<(), StartError> {
    let uid = rustix::process::getuid().as_raw();
    let runtime_dir = server::runtime_dir(
        env_var("XDG_RUNTIME_DIR").as_deref(),
        uid,
        Path::new("/run/user"),
    )
    .map_err(|e| StartError::Environment(e.to_string()))?;
    let rt = Runtime {
        store_dir: store_dir()?,
        runtime_dir,
        caps: load_caps()?,
        launch: LaunchEnv::from_env(None),
        lock_retry: LOCK_RETRY,
    };
    let env = SessionEnv {
        wayland_display: env_var("WAYLAND_DISPLAY"),
        display: env_var("DISPLAY"),
        session_type: env_var("XDG_SESSION_TYPE"),
        xdg_runtime_dir: env_var("XDG_RUNTIME_DIR"),
    };
    let (stop, shutdown) = watch::channel(false);
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|e| StartError::Unexpected(e.to_string()))?;
    let mut int = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .map_err(|e| StartError::Unexpected(e.to_string()))?;
    tokio::spawn(async move {
        tokio::select! {
            _ = term.recv() => {}
            _ = int.recv() => {}
        }
        tracing::info!("signal received; shutting down");
        let _ = stop.send(true);
    });
    serve_process(rt, || provide_backend(args, env), shutdown).await
}
