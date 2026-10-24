//! The control socket: runtime directory, binding, peer check, per-connection serving.
//!
//! The server is generic over the daemon: it forwards parsed commands over a [`Link`] and
//! pushes the events it is given. It knows nothing of the store or the backends.

use std::collections::HashSet;
use std::io;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use serde_json::{Value, json};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{Semaphore, broadcast, mpsc, oneshot, watch};

use super::codec::{CodecError, LineReader, encode};
use super::error::{Correlation, ErrorCode, ProtoError};
use super::peercred::peer_cred;
use super::protocol::{Command, Event, EventKind, parse_request, response_err, response_ok};

/// Directory (under the runtime dir) that holds the socket.
pub const SOCKET_DIR: &str = "rstory";
/// Socket file name.
pub const SOCKET_FILE: &str = "control.sock";
/// Most connections served at once; further ones are closed at once.
pub const MAX_CONNECTIONS: usize = 256;

const EVENT_BACKLOG: usize = 256;
const CALL_BACKLOG: usize = 64;

/// A request on its way to the daemon, with the channel for its answer.
#[derive(Debug)]
pub struct Call {
    /// The parsed operation.
    pub command: Command,
    /// Where the daemon sends the result.
    pub reply: oneshot::Sender<Result<Value, ProtoError>>,
}

/// The seam between the socket server and the daemon: calls in, events out. Cloneable; the
/// tray (spec 009) uses the same link in process.
#[derive(Debug, Clone)]
pub struct Link {
    calls: mpsc::Sender<Call>,
    events: broadcast::Sender<Event>,
}

impl Link {
    /// A link and the receiving end of its calls.
    #[must_use]
    pub fn new() -> (Self, mpsc::Receiver<Call>) {
        let (calls, rx) = mpsc::channel(CALL_BACKLOG);
        let (events, _) = broadcast::channel(EVENT_BACKLOG);
        (Self { calls, events }, rx)
    }

    /// Sends `command` to the daemon and waits for its answer.
    ///
    /// # Errors
    /// The daemon's [`ProtoError`], or `internal` when the daemon is gone.
    pub async fn request(&self, command: Command) -> Result<Value, ProtoError> {
        let (reply, answer) = oneshot::channel();
        let gone = || ProtoError::internal(Correlation::next());
        self.calls
            .send(Call { command, reply })
            .await
            .map_err(|_| gone())?;
        answer.await.map_err(|_| gone())?
    }

    /// Pushes an event to every subscribed connection.
    pub fn publish(&self, event: Event) {
        let _ = self.events.send(event);
    }

    /// A receiver of the events published from now on.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }
}

/// Why the runtime directory could not be determined.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RuntimeDirError {
    /// `XDG_RUNTIME_DIR` is unset and `/run/user/<uid>` is missing or not ours.
    #[error(
        "XDG_RUNTIME_DIR is not set and {0} does not exist or is not owned by this user; set XDG_RUNTIME_DIR to a private directory"
    )]
    Unavailable(PathBuf),
}

/// `$XDG_RUNTIME_DIR` when set and non-empty, else `<run_user_root>/<uid>` when it exists and
/// belongs to `uid`.
///
/// # Errors
/// [`RuntimeDirError::Unavailable`].
pub fn runtime_dir(
    env: Option<&str>,
    uid: u32,
    run_user_root: &Path,
) -> Result<PathBuf, RuntimeDirError> {
    if let Some(dir) = env.filter(|d| !d.is_empty()) {
        return Ok(PathBuf::from(dir));
    }
    let fallback = run_user_root.join(uid.to_string());
    match std::fs::metadata(&fallback) {
        Ok(m) if m.is_dir() && m.uid() == uid => Ok(fallback),
        _ => Err(RuntimeDirError::Unavailable(fallback)),
    }
}

/// `<runtime_dir>/rstory/control.sock`.
#[must_use]
pub fn socket_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join(SOCKET_DIR).join(SOCKET_FILE)
}

/// Creates the socket directory (mode `0700`) and binds the socket (mode `0600`).
///
/// The caller MUST hold the store's exclusive lock: that is what makes a leftover socket file
/// stale. A file that is not a socket is never touched, and a socket that still accepts
/// connections is never replaced.
///
/// # Errors
/// I/O errors; `AddrInUse` when a live socket is there; `InvalidInput` when the path holds
/// something else, or the directory belongs to another user.
pub fn bind(runtime_dir: &Path) -> io::Result<UnixListener> {
    let dir = runtime_dir.join(SOCKET_DIR);
    match std::fs::create_dir(&dir) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            if std::fs::metadata(&dir)?.uid() != rustix::process::getuid().as_raw() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "the socket directory belongs to another user",
                ));
            }
        }
        Err(e) => return Err(e),
    }
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;

    let path = dir.join(SOCKET_FILE);
    match std::fs::symlink_metadata(&path) {
        Ok(m) if m.file_type().is_socket() => {
            if std::os::unix::net::UnixStream::connect(&path).is_ok() {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "another process is serving the control socket",
                ));
            }
            std::fs::remove_file(&path)?;
        }
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the socket path holds something that is not a socket",
            ));
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    let listener = UnixListener::bind(&path)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

