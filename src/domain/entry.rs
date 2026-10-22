//! Entry value objects: `Content`, `ContentHash`, `Preview`, plus the persisted `Entry` row
//! and `PauseState`.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

use super::ids::EntryId;
use super::redact::Redacted;

/// Longest preview, in characters (after escaping).
pub const PREVIEW_MAX_CHARS: usize = 256;

/// Why bytes are not valid [`Content`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContentError {
    /// Empty content is never recorded.
    #[error("content is empty")]
    Empty,
    /// The content exceeds the configured per-entry limit.
    #[error("content is {size} bytes; the limit is {max}")]
    TooLarge {
        /// Size of the rejected content.
        size: usize,
        /// The limit.
        max: u64,
    },
}

/// Non-empty clipboard text of at most `max_entry_bytes`. Its `Debug` shows only the size.
#[derive(Clone, PartialEq, Eq)]
pub struct Content(Vec<u8>);

impl Content {
    /// Validates `bytes` against the size limit.
    ///
    /// # Errors
    /// [`ContentError`] when empty or longer than `max_bytes`.
    pub fn new(bytes: Vec<u8>, max_bytes: u64) -> Result<Self, ContentError> {
        unimplemented!("T021")
    }

    /// The raw bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Size in bytes (always at least 1).
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Always false; present for API symmetry.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The de-duplication key.
    #[must_use]
    pub fn hash(&self) -> ContentHash {
        ContentHash::of(&self.0)
    }

    /// The listing preview.
    #[must_use]
    pub fn preview(&self) -> Preview {
        Preview::from_bytes(&self.0)
    }
}

impl fmt::Debug for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&Redacted(&self.0), f)
    }
}

/// BLAKE3 digest of the content; hex in JSON.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Hashes `bytes`.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        unimplemented!("T021")
    }

    /// Parses 64 hex characters.
    ///
    /// # Errors
    /// A message when the text is not 64 hex digits.
    pub fn from_hex(s: &str) -> Result<Self, String> {
        unimplemented!("T021")
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({self})")
    }
}

impl Serialize for ContentHash {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// The first characters of the content, control characters escaped, at most 256 characters.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Preview(String);

impl Preview {
    /// Builds the preview of `bytes` (lossy for invalid UTF-8).
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        unimplemented!("T021")
    }

    /// The preview text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// A preview is content-derived: never show it through `Debug`.
impl fmt::Debug for Preview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&Redacted(&self.0), f)
    }
}

impl<'de> Deserialize<'de> for Preview {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if s.chars().count() > PREVIEW_MAX_CHARS {
            return Err(serde::de::Error::custom("preview longer than 256 characters"));
        }
        Ok(Self(s))
    }
}

/// One history row, as persisted in the index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// Unique, time-ordered id.
    pub id: EntryId,
    /// De-duplication key.
    pub hash: ContentHash,
    /// Content size in bytes.
    pub size: u32,
    /// First time this content was recorded.
    #[serde(with = "time::serde::rfc3339")]
    pub created: OffsetDateTime,
    /// Last copy or restore; the ordering key.
    #[serde(with = "time::serde::rfc3339")]
    pub last_copied: OffsetDateTime,
    /// Whether the entry is pinned.
    pub pinned: bool,
    /// Listing preview.
    pub preview: Preview,
}

/// Whether recording is paused (persisted with the index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PauseState {
    /// Clipboard changes are recorded.
    #[default]
    Recording,
    /// Recording is paused, until the given instant or indefinitely.
    Paused {
        /// End of the pause, if timed.
        #[serde(default, with = "time::serde::rfc3339::option")]
        until: Option<OffsetDateTime>,
    },
}
