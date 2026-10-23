//! A daemon with a FakeBackend on a temporary store and socket, and a protocol client.
//! All clipboard text in the tests is synthetic.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rstory::backend::fake::{FakeBackend, FakeHandle};
use rstory::backend::{BackendKind, Mode};
use rstory::config::Caps;
use rstory::daemon::state::{BackendInfo, Clock};
use rstory::daemon::{Connector, Daemon, LaunchEnv, Probe, run_with_listener};
use rstory::ipc::server;
use rstory::store::Store;
use rstory::store::fs::RealFs;
use serde_json::{Value, json};
use tempfile::TempDir;
use time::OffsetDateTime;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::watch;

/// A fixed start so expiry and pause arithmetic is easy to read.
pub const BASE: OffsetDateTime = time::macros::datetime!(2026-10-21 14:00:00 UTC);

/// Directories that outlive one daemon, so a test can restart it.
pub struct Dirs {
    pub _tmp: TempDir,
    pub store: PathBuf,
    pub runtime: PathBuf,
}

impl Dirs {
    pub fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let runtime = tmp.path().join("run");
        std::fs::create_dir(&runtime).unwrap();
        let state = tmp.path().join("state");
        std::fs::create_dir(&state).unwrap();
        Self {
            store: state.join("rstory"),
            runtime,
            _tmp: tmp,
        }
    }

    /// The `rstory` binary with an empty environment pointing at these directories:
    /// the socket under `runtime`, the store at `<state>/rstory`, no config file, no color.
    /// Nothing of the real session or `$HOME` is visible to it.
    pub fn cli(&self, args: &[&str]) -> assert_cmd::Command {
        let root = self._tmp.path();
        let mut cmd = assert_cmd::Command::cargo_bin("rstory").unwrap();
        cmd.env_clear()
            .env("XDG_RUNTIME_DIR", &self.runtime)
            .env("XDG_STATE_HOME", root.join("state"))
            .env("XDG_CONFIG_HOME", root.join("config"))
            .env("HOME", root.join("home"))
            .env("LANG", "C.UTF-8")
            .env("NO_COLOR", "1")
            .args(args);
        cmd
    }

    /// Writes `<config>/rstory/config.toml`.
    pub fn write_config(&self, text: &str) {
        let dir = self._tmp.path().join("config/rstory");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.toml"), text).unwrap();
    }

    pub fn config_file(&self) -> PathBuf {
        self._tmp.path().join("config/rstory/config.toml")
    }

    pub fn socket(&self) -> PathBuf {
        server::socket_path(&self.runtime)
    }
}

/// What a finished CLI run left behind.
pub struct Ran {
    pub code: i32,
    pub stdout: Vec<u8>,
    pub stderr: String,
}

impl Ran {
    pub fn out(&self) -> String {
        String::from_utf8(self.stdout.clone()).unwrap()
    }
}

/// Runs the CLI on a blocking thread, so the in-process daemon keeps serving meanwhile.
pub async fn run(mut cmd: assert_cmd::Command) -> Ran {
    tokio::task::spawn_blocking(move || {
        let o = cmd.output().unwrap();
        Ran {
            code: o.status.code().unwrap_or(-1),
            stdout: o.stdout,
            stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
        }
    })
    .await
    .unwrap()
}

impl Dirs {
    /// `rstory <args>` run to completion.
    pub async fn rstory(&self, args: &[&str]) -> Ran {
        run(self.cli(args)).await
    }
}

pub struct Opts {
    pub caps: Caps,
    pub clock_base: OffsetDateTime,
    pub backend: bool,
    pub launch: LaunchEnv,
    pub connector: Option<Connector<FakeBackend>>,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            caps: Caps::default(),
            clock_base: BASE,
            backend: true,
            launch: no_terminal(),
            connector: None,
        }
    }
}

/// A launch environment that finds no terminal anywhere.
pub fn no_terminal() -> LaunchEnv {
    LaunchEnv {
        picker_command: None,
        terminal: None,
        on_path: Arc::new(|_| false),
    }
}

pub struct Rig {
    pub fake: FakeHandle,
    pub probe: Probe,
    pub socket: PathBuf,
    stop: watch::Sender<bool>,
    task: tokio::task::JoinHandle<Store<RealFs>>,
}