/// Accepts connections until `shutdown` turns true. Peers whose uid is not `expected_uid` are
/// dropped without a byte (Constitution III).
pub async fn serve(
    listener: UnixListener,
    link: Link,
    expected_uid: u32,
    mut shutdown: watch::Receiver<bool>,
) {
    let slots = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    loop {
        let accepted = tokio::select! {
            biased;
            _ = shutdown.changed() => break,
            r = listener.accept() => r,
        };
        let stream = match accepted {
            Ok((stream, _)) => stream,
            Err(e) => {
                tracing::warn!(kind = ?e.kind(), "accept failed");
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                continue;
            }
        };
        match peer_cred(&stream) {
            Ok(c) if c.uid == expected_uid => {}
            Ok(c) => {
                tracing::warn!(uid = c.uid, "peer_rejected");
                continue;
            }
            Err(e) => {
                tracing::warn!(kind = ?e.kind(), "peer credentials unavailable; closing");
                continue;
            }
        }
        let Ok(slot) = Arc::clone(&slots).try_acquire_owned() else {
            tracing::warn!(limit = MAX_CONNECTIONS, "too many connections; closing");
            continue;
        };
        let (link, shutdown) = (link.clone(), shutdown.clone());
        tokio::spawn(async move {
            connection(stream, link, shutdown).await;
            drop(slot);
        });
    }
}

async fn recv_event(
    sub: &mut Option<(HashSet<EventKind>, broadcast::Receiver<Event>)>,
) -> Option<Event> {
    let Some((kinds, rx)) = sub else {
        return std::future::pending().await;
    };
    loop {
        match rx.recv().await {
            Ok(event) if kinds.contains(&event.kind) => return Some(event),
            Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {}
            Err(broadcast::error::RecvError::Closed) => {
                *sub = None;
                return None;
            }
        }
    }
}

async fn connection(stream: UnixStream, link: Link, mut shutdown: watch::Receiver<bool>) {
    let conn = Correlation::next();
    let (read, mut write) = stream.into_split();
    let mut lines = LineReader::new(read);
    let mut sub: Option<(HashSet<EventKind>, broadcast::Receiver<Event>)> = None;
    tracing::debug!(conn = %conn, "connection opened");
    loop {
        let line = tokio::select! {
            biased;
            _ = shutdown.changed() => break,
            Some(event) = recv_event(&mut sub) => {
                if write.write_all(&encode(&event.to_value())).await.is_err() {
                    break;
                }
                continue;
            }
            line = lines.next_line() => line,
        };
        let out = match line {
            Ok(Some(line)) => answer(&line, &link, &mut sub, conn).await,
            Ok(None) | Err(CodecError::Io(_)) => break,
            Err(CodecError::TooLong) => {
                let e = ProtoError::new(ErrorCode::TooLarge, "request line is longer than 2 MiB");
                let _ = write.write_all(&encode(&response_err(0, &e))).await;
                tracing::warn!(conn = %conn, "oversize line; closing");
                break;
            }
        };
        if write.write_all(&encode(&out)).await.is_err() {
            break;
        }
    }
    tracing::debug!(conn = %conn, "connection closed");
}

/// Turns one request line into the response to write.
async fn answer(
    line: &[u8],
    link: &Link,
    sub: &mut Option<(HashSet<EventKind>, broadcast::Receiver<Event>)>,
    conn: Correlation,
) -> Value {
    let started = Instant::now();
    let parsed = match parse_request(line) {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!(conn = %conn, ok = false, code = %e.error.code, "request");
            return response_err(e.id, &e.error);
        }
    };
    let op = parsed.command.op();
    let result = if let Command::Subscribe { events } = parsed.command {
        // Subscribe before answering, so no event after the answer is missed.
        *sub = Some((events.into_iter().collect(), link.subscribe()));
        Ok(json!({}))
    } else {
        link.request(parsed.command).await
    };
    let ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    match result {
        Ok(v) => {
            tracing::debug!(conn = %conn, op, ms, ok = true, "request");
            response_ok(parsed.id, v)
        }
        Err(e) => {
            tracing::debug!(conn = %conn, op, ms, ok = false, code = %e.code, "request");
            response_err(parsed.id, &e)
        }
    }
}
