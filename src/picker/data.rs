//! Where the picker's data comes from (FR-007): the daemon over the control socket, or the
//! store opened read-only when no daemon answers (BR14).
//!
//! The read-only source never keeps the store open: each `list` or `show` opens it, copies
//! what it needs and drops the handle before returning, so the shared lock is never held
//! while the picker waits for a key and a daemon can start meanwhile (spec 004).

use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use time::OffsetDateTime;

use crate::domain::ids::EntryRef;
use crate::ipc::client::{Client, ClientError};
use crate::ipc::error::ErrorCode;
use crate::ipc::protocol::{EntrySummary, State, StateKind, VERSION};
use crate::store::StoreError;
use crate::store::fs::RealFs;
use crate::store::readonly::ReadOnlyStore;
use crate::ui::widgets::RecState;

use super::state::{Action, Failure, Snapshot};

/// The operations the picker needs. Ids are full entry ids.
pub trait Source {
    /// True when writes are impossible (no daemon).
    fn read_only(&self) -> bool;
    /// The history, newest first, with the current time and recording state.
    ///
    /// # Errors
    /// The listing could not be read.
    fn list(&mut self) -> Result<Snapshot, Failure>;
    /// The full content of one entry, as text.
    ///
    /// # Errors
    /// The entry is gone or could not be read.
    fn show(&mut self, id: &str) -> Result<String, Failure>;
    /// Runs `action` on `id` through the daemon.
    ///
    /// # Errors
    /// [`Failure::DaemonDown`] for a read-only source, else what the daemon answered.
    fn act(&mut self, action: Action, id: &str) -> Result<(), Failure>;
}

/// Where the daemon socket and the store live (computed by the CLI from the environment).
#[derive(Debug, Clone, Default)]
pub struct Paths {
    /// The control socket; `None` when there is no runtime directory.
    pub socket: Option<PathBuf>,
    /// The store directory; `None` when neither `XDG_STATE_HOME` nor `HOME` is set.
    pub store: Option<PathBuf>,
}

/// Re-indexes a listing newest first (the read-only store does not carry indexes).
#[must_use]
pub fn indexed(mut entries: Vec<EntrySummary>) -> Vec<EntrySummary> {
    for (i, e) in entries.iter_mut().enumerate() {
        e.index = u32::try_from(i).unwrap_or(u32::MAX);
    }
    entries
}

/// The recording state from a `status` result.
fn rec_state(status: &Value, now: OffsetDateTime) -> Option<RecState> {
    let parsed: State = serde_json::from_value(status.clone()).ok()?;
    Some(match parsed.state {
        StateKind::Recording => RecState::Recording,
        StateKind::Paused => match parsed.paused_until {
            Some(until) => RecState::PausedUntil {
                secs: u64::try_from((until - now).whole_seconds()).unwrap_or(0),
            },
            None => RecState::Paused,
        },
        StateKind::Inactive => RecState::Inactive {
            reason: parsed.reason.unwrap_or_default(),
        },
    })
}

/// The daemon over its control socket.
#[derive(Debug)]
pub struct DaemonSource {
    client: Client,
    path: PathBuf,
}

impl DaemonSource {
    /// Connects to the socket at `path`.
    ///
    /// # Errors
    /// [`ClientError::NotRunning`] when no daemon answers.
    pub fn connect(path: &Path) -> Result<Self, ClientError> {
        Ok(Self {
            client: Client::connect(path)?,
            path: path.to_path_buf(),
        })
    }

    /// A failure for the state core. A broken connection counts as "daemon down" only when
    /// nobody answers on the socket any more.
    fn failure(&self, e: ClientError) -> Failure {
        match e {
            ClientError::Remote(p) if p.code == ErrorCode::PinLimit => Failure::PinLimit,
            ClientError::Remote(p) => Failure::Other(p.message),
            ClientError::NotRunning => Failure::DaemonDown,
            other => match Client::connect(&self.path) {
                Err(ClientError::NotRunning) => Failure::DaemonDown,
                _ => Failure::Other(other.to_string()),
            },
        }
    }

    fn call(&mut self, op: &str, args: Value) -> Result<Value, Failure> {
        self.client.call(op, args).map_err(|e| self.failure(e))
    }
}

fn bad_answer() -> Failure {
    Failure::Other("The daemon sent an answer rstory does not understand.".into())
}

impl Source for DaemonSource {
    fn read_only(&self) -> bool {
        false
    }

