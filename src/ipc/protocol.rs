//! Protocol v1 types and request parsing. Canonical: `contracts/protocol.md` and
//! `contracts/protocol.schema.json`; every shape here is validated against that schema by
//! `tests/contract`.

use std::str::FromStr;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use time::OffsetDateTime;

use super::error::{ErrorCode, ProtoError};
use crate::domain::ids::EntryRef;

/// The protocol version carried in every message.
pub const VERSION: u64 = 1;

/// The longest accepted request line, newline excluded (2 MiB).
pub const MAX_LINE_BYTES: usize = 2 * 1024 * 1024;

/// Shortest and longest timed pause.
pub const PAUSE_RANGE: (Duration, Duration) =
    (Duration::from_secs(1), Duration::from_secs(24 * 60 * 60));

/// The longest `list` the daemon serves in one answer.
pub const LIST_MAX: u32 = 500;

/// One entry as clients see it. Never carries content beyond the escaped preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntrySummary {
    /// The 26-character time-ordered id.
    pub id: String,
    /// The shortest unique prefix (at least 4 characters).
    pub short_id: String,
    /// Position in the listing, 0 = newest.
    pub index: u32,
    /// Size in bytes.
    pub size: u32,
    /// First time the content was copied.
    #[serde(with = "time::serde::rfc3339")]
    pub created: OffsetDateTime,
    /// Last time the content was copied or restored.
    #[serde(with = "time::serde::rfc3339")]
    pub last_copied: OffsetDateTime,
    /// Whether the entry is pinned.
    pub pinned: bool,
    /// Control-character-escaped preview.
    pub preview: String,
}

/// `State.state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateKind {
    /// Copies are recorded.
    Recording,
    /// Recording is paused.
    Paused,
    /// No capture backend.
    Inactive,
}

/// The daemon state as clients see it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Which state.
    pub state: StateKind,
    /// End of a timed pause; `null` otherwise.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub paused_until: Option<OffsetDateTime>,
    /// Why the daemon is inactive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Capture counters since the daemon started.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counters {
    /// Copies skipped by the sensitive-content filter.
    pub skipped_sensitive: u64,
    /// Copies skipped for exceeding the size limit.
    pub skipped_oversize: u64,
    /// Copies skipped while paused.
    pub skipped_paused: u64,
    /// Pending changes dropped because the queue was full.
    pub dropped_burst: u64,
}

/// The `status` result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// State, pause end and reason.
    #[serde(flatten)]
    pub state: State,
    /// `wayland`, `x11` or `none`.
    pub backend: String,
    /// `native`, `xwayland_bridge` or `null`.
    pub mode: Option<String>,
    /// Number of entries.
    pub entries: u32,
    /// Number of pinned entries.
    pub pinned: u32,
    /// Configured entry cap.
    pub max_entries: u32,
    /// Configured pin cap.
    pub max_pinned: u32,
    /// Capture counters.
    pub counters: Counters,
    /// `active`, `no_host`, `no_bus` or `disabled`.
    pub tray: String,
    /// Whether `open_picker` can find a terminal.
    pub picker_available: bool,
    /// The daemon's version.
    pub version: String,
    /// Always 1.
    pub protocol: u64,
}

/// The two event names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// The history changed.
    HistoryChanged,
    /// The pause state changed.
    PauseChanged,
}

impl EventKind {
    /// Both kinds, the default of `subscribe`.
    pub const ALL: [Self; 2] = [Self::HistoryChanged, Self::PauseChanged];

    /// The wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HistoryChanged => "history_changed",
            Self::PauseChanged => "pause_changed",
        }
    }
}

/// A pushed event. Carries ids and counts only, never content.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// Which event.
    pub kind: EventKind,
    /// The `data` object.
    pub data: Value,
}

impl Event {
    /// `history_changed{entries, top}`.
    #[must_use]
    pub fn history_changed(entries: usize, top: Option<String>) -> Self {
        Self {
            kind: EventKind::HistoryChanged,
            data: json!({"entries": entries, "top": top}),
        }
    }

