//! Privacy filter decisions (spec 005). Pure: no I/O, no logging.
//!
//! Order of checks, first match wins: paused, sensitive hint, no text type, size-limited read,
//! empty. Pause and sensitivity are decided on the offered type names alone, so a hinted copy is
//! never read (Constitution III).

use std::fmt;

use super::redact::Redacted;

/// Offered type names that mark a copy as sensitive. Exact and case-sensitive. This is the only
/// definition; extending it is a spec amendment (spec 005, FR-002).
pub const SENSITIVE_HINTS: [&str; 1] = ["x-kde-passwordManagerHint"];

/// What the filter decides about one clipboard change.
#[derive(Clone, PartialEq, Eq)]
pub enum FilterDecision {
    /// Store this text.
    Record(Vec<u8>),
    /// Recording is paused: not read.
    SkipPaused,
    /// A sensitive hint is offered: not read.
    SkipSensitive,
    /// Larger than the size cap: dropped whole, never truncated.
    SkipOversize,
    /// No text type is offered (no counter).
    IgnoreNoText,
    /// The text is empty (no counter).
    IgnoreEmpty,
}

impl fmt::Debug for FilterDecision {
    /// Never shows the recorded text, only its size.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Record(b) => write!(f, "Record({})", Redacted(b)),
            Self::SkipPaused => f.write_str("SkipPaused"),
            Self::SkipSensitive => f.write_str("SkipSensitive"),
            Self::SkipOversize => f.write_str("SkipOversize"),
            Self::IgnoreNoText => f.write_str("IgnoreNoText"),
            Self::IgnoreEmpty => f.write_str("IgnoreEmpty"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hint_list_is_exactly_the_specified_one() {
        assert_eq!(SENSITIVE_HINTS, ["x-kde-passwordManagerHint"]);
    }

    #[test]
    fn debug_never_shows_the_text() {
        let d = FilterDecision::Record(b"synthetic-canary-text".to_vec());
        let shown = format!("{d:?}");
        assert!(!shown.contains("canary"));
        assert!(shown.contains("21 bytes"));
    }
}
