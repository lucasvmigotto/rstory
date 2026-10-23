//! Where the picker's data comes from (FR-007): the daemon over the control socket, or the
//! store opened read-only when no daemon answers (BR14).
//!
//! The read-only source never keeps the store open: each `list` or `show` opens it, copies
//! what it needs and drops the handle before returning, so the shared lock is never held
//! while the picker waits for a key and a daemon can start meanwhile (spec 004).

use std::path::PathBuf;

use crate::ipc::protocol::EntrySummary;

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

/// The daemon over its control socket.
#[derive(Debug)]
pub struct DaemonSource {
    client: crate::ipc::client::Client,
}

impl DaemonSource {
    /// Connects to the socket at `path`.
    ///
    /// # Errors
    /// [`crate::ipc::client::ClientError::NotRunning`] when no daemon answers.
    pub fn connect(path: &std::path::Path) -> Result<Self, crate::ipc::client::ClientError> {
        let _ = path;
        todo!()
    }
}

impl Source for DaemonSource {
    fn read_only(&self) -> bool {
        false
    }
    fn list(&mut self) -> Result<Snapshot, Failure> {
        let _ = &self.client;
        todo!()
    }
    fn show(&mut self, id: &str) -> Result<String, Failure> {
        let _ = id;
        todo!()
    }
    fn act(&mut self, action: Action, id: &str) -> Result<(), Failure> {
        let _ = (action, id);
        todo!()
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
}

impl Source for ReadOnlySource {
    fn read_only(&self) -> bool {
        true
    }
    fn list(&mut self) -> Result<Snapshot, Failure> {
        let _ = &self.dir;
        todo!()
    }
    fn show(&mut self, id: &str) -> Result<String, Failure> {
        let _ = id;
        todo!()
    }
    fn act(&mut self, _action: Action, _id: &str) -> Result<(), Failure> {
        Err(Failure::DaemonDown)
    }
}

/// The daemon when it answers, else the read-only store.
///
/// # Errors
/// Neither is reachable: no socket and no store directory, or the socket failed oddly.
pub fn open(paths: &Paths) -> Result<Box<dyn Source>, String> {
    let _ = paths;
    todo!()
}

/// Subscribes to `history_changed` on its own connection and calls `notify` for each event,
/// on a background thread, until the connection closes or `notify` returns false.
///
/// # Errors
/// The socket could not be reached or refused the subscription.
pub fn subscribe(
    path: &std::path::Path,
    notify: impl FnMut() -> bool + Send + 'static,
) -> std::io::Result<()> {
    let _ = (path, notify);
    todo!()
}
