//! The blocking protocol client used by the CLI: one request at a time over a Unix stream,
//! with timeouts and typed errors. No runtime is started, so short-lived commands stay fast.
//!
//! Errors never carry clipboard content: a remote error is the daemon's code and message
//! (ids, counts and limits only), the rest names the kind of failure.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use super::error::ProtoError;
use super::protocol::VERSION;

/// How long a request may take to be answered.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

/// `restore` waits for the backend to confirm clipboard ownership.
pub const RESTORE_TIMEOUT: Duration = Duration::from_secs(5);

/// The longest response line accepted (a `show` of a full entry, escaped, fits easily).
const MAX_RESPONSE_BYTES: u64 = 16 * 1024 * 1024;

/// The timeout of one operation.
#[must_use]
pub fn timeout_for(op: &str) -> Duration {
    if op == "restore" {
        RESTORE_TIMEOUT
    } else {
        DEFAULT_TIMEOUT
    }
}

/// Why a call failed.
#[derive(Debug)]
pub enum ClientError {
    /// There is no daemon behind the socket (missing file, or nobody accepting).
    NotRunning,
    /// The daemon did not answer in time.
    Timeout,
    /// The connection failed in some other way.
    Io(io::Error),
    /// The answer was not a protocol v1 response, or the connection closed first.
    Protocol(String),
    /// The daemon answered with an error.
    Remote(ProtoError),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRunning => f.write_str("the daemon is not running"),
            Self::Timeout => f.write_str("the daemon did not answer in time"),
            Self::Io(e) => write!(f, "connection error: {}", e.kind()),
            Self::Protocol(m) => write!(f, "unexpected answer from the daemon: {m}"),
            Self::Remote(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ClientError {}

fn from_io(e: io::Error) -> ClientError {
    match e.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => ClientError::Timeout,
        io::ErrorKind::BrokenPipe
        | io::ErrorKind::ConnectionReset
        | io::ErrorKind::UnexpectedEof => ClientError::Protocol("the connection closed".into()),
        _ => ClientError::Io(e),
    }
}

/// A connection to the daemon's control socket.
#[derive(Debug)]
pub struct Client {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
    next_id: u64,
    timeout: Option<Duration>,
}

impl Client {
    /// Connects to the socket at `path`.
    ///
    /// # Errors
    /// [`ClientError::NotRunning`] when the file is missing or refuses connections;
    /// [`ClientError::Io`] otherwise.
    pub fn connect(path: &Path) -> Result<Self, ClientError> {
        let writer = UnixStream::connect(path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused => ClientError::NotRunning,
            _ => ClientError::Io(e),
        })?;
        let reader = BufReader::new(writer.try_clone().map_err(ClientError::Io)?);
        Ok(Self {
            reader,
            writer,
            next_id: 1,
            timeout: None,
        })
    }