impl Rig {
    pub async fn start(dirs: &Dirs, opts: Opts) -> Self {
        let clock = Clock::at(opts.clock_base);
        let store = Store::open_exclusive_at(&dirs.store, opts.caps, RealFs, clock.now()).unwrap();
        let (backend, fake) = FakeBackend::new();
        let (backend, info) = if opts.backend {
            (
                Some(backend),
                BackendInfo::active(BackendKind::X11, Mode::Native),
            )
        } else {
            (None, BackendInfo::none("no_session"))
        };
        let mut daemon = Daemon::new(store, backend, info, opts.caps, clock, opts.launch).unwrap();
        if let Some(c) = opts.connector {
            daemon.set_connector(c);
        }
        let probe = daemon.probe();
        let listener = server::bind(&dirs.runtime).unwrap();
        let (stop, rx) = watch::channel(false);
        let task = tokio::spawn(run_with_listener(daemon, listener, rx));
        Self {
            fake,
            probe,
            socket: dirs.socket(),
            stop,
            task,
        }
    }

    pub async fn client(&self) -> Client {
        Client::connect(&self.socket).await
    }

    /// Stops the daemon and waits until the store lock is released.
    pub async fn stop(self) {
        self.stop.send(true).unwrap();
        let store = self.task.await.unwrap();
        drop(store);
    }
}

pub struct Client {
    rd: BufReader<OwnedReadHalf>,
    wr: OwnedWriteHalf,
    next_id: u64,
}

impl Client {
    pub async fn connect(path: &std::path::Path) -> Self {
        let (rd, wr) = UnixStream::connect(path).await.unwrap().into_split();
        Self {
            rd: BufReader::new(rd),
            wr,
            next_id: 1,
        }
    }

    pub async fn send_line(&mut self, line: &str) {
        self.wr.write_all(line.as_bytes()).await.unwrap();
        self.wr.write_all(b"\n").await.unwrap();
    }

    /// The next line as JSON, whatever it is (answer or event).
    pub async fn recv(&mut self) -> Value {
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_secs(10), self.rd.read_line(&mut line))
            .await
            .expect("a line in time")
            .unwrap();
        assert!(n > 0, "connection closed");
        serde_json::from_str(&line).unwrap()
    }

    /// Sends a request and returns the full response (events in between are skipped).
    pub async fn call(&mut self, op: &str, args: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send_line(&json!({"v":1,"id":id,"op":op,"args":args}).to_string())
            .await;
        loop {
            let v = self.recv().await;
            if v.get("id").and_then(Value::as_u64) == Some(id) {
                return v;
            }
        }
    }

    /// The `result` of a successful call.
    pub async fn ok(&mut self, op: &str, args: Value) -> Value {
        let r = self.call(op, args).await;
        assert_eq!(r["ok"], true, "{op} failed: {r}");
        r["result"].clone()
    }

    /// The error code of a failed call.
    pub async fn err(&mut self, op: &str, args: Value) -> String {
        let r = self.call(op, args).await;
        assert_eq!(r["ok"], false, "{op} should fail: {r}");
        r["error"]["code"].as_str().unwrap().to_string()
    }

    /// Subscribes to both events.
    pub async fn subscribe(&mut self) {
        self.ok("subscribe", json!({})).await;
    }

    /// The next event of the given name, skipping others.
    pub async fn event(&mut self, name: &str) -> Value {
        loop {
            let v = self.recv().await;
            if v.get("event").and_then(Value::as_str) == Some(name) {
                return v["data"].clone();
            }
        }
    }

    pub async fn list(&mut self) -> Vec<Value> {
        self.ok("list", json!({})).await["entries"]
            .as_array()
            .unwrap()
            .clone()
    }

    pub async fn status(&mut self) -> Value {
        self.ok("status", json!({})).await
    }
}

/// Emits one foreign copy of `text`: scripts the read and the event.
pub fn copy(fake: &FakeHandle, text: &str) {
    fake.push_read(Ok(text.as_bytes().to_vec()));
    fake.emit(rstory::backend::BackendEvent::SelectionChanged {
        types: vec!["UTF8_STRING".to_string()],
        self_owned: false,
    });
}

/// A change event with no scripted read: for copies the daemon must not read (paused).
pub fn change_only(fake: &FakeHandle) {
    fake.emit(rstory::backend::BackendEvent::SelectionChanged {
        types: vec!["UTF8_STRING".to_string()],
        self_owned: false,
    });
}