    /// `pause_changed` carrying the new `State`.
    #[must_use]
    pub fn pause_changed(state: &State) -> Self {
        Self {
            kind: EventKind::PauseChanged,
            data: serde_json::to_value(state).unwrap_or(Value::Null),
        }
    }

    /// The full wire object.
    #[must_use]
    pub fn to_value(&self) -> Value {
        json!({"v": VERSION, "event": self.kind.as_str(), "data": self.data})
    }
}

/// A parsed operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `list{limit?}`.
    List {
        /// At most this many entries.
        limit: Option<u32>,
    },
    /// `show{ref}`.
    Show(EntryRef),
    /// `restore{ref}`.
    Restore(EntryRef),
    /// `pin{ref}`.
    Pin(EntryRef),
    /// `unpin{ref}`.
    Unpin(EntryRef),
    /// `rm{ref}`.
    Rm(EntryRef),
    /// `clear{all?}`.
    Clear {
        /// Also remove pinned entries.
        all: bool,
    },
    /// `pause{for?}`; the duration is already within 1 s to 24 h.
    Pause {
        /// Length of a timed pause.
        duration: Option<Duration>,
    },
    /// `resume`.
    Resume,
    /// `status`.
    Status,
    /// `open_picker`.
    OpenPicker,
    /// `subscribe{events?}`.
    Subscribe {
        /// The events to receive (both when omitted).
        events: Vec<EventKind>,
    },
}

impl Command {
    /// The wire name of the operation, for logs.
    #[must_use]
    pub fn op(&self) -> &'static str {
        match self {
            Self::List { .. } => "list",
            Self::Show(_) => "show",
            Self::Restore(_) => "restore",
            Self::Pin(_) => "pin",
            Self::Unpin(_) => "unpin",
            Self::Rm(_) => "rm",
            Self::Clear { .. } => "clear",
            Self::Pause { .. } => "pause",
            Self::Resume => "resume",
            Self::Status => "status",
            Self::OpenPicker => "open_picker",
            Self::Subscribe { .. } => "subscribe",
        }
    }
}

/// A request that parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRequest {
    /// The request id to echo.
    pub id: u64,
    /// The operation.
    pub command: Command,
}

/// A request that did not parse: the error to answer with (`id` is 0 when none was readable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestError {
    /// The id to echo.
    pub id: u64,
    /// What to answer.
    pub error: ProtoError,
}

fn fail<T>(id: u64, code: ErrorCode, message: &str) -> Result<T, RequestError> {
    Err(RequestError {
        id,
        error: ProtoError::new(code, message),
    })
}

/// Parses one request line (newline already stripped).
///
/// # Errors
/// [`RequestError`] with `bad_request` (not JSON, not an object, no readable id, unknown op,
/// invalid arguments) or `unsupported_version`.
pub fn parse_request(line: &[u8]) -> Result<ParsedRequest, RequestError> {
    let Ok(value) = serde_json::from_slice::<Value>(line) else {
        return fail(0, ErrorCode::BadRequest, "malformed JSON");
    };
    let Some(obj) = value.as_object() else {
        return fail(0, ErrorCode::BadRequest, "a request is a JSON object");
    };
    let Some(id) = obj.get("id").and_then(Value::as_u64) else {
        return fail(
            0,
            ErrorCode::BadRequest,
            "id must be a non-negative integer",
        );
    };
    match obj.get("v") {
        Some(v) if v.as_u64() == Some(VERSION) => {}
        Some(_) => return fail(id, ErrorCode::UnsupportedVersion, "only v 1 is supported"),
        None => return fail(id, ErrorCode::BadRequest, "missing v"),
    }
    let Some(op) = obj.get("op").and_then(Value::as_str) else {
        return fail(id, ErrorCode::BadRequest, "missing op");
    };
    let empty = Map::new();
    let args = match obj.get("args") {
        None | Some(Value::Null) => &empty,
        Some(Value::Object(m)) => m,
        Some(_) => return fail(id, ErrorCode::BadRequest, "args must be an object"),
    };
    match command(op, args) {
        Ok(command) => Ok(ParsedRequest { id, command }),
        Err(message) => fail(id, ErrorCode::BadRequest, &message),
    }
}

