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
pub struct Content(Vec<u8>, u32);

impl Content {
    /// Validates `bytes` against the size limit.
    ///
    /// # Errors
    /// [`ContentError`] when empty or longer than `max_bytes`.
    pub fn new(bytes: Vec<u8>, max_bytes: u64) -> Result<Self, ContentError> {
        if bytes.is_empty() {
            return Err(ContentError::Empty);
        }
        let too_large = || ContentError::TooLarge {
            size: bytes.len(),
            max: max_bytes,
        };
        let size = u32::try_from(bytes.len()).map_err(|_| too_large())?;
        if u64::from(size) > max_bytes {
            return Err(too_large());
        }
        Ok(Self(bytes, size))
    }

    /// Size in bytes as stored in the index (at least 1).
    #[must_use]
    pub fn size(&self) -> u32 {
        self.1
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
        Self(*blake3::hash(bytes).as_bytes())
    }

    /// Parses 64 hex characters.
    ///
    /// # Errors
    /// A message when the text is not 64 hex digits.
    pub fn from_hex(s: &str) -> Result<Self, String> {
        let b = s.as_bytes();
        if b.len() != 64 {
            return Err(format!("a hash has 64 hex digits, got {}", b.len()));
        }
        let digit = |c: u8| match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        };
        let mut out = [0u8; 32];
        for (i, o) in out.iter_mut().enumerate() {
            match (digit(b[2 * i]), digit(b[2 * i + 1])) {
                (Some(h), Some(l)) => *o = (h << 4) | l,
                _ => return Err("a hash must be hexadecimal".to_string()),
            }
        }
        Ok(Self(out))
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
        // 256 source characters take at most 1024 UTF-8 bytes; a cut at the end can only
        // damage a character past the 256th, which is never used.
        let head = &bytes[..bytes.len().min(PREVIEW_MAX_CHARS * 4)];
        let mut out = String::new();
        let mut chars = 0;
        for c in String::from_utf8_lossy(head).chars() {
            let piece = match c {
                '\n' => "\\n".to_string(),
                '\t' => "\\t".to_string(),
                c if c.is_control() => format!("\\x{:02x}", u32::from(c) & 0xff),
                c => c.to_string(),
            };
            let n = piece.chars().count();
            if chars + n > PREVIEW_MAX_CHARS {
                break;
            }
            chars += n;
            out.push_str(&piece);
        }
        Self(out)
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
            return Err(serde::de::Error::custom(
                "preview longer than 256 characters",
            ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn content_bounds() {
        assert_eq!(Content::new(vec![], 10), Err(ContentError::Empty));
        assert!(Content::new(vec![1; 10], 10).is_ok());
        assert_eq!(
            Content::new(vec![1; 11], 10),
            Err(ContentError::TooLarge { size: 11, max: 10 })
        );
    }

    #[test]
    fn content_debug_hides_the_bytes() {
        let c = Content::new(b"hunter2-CANARY".to_vec(), 100).unwrap();
        assert_eq!(format!("{c:?}"), "<14 bytes>");
        assert_eq!(c.len(), 14);
    }

    #[test]
    fn hash_is_blake3_hex_and_roundtrips() {
        let h = ContentHash::of(b"abc");
        let hex = h.to_string();
        assert_eq!(hex.len(), 64);
        assert_eq!(hex, blake3::hash(b"abc").to_hex().to_string());
        assert_eq!(ContentHash::from_hex(&hex).unwrap(), h);
        assert_eq!(ContentHash::from_hex(&hex.to_uppercase()).unwrap(), h);
        assert_eq!(serde_json::to_string(&h).unwrap(), format!("\"{hex}\""));
        assert!(ContentHash::from_hex("zz").is_err());
        assert!(ContentHash::from_hex(&"g".repeat(64)).is_err());
        assert_ne!(ContentHash::of(b"abd"), h);
    }

    #[test]
    fn preview_escapes_controls() {
        let p = Preview::from_bytes(b"a\tb\nc\x01\r\x7f");
        assert_eq!(p.as_str(), "a\\tb\\nc\\x01\\x0d\\x7f");
    }

    #[test]
    fn preview_is_lossy_for_invalid_utf8() {
        let p = Preview::from_bytes(&[b'o', b'k', 0xff, 0xfe, b'!']);
        assert_eq!(p.as_str(), "ok\u{fffd}\u{fffd}!");
    }

    #[test]
    fn preview_is_capped_at_256_chars() {
        let p = Preview::from_bytes("é".repeat(1000).as_bytes());
        assert_eq!(p.as_str().chars().count(), 256);
        let p = Preview::from_bytes(&b"\n".repeat(1000));
        assert_eq!(
            p.as_str().chars().count(),
            256,
            "escapes count by their output length"
        );
        assert!(p.as_str().chars().count() <= PREVIEW_MAX_CHARS);
        let p = Preview::from_bytes("😀".repeat(300).as_bytes());
        assert_eq!(p.as_str().chars().count(), 256);
        assert!(!p.as_str().contains('\u{fffd}'));
    }

    #[test]
    fn preview_debug_hides_text() {
        let p = Preview::from_bytes(b"secret");
        assert_eq!(format!("{p:?}"), "<6 bytes>");
    }

    #[test]
    fn preview_deserialize_rejects_overlong() {
        let long = format!("\"{}\"", "a".repeat(257));
        assert!(serde_json::from_str::<Preview>(&long).is_err());
        assert!(serde_json::from_str::<Preview>("\"ok\"").is_ok());
    }

    #[test]
    fn pause_state_json_shape() {
        assert_eq!(
            serde_json::to_string(&PauseState::Recording).unwrap(),
            r#"{"kind":"recording"}"#
        );
        let p = PauseState::Paused {
            until: Some(datetime!(2026-10-22 12:00:00 UTC)),
        };
        let j = serde_json::to_string(&p).unwrap();
        assert_eq!(j, r#"{"kind":"paused","until":"2026-10-22T12:00:00Z"}"#);
        assert_eq!(serde_json::from_str::<PauseState>(&j).unwrap(), p);
        assert_eq!(
            serde_json::from_str::<PauseState>(r#"{"kind":"paused"}"#).unwrap(),
            PauseState::Paused { until: None }
        );
    }

    #[test]
    fn entry_json_shape_matches_the_data_model() {
        let c = Content::new(b"git rebase -i HEAD~3".to_vec(), 100).unwrap();
        let t = datetime!(2026-10-22 12:00:00 UTC);
        let e = Entry {
            id: EntryId::from_parts(1, 2),
            hash: c.hash(),
            size: 20,
            created: t,
            last_copied: t,
            pinned: false,
            preview: c.preview(),
        };
        let v: serde_json::Value = serde_json::to_value(&e).unwrap();
        let mut keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        assert_eq!(
            keys,
            [
                "created",
                "hash",
                "id",
                "last_copied",
                "pinned",
                "preview",
                "size"
            ]
        );
        assert_eq!(v["last_copied"], "2026-10-22T12:00:00Z");
        assert_eq!(serde_json::from_value::<Entry>(v).unwrap(), e);
    }
}
