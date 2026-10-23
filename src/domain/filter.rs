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

/// The outcome of the checks that run before the clipboard is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreRead {
    /// Decided without reading: [`FilterDecision::SkipPaused`] or [`FilterDecision::SkipSensitive`].
    Skip(FilterDecision),
    /// Read at most `limit` bytes, then call [`CopyFilter::after_read`].
    Read {
        /// Read limit: `max_entry_bytes + 1`, so an oversize copy is seen as such, not truncated.
        limit: usize,
    },
}

/// The copy filter for one size cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyFilter {
    max_entry_bytes: u64,
}

impl CopyFilter {
    /// A filter dropping texts above `max_entry_bytes`.
    #[must_use]
    pub fn new(max_entry_bytes: u64) -> Self {
        Self { max_entry_bytes }
    }

    /// Pause and sensitive gates, on the offered type names alone: nothing is read for a skipped
    /// change.
    #[must_use]
    pub fn decide<S: AsRef<str>>(&self, types: &[S], paused: bool) -> PreRead {
        let _ = (types, paused, self.max_entry_bytes);
        todo!()
    }

    /// Post-read gates (oversize, empty). Consumes `bytes`, so a rejected text is dropped here.
    #[must_use]
    pub fn after_read(&self, bytes: Vec<u8>) -> FilterDecision {
        let _ = (bytes, self.max_entry_bytes);
        todo!()
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

    const HINT: &str = "x-kde-passwordManagerHint";
    const CAP: u64 = 16;

    fn f() -> CopyFilter {
        CopyFilter::new(CAP)
    }

    #[test]
    fn hint_with_text_is_skipped_unread() {
        let t = ["text/plain;charset=utf-8", HINT];
        assert_eq!(
            f().decide(&t, false),
            PreRead::Skip(FilterDecision::SkipSensitive)
        );
    }

    #[test]
    fn hint_only_is_skipped_unread() {
        assert_eq!(
            f().decide(&[HINT], false),
            PreRead::Skip(FilterDecision::SkipSensitive)
        );
    }

    #[test]
    fn hint_among_bridge_targets_is_skipped_unread() {
        let t = [
            "TARGETS",
            "TIMESTAMP",
            "UTF8_STRING",
            "STRING",
            HINT,
            "TEXT",
        ];
        assert_eq!(
            f().decide(&t, false),
            PreRead::Skip(FilterDecision::SkipSensitive)
        );
    }

    #[test]
    fn hint_matching_is_exact_and_case_sensitive() {
        for near in [
            "X-KDE-PASSWORDMANAGERHINT",
            "x-kde-passwordmanagerhint",
            "x-kde-passwordManagerHint ",
            "x-kde-passwordManager",
        ] {
            assert_eq!(
                f().decide(&[near], false),
                PreRead::Read { limit: 17 },
                "{near}"
            );
        }
    }

    #[test]
    fn plain_text_is_read_with_one_byte_over_the_cap() {
        assert_eq!(
            f().decide(&["UTF8_STRING"], false),
            PreRead::Read { limit: 17 }
        );
        let no_types: [&str; 0] = [];
        assert_eq!(f().decide(&no_types, false), PreRead::Read { limit: 17 });
    }

    #[test]
    fn paused_is_skipped_unread() {
        assert_eq!(
            f().decide(&["UTF8_STRING"], true),
            PreRead::Skip(FilterDecision::SkipPaused)
        );
    }

    #[test]
    fn order_of_checks_table() {
        // (types, paused, expected): pause wins over the hint.
        let table: [(&[&str], bool, PreRead); 4] = [
            (&[HINT], true, PreRead::Skip(FilterDecision::SkipPaused)),
            (&[HINT], false, PreRead::Skip(FilterDecision::SkipSensitive)),
            (
                &["UTF8_STRING"],
                true,
                PreRead::Skip(FilterDecision::SkipPaused),
            ),
            (&["UTF8_STRING"], false, PreRead::Read { limit: 17 }),
        ];
        for (types, paused, want) in table {
            assert_eq!(f().decide(types, paused), want, "{types:?} paused={paused}");
        }
    }

    #[test]
    fn size_boundaries() {
        let at = |n: u64| f().after_read(vec![b'a'; usize::try_from(n).unwrap()]);
        assert_eq!(at(CAP - 1), FilterDecision::Record(vec![b'a'; 15]));
        assert_eq!(at(CAP), FilterDecision::Record(vec![b'a'; 16]));
        assert_eq!(at(CAP + 1), FilterDecision::SkipOversize);
    }

    #[test]
    fn empty_is_ignored() {
        assert_eq!(f().after_read(Vec::new()), FilterDecision::IgnoreEmpty);
    }
}