fn command(op: &str, args: &Map<String, Value>) -> Result<Command, String> {
    Ok(match op {
        "list" => Command::List {
            limit: match args.get("limit") {
                None | Some(Value::Null) => None,
                Some(v) => {
                    let n = v
                        .as_u64()
                        .filter(|n| (1..=u64::from(LIST_MAX)).contains(n))
                        .ok_or_else(|| format!("limit must be an integer from 1 to {LIST_MAX}"))?;
                    Some(u32::try_from(n).map_err(|e| e.to_string())?)
                }
            },
        },
        "show" => Command::Show(reference(args)?),
        "restore" => Command::Restore(reference(args)?),
        "pin" => Command::Pin(reference(args)?),
        "unpin" => Command::Unpin(reference(args)?),
        "rm" => Command::Rm(reference(args)?),
        "clear" => Command::Clear {
            all: match args.get("all") {
                None | Some(Value::Null) => false,
                Some(Value::Bool(b)) => *b,
                Some(_) => return Err("all must be a boolean".to_string()),
            },
        },
        "pause" => Command::Pause {
            duration: match args.get("for") {
                None | Some(Value::Null) => None,
                Some(v) => {
                    let text = v.as_str().ok_or("for must be a string like \"50s\"")?;
                    let d = parse_duration(text)
                        .ok_or("for must be a number followed by s, m or h (e.g. \"50s\")")?;
                    if d < PAUSE_RANGE.0 || d > PAUSE_RANGE.1 {
                        return Err("for must be between 1s and 24h".to_string());
                    }
                    Some(d)
                }
            },
        },
        "resume" => Command::Resume,
        "status" => Command::Status,
        "open_picker" => Command::OpenPicker,
        "subscribe" => Command::Subscribe {
            events: match args.get("events") {
                None | Some(Value::Null) => EventKind::ALL.to_vec(),
                Some(Value::Array(items)) => {
                    let mut out = Vec::new();
                    for item in items {
                        let kind = EventKind::ALL
                            .into_iter()
                            .find(|k| item.as_str() == Some(k.as_str()))
                            .ok_or("unknown event name")?;
                        if !out.contains(&kind) {
                            out.push(kind);
                        }
                    }
                    out
                }
                Some(_) => return Err("events must be an array".to_string()),
            },
        },
        _ => return Err("unknown op".to_string()),
    })
}

fn reference(args: &Map<String, Value>) -> Result<EntryRef, String> {
    let text = args
        .get("ref")
        .and_then(Value::as_str)
        .ok_or("ref must be a string")?;
    EntryRef::from_str(text).map_err(|_| "ref is not an index or a short id".to_string())
}

/// Parses `^[0-9]+[smh]$` ("50s", "2m", "1h").
#[must_use]
pub fn parse_duration(text: &str) -> Option<Duration> {
    let unit = text.chars().last()?;
    let digits = text.get(..text.len() - 1)?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u64 = digits.parse().ok()?;
    let factor = match unit {
        's' => 1,
        'm' => 60,
        'h' => 3600,
        _ => return None,
    };
    n.checked_mul(factor).map(Duration::from_secs)
}

/// A success response.
#[must_use]
pub fn response_ok(id: u64, result: Value) -> Value {
    json!({"v": VERSION, "id": id, "ok": true, "result": result})
}

/// An error response.
#[must_use]
pub fn response_err(id: u64, error: &ProtoError) -> Value {
    json!({"v": VERSION, "id": id, "ok": false, "error": {"code": error.code, "message": error.message}})
}