    /// Overrides the per-operation timeouts (`None` restores them).
    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }

    /// Sends one request and waits for its response; events in between are skipped.
    ///
    /// # Errors
    /// See [`ClientError`].
    pub fn call(&mut self, op: &str, args: Value) -> Result<Value, ClientError> {
        let timeout = self.timeout.unwrap_or_else(|| timeout_for(op));
        self.writer
            .set_write_timeout(Some(timeout))
            .and_then(|()| self.reader.get_ref().set_read_timeout(Some(timeout)))
            .map_err(ClientError::Io)?;
        let id = self.next_id;
        self.next_id += 1;
        let mut line = json!({"v": VERSION, "id": id, "op": op, "args": args}).to_string();
        line.push('\n');
        self.writer.write_all(line.as_bytes()).map_err(from_io)?;
        loop {
            let value = self.read_value()?;
            if value.get("id").and_then(Value::as_u64) != Some(id) {
                continue; // an event, or an answer to nothing we asked
            }
            return if value.get("ok") == Some(&Value::Bool(true)) {
                Ok(value.get("result").cloned().unwrap_or(Value::Null))
            } else {
                let error = value
                    .get("error")
                    .and_then(|e| serde_json::from_value::<ProtoError>(e.clone()).ok())
                    .ok_or_else(|| ClientError::Protocol("an error without a code".into()))?;
                Err(ClientError::Remote(error))
            };
        }
    }

    fn read_value(&mut self) -> Result<Value, ClientError> {
        let mut buf = Vec::new();
        let n = (&mut self.reader)
            .take(MAX_RESPONSE_BYTES)
            .read_until(b'\n', &mut buf)
            .map_err(from_io)?;
        if n == 0 {
            return Err(ClientError::Protocol("the connection closed".into()));
        }
        serde_json::from_slice(&buf).map_err(|_| ClientError::Protocol("not JSON".into()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    use serde_json::{Value, json};

    use super::*;
    use crate::ipc::error::ErrorCode;

    /// A server that answers each request line with `reply(request)`; `None` stays silent.
    fn serve(
        reply: impl Fn(&Value) -> Option<String> + Send + 'static,
    ) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.sock");
        let listener = UnixListener::bind(&path).unwrap();
        thread::spawn(move || {
            let Ok((stream, _)) = listener.accept() else {
                return;
            };
            let mut out = stream.try_clone().unwrap();
            for line in BufReader::new(stream).lines() {
                let Ok(line) = line else { return };
                let req: Value = serde_json::from_str(&line).unwrap();
                if let Some(text) = reply(&req) {
                    out.write_all(text.as_bytes()).unwrap();
                    out.write_all(b"\n").unwrap();
                }
            }
        });
        (dir, path)
    }

    fn ok(req: &Value, result: Value) -> Option<String> {
        Some(json!({"v":1,"id":req["id"],"ok":true,"result":result}).to_string())
    }

    #[test]
    fn a_missing_socket_means_the_daemon_is_not_running() {
        let dir = tempfile::tempdir().unwrap();
        let err = Client::connect(&dir.path().join("none.sock")).unwrap_err();
        assert!(matches!(err, ClientError::NotRunning), "{err:?}");
    }

    #[test]
    fn a_stale_socket_file_means_the_daemon_is_not_running() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.sock");
        drop(UnixListener::bind(&path).unwrap());
        let err = Client::connect(&path).unwrap_err();
        assert!(matches!(err, ClientError::NotRunning), "{err:?}");
    }

    #[test]
    fn a_call_sends_the_envelope_and_returns_the_result() {
        let (_d, path) = serve(|req| {
            assert_eq!(req["v"], 1);
            assert_eq!(req["op"], "show");
            assert_eq!(req["args"], json!({"ref": "1"}));
            ok(req, json!({"content": "x"}))
        });
        let mut c = Client::connect(&path).unwrap();
        assert_eq!(
            c.call("show", json!({"ref": "1"})).unwrap(),
            json!({"content": "x"})
        );
    }

    #[test]
    fn ids_count_up_and_events_are_skipped() {
        let (_d, path) = serve(|req| {
            let id = req["id"].as_u64().unwrap();
            // an event first, then the answer on the same write
            Some(format!(
                "{}\n{}",
                json!({"v":1,"event":"history_changed","data":{"entries":1,"top":null}}),
                json!({"v":1,"id":id,"ok":true,"result":{"n":id}})
            ))
        });
        let mut c = Client::connect(&path).unwrap();
        assert_eq!(c.call("status", json!({})).unwrap()["n"], 1);
        assert_eq!(c.call("status", json!({})).unwrap()["n"], 2);
    }

    #[test]
    fn a_remote_error_keeps_its_code_and_message() {
        let (_d, path) = serve(|req| {
            Some(json!({"v":1,"id":req["id"],"ok":false,"error":{"code":"pin_limit","message":"pin limit reached (10)"}}).to_string())
        });
        let mut c = Client::connect(&path).unwrap();
        match c.call("pin", json!({"ref": "0"})).unwrap_err() {
            ClientError::Remote(e) => {
                assert_eq!(e.code, ErrorCode::PinLimit);
                assert_eq!(e.message, "pin limit reached (10)");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn silence_times_out() {
        let (_d, path) = serve(|_| None);
        let mut c = Client::connect(&path).unwrap();
        c.set_timeout(Some(Duration::from_millis(50)));
        let err = c.call("status", json!({})).unwrap_err();
        assert!(matches!(err, ClientError::Timeout), "{err:?}");
    }

    #[test]
    fn garbage_is_a_protocol_error() {
        let (_d, path) = serve(|_| Some("not json".to_string()));
        let mut c = Client::connect(&path).unwrap();
        let err = c.call("status", json!({})).unwrap_err();
        assert!(matches!(err, ClientError::Protocol(_)), "{err:?}");
    }

    #[test]
    fn a_closed_connection_is_a_protocol_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.sock");
        let l = UnixListener::bind(&path).unwrap();
        thread::spawn(move || drop(l.accept()));
        let mut c = Client::connect(&path).unwrap();
        let err = c.call("status", json!({})).unwrap_err();
        assert!(matches!(err, ClientError::Protocol(_)), "{err:?}");
    }

    #[test]
    fn restore_waits_longer_than_other_calls() {
        assert_eq!(timeout_for("list"), Duration::from_secs(2));
        assert_eq!(timeout_for("restore"), Duration::from_secs(5));
    }
}