    fn list(&mut self) -> Result<Snapshot, Failure> {
        let result = self.call("list", json!({}))?;
        let entries: Vec<EntrySummary> = result
            .get("entries")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(bad_answer)?;
        let now = OffsetDateTime::now_utc();
        // The badge is a nicety: a failed status leaves it out rather than failing the list.
        let rec = self
            .call("status", json!({}))
            .ok()
            .and_then(|s| rec_state(&s, now));
        Ok(Snapshot { entries, now, rec })
    }

    fn show(&mut self, id: &str) -> Result<String, Failure> {
        let result = self.call("show", json!({"ref": id}))?;
        result
            .get("content")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(bad_answer)
    }

    fn act(&mut self, action: Action, id: &str) -> Result<(), Failure> {
        let op = match action {
            Action::Restore => "restore",
            Action::Pin => "pin",
            Action::Unpin => "unpin",
            Action::Remove => "rm",
        };
        self.call(op, json!({"ref": id})).map(|_| ())
    }
}

/// The store, opened read-only for each read and closed again.
#[derive(Debug, Clone)]
pub struct ReadOnlySource {
    dir: PathBuf,
}

impl ReadOnlySource {
    /// A source over the store at `dir` (a missing directory is an empty history).
    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn open(&self) -> Result<ReadOnlyStore<RealFs>, Failure> {
        ReadOnlyStore::open(&self.dir).map_err(|e| match e {
            StoreError::DaemonRunning => Failure::Other(
                "The daemon holds the store but its socket does not answer. Run `rstory doctor` for details."
                    .into(),
            ),
            other => Failure::Other(format!("Cannot read the store: {other}")),
        })
    }
}

impl Source for ReadOnlySource {
    fn read_only(&self) -> bool {
        true
    }

    fn list(&mut self) -> Result<Snapshot, Failure> {
        let store = self.open()?;
        let entries = store
            .entries()
            .iter()
            .zip(store.short_ids())
            .map(|(e, short)| EntrySummary {
                id: e.id.to_string(),
                short_id: short.to_string(),
                index: 0,
                size: e.size,
                created: e.created,
                last_copied: e.last_copied,
                pinned: e.pinned,
                preview: e.preview.as_str().to_string(),
            })
            .collect();
        drop(store);
        Ok(Snapshot {
            entries: indexed(entries),
            now: OffsetDateTime::now_utc(),
            rec: None,
        })
    }

    fn show(&mut self, id: &str) -> Result<String, Failure> {
        let r: EntryRef = id
            .parse()
            .map_err(|_| Failure::Other("not an entry id".into()))?;
        let store = self.open()?;
        let entry_id = store
            .resolve(&r)
            .map_err(|_| Failure::Other("the entry is gone".into()))?
            .id;
        let bytes = store
            .content(&entry_id)
            .map_err(|e| Failure::Other(format!("Cannot read the entry: {e}")))?;
        drop(store);
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn act(&mut self, _action: Action, _id: &str) -> Result<(), Failure> {
        Err(Failure::DaemonDown)
    }
}

/// The daemon when it answers, else the read-only store.
///
/// # Errors
/// Neither is reachable: no daemon and no store directory.
pub fn open(paths: &Paths) -> Result<Box<dyn Source>, String> {
    if let Some(socket) = &paths.socket
        && let Ok(d) = DaemonSource::connect(socket)
    {
        return Ok(Box::new(d));
    }
    match &paths.store {
        Some(dir) => Ok(Box::new(ReadOnlySource::new(dir.clone()))),
        None => Err("Neither XDG_STATE_HOME nor HOME is set, so the store is unknown.".into()),
    }
}

/// Subscribes to `history_changed` on its own connection and calls `notify` for each event,
/// on a background thread, until the connection closes or `notify` returns false.
///
/// # Errors
/// The socket could not be reached.
pub fn subscribe(path: &Path, mut notify: impl FnMut() -> bool + Send + 'static) -> io::Result<()> {
    let mut stream = UnixStream::connect(path)?;
    let mut line = json!({
        "v": VERSION,
        "id": 1,
        "op": "subscribe",
        "args": {"events": ["history_changed"]},
    })
    .to_string();
    line.push('\n');
    stream.write_all(line.as_bytes())?;
    std::thread::Builder::new()
        .name("rstory-pick-subscribe".into())
        .spawn(move || {
            for line in BufReader::new(stream).lines() {
                let Ok(line) = line else { return };
                let Ok(v) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                if v.get("event").and_then(Value::as_str) == Some("history_changed") && !notify() {
                    return;
                }
            }
        })?;
    Ok(())
}
