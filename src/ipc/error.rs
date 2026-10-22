//! Protocol error codes and per-connection correlation ids.
//!
//! Messages never carry clipboard content (Constitution III): they name ids, counts and limits.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

/// The error catalog of `contracts/protocol.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Malformed JSON, missing or invalid arguments.
    BadRequest,
    /// The request line exceeded 2 MiB; the connection is closed.
    TooLarge,
    /// `v` is not 1.
    UnsupportedVersion,
    /// An unexpected daemon error.
    Internal,
    /// The reference matches no entry.
    NotFound,
    /// A short id matches several entries.
    Ambiguous,
    /// Pinning would exceed `max_pinned`.
    PinLimit,
    /// Restore while there is no capture backend.
    Inactive,
    /// The backend failed to take ownership.
    BackendError,
    /// `open_picker` with no resolvable terminal.
    NoTerminal,
    /// The configured `picker_command` could not be spawned.
    PickerCommandFailed,
}

impl ErrorCode {
    /// The wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BadRequest => "bad_request",
            Self::TooLarge => "too_large",
            Self::UnsupportedVersion => "unsupported_version",
            Self::Internal => "internal",
            Self::NotFound => "not_found",
            Self::Ambiguous => "ambiguous",
            Self::PinLimit => "pin_limit",
            Self::Inactive => "inactive",
            Self::BackendError => "backend_error",
            Self::NoTerminal => "no_terminal",
            Self::PickerCommandFailed => "picker_command_failed",
        }
    }

    /// The CLI exit code of the catalog (`None` where the CLI has no mapping).
    #[must_use]
    pub fn cli_exit(self) -> Option<u8> {
        match self {
            Self::BadRequest | Self::NotFound | Self::Ambiguous => Some(2),
            Self::PinLimit => Some(6),
            Self::Inactive => Some(7),
            Self::TooLarge | Self::UnsupportedVersion | Self::BackendError | Self::Internal => {
                Some(1)
            }
            Self::NoTerminal | Self::PickerCommandFailed => None,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An error as it travels in a response: a code and a human message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtoError {
    /// Machine-readable code.
    pub code: ErrorCode,
    /// Human-readable message; never content.
    pub message: String,
}

impl ProtoError {
    /// An error with `code` and `message`.
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// `bad_request`.
    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::BadRequest, message)
    }

    /// `internal`; the message carries only the correlation id of the log line.
    #[must_use]
    pub fn internal(correlation: Correlation) -> Self {
        Self::new(
            ErrorCode::Internal,
            format!("internal error (see the daemon log, {correlation})"),
        )
    }
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ProtoError {}

/// A process-unique id that ties a log line to the error a client saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Correlation(u64);

impl Correlation {
    /// The next id.
    #[must_use]
    pub fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    /// The numeric value.
    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Correlation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_names_match_serde_names() {
        for code in [
            ErrorCode::BadRequest,
            ErrorCode::TooLarge,
            ErrorCode::UnsupportedVersion,
            ErrorCode::Internal,
            ErrorCode::NotFound,
            ErrorCode::Ambiguous,
            ErrorCode::PinLimit,
            ErrorCode::Inactive,
            ErrorCode::BackendError,
            ErrorCode::NoTerminal,
            ErrorCode::PickerCommandFailed,
        ] {
            assert_eq!(
                serde_json::to_string(&code).unwrap(),
                format!("\"{}\"", code.as_str())
            );
        }
    }

    #[test]
    fn cli_exit_follows_the_catalog() {
        assert_eq!(ErrorCode::NotFound.cli_exit(), Some(2));
        assert_eq!(ErrorCode::PinLimit.cli_exit(), Some(6));
        assert_eq!(ErrorCode::Inactive.cli_exit(), Some(7));
        assert_eq!(ErrorCode::Internal.cli_exit(), Some(1));
        assert_eq!(ErrorCode::NoTerminal.cli_exit(), None);
    }

    #[test]
    fn correlation_ids_are_distinct() {
        assert_ne!(Correlation::next(), Correlation::next());
    }
}
