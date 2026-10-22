//! Entry identifiers: `EntryId` (ULID layout), `ShortId` and `EntryRef`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use time::OffsetDateTime;

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const ID_LEN: usize = 26;
/// Shortest short id.
pub const MIN_SHORT_LEN: usize = 4;
const TS_MASK: u128 = (1 << 48) - 1;
const RAND_MASK: u128 = (1 << 80) - 1;

/// A 128-bit id: 48-bit millisecond timestamp, then 80 random bits, written as 26 characters
/// of Crockford base32. Ordering is by creation time (then by the random part).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(u128);

/// Why an id or reference could not be parsed.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdError {
    /// The id is not 26 characters long.
    #[error("an entry id has {ID_LEN} characters, got {0}")]
    Length(usize),
    /// A character is outside the Crockford base32 alphabet.
    #[error("invalid character {0:?} (expected Crockford base32: 0-9, A-Z without I, L, O, U)")]
    Char(char),
    /// The first character would overflow 128 bits.
    #[error("id out of range (first character must be 0-7)")]
    Overflow,
    /// A reference is neither an index nor a short id.
    #[error("not a list index or a short id (at least {MIN_SHORT_LEN} base32 characters): {0:?}")]
    Ref(String),
}

fn decode_char(c: char) -> Option<u8> {
    let u = c.to_ascii_uppercase();
    ALPHABET
        .iter()
        .position(|&a| u32::from(a) == u32::from(u))
        .and_then(|p| u8::try_from(p).ok())
}

impl EntryId {
    /// Creates an id for time `now` with 80 fresh random bits.
    #[must_use]
    pub fn new(now: OffsetDateTime) -> Self {
        let ms = now.unix_timestamp_nanos() / 1_000_000;
        let ms = u128::try_from(ms).unwrap_or(0) & TS_MASK;
        let entropy = rand::random::<u128>() & RAND_MASK;
        Self::from_parts(ms, entropy)
    }

    /// Builds an id from its timestamp (milliseconds) and 80-bit random part.
    #[must_use]
    pub fn from_parts(ms: u128, entropy: u128) -> Self {
        Self(((ms & TS_MASK) << 80) | (entropy & RAND_MASK))
    }

    /// The creation time in milliseconds since the Unix epoch.
    #[must_use]
    pub fn timestamp_ms(&self) -> u64 {
        u64::try_from(self.0 >> 80).unwrap_or(u64::MAX)
    }

    /// Parses 26 characters of Crockford base32 (case-insensitive).
    ///
    /// # Errors
    /// [`IdError`] on a wrong length, a bad character or overflow.
    pub fn parse(s: &str) -> Result<Self, IdError> {
        let n = s.chars().count();
        if n != ID_LEN {
            return Err(IdError::Length(n));
        }
        let mut v: u128 = 0;
        for (i, c) in s.chars().enumerate() {
            let d = decode_char(c).ok_or(IdError::Char(c))?;
            if i == 0 && d > 7 {
                return Err(IdError::Overflow);
            }
            v = (v << 5) | u128::from(d);
        }
        Ok(Self(v))
    }
}

impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8; ID_LEN];
        for (i, b) in buf.iter_mut().enumerate() {
            let shift = 5 * (ID_LEN - 1 - i);
            *b = ALPHABET[usize::try_from((self.0 >> shift) & 31).unwrap_or(0)];
        }
        f.write_str(std::str::from_utf8(&buf).unwrap_or_default())
    }
}

impl fmt::Debug for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EntryId({self})")
    }
}

impl FromStr for EntryId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, IdError> {
        Self::parse(s)
    }
}

impl Serialize for EntryId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for EntryId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// The shortest prefix (at least [`MIN_SHORT_LEN`] characters) of an id that is unique among
/// the ids of one listing. Recomputed per listing; never stored.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortId(String);

impl ShortId {
    /// The short id as text (upper case).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Computes the short id of every id in `ids`, in the same order. Duplicate ids (which
    /// cannot be told apart) get the full id.
    #[must_use]
    pub fn compute(ids: &[EntryId]) -> Vec<Self> {
        let texts: Vec<String> = ids.iter().map(ToString::to_string).collect();
        texts
            .iter()
            .map(|t| {
                let mut len = MIN_SHORT_LEN;
                for o in &texts {
                    if o == t {
                        continue;
                    }
                    let common = t.bytes().zip(o.bytes()).take_while(|(a, b)| a == b).count();
                    len = len.max(common + 1);
                }
                // An identical twin makes every prefix ambiguous: use the whole id.
                if texts.iter().filter(|o| *o == t).count() > 1 {
                    len = ID_LEN;
                }
                Self(t[..len.min(ID_LEN)].to_string())
            })
            .collect()
    }
}

impl fmt::Display for ShortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// How a user points at an entry: a list position (0 = newest) or a short id prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryRef {
    /// A position in the current listing.
    Index(u32),
    /// A (short) id prefix.
    ShortId(String),
}

impl FromStr for EntryRef {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, IdError> {
        if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) {
            return s
                .parse()
                .map(Self::Index)
                .map_err(|_| IdError::Ref(s.to_string()));
        }
        if s.len() >= MIN_SHORT_LEN
            && s.len() <= ID_LEN
            && s.chars().all(|c| decode_char(c).is_some())
        {
            return Ok(Self::ShortId(s.to_ascii_uppercase()));
        }
        Err(IdError::Ref(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use time::macros::datetime;

    fn id(ms: u128, e: u128) -> EntryId {
        EntryId::from_parts(ms, e)
    }

    #[test]
    fn new_has_26_chars_and_the_time() {
        let now = datetime!(2026-10-22 12:00:00 UTC);
        let a = EntryId::new(now);
        assert_eq!(a.to_string().len(), 26);
        assert_eq!(
            i128::from(a.timestamp_ms()),
            now.unix_timestamp_nanos() / 1_000_000
        );
        assert_ne!(a, EntryId::new(now), "80 random bits differ");
    }

    #[test]
    fn display_parse_roundtrip_and_case() {
        let a = id(1_700_000_000_000, 0xDEAD_BEEF_CAFE_F00D_1234);
        let s = a.to_string();
        assert_eq!(EntryId::parse(&s).unwrap(), a);
        assert_eq!(EntryId::parse(&s.to_lowercase()).unwrap(), a);
        assert_eq!(s.parse::<EntryId>().unwrap(), a);
    }

    #[test]
    fn known_layout() {
        assert_eq!(id(0, 0).to_string(), "00000000000000000000000000");
        assert_eq!(
            id(TS_MASK, RAND_MASK).to_string(),
            "7ZZZZZZZZZZZZZZZZZZZZZZZZZ"
        );
    }

    #[test]
    fn parse_errors() {
        assert_eq!(EntryId::parse("ABC"), Err(IdError::Length(3)));
        assert_eq!(
            EntryId::parse("0000000000000000000000000U"),
            Err(IdError::Char('U'))
        );
        assert_eq!(
            EntryId::parse("8ZZZZZZZZZZZZZZZZZZZZZZZZZ"),
            Err(IdError::Overflow)
        );
    }

    #[test]
    fn ordering_follows_time() {
        let a = id(1000, u128::MAX);
        let b = id(1001, 0);
        assert!(a < b);
        assert!(a.to_string() < b.to_string(), "text order matches id order");
        let mut v = vec![b, a];
        v.sort();
        assert_eq!(v, [a, b]);
    }

    #[test]
    fn serde_as_string() {
        let a = id(42, 7);
        let j = serde_json::to_string(&a).unwrap();
        assert_eq!(j, format!("\"{a}\""));
        assert_eq!(serde_json::from_str::<EntryId>(&j).unwrap(), a);
        assert!(serde_json::from_str::<EntryId>("\"nope\"").is_err());
        assert!(serde_json::from_str::<EntryId>("5").is_err());
    }

    #[test]
    fn short_ids_are_at_least_four_and_minimal() {
        let a = id(1, 0);
        let b = id(1, 1); // differs only in the last character
        let c = id(2, 0);
        let s = ShortId::compute(&[a, b, c]);
        assert!(s[2].as_str().len() >= MIN_SHORT_LEN);
        assert_eq!(s[0].as_str().len(), 26);
        assert_eq!(s[1].as_str().len(), 26);
        assert!(s[2].as_str().len() < 26);
        assert!(a.to_string().starts_with(s[0].as_str()));
    }

    #[test]
    fn short_id_of_one_entry_is_four_chars() {
        let s = ShortId::compute(&[id(5, 5)]);
        assert_eq!(s[0].as_str().len(), 4);
        assert!(ShortId::compute(&[]).is_empty());
    }

    #[test]
    fn duplicate_ids_get_the_full_id() {
        let a = id(9, 9);
        let s = ShortId::compute(&[a, a]);
        assert_eq!(s[0].as_str(), a.to_string());
    }

    proptest! {
        #[test]
        fn short_ids_are_unique_prefixes(raw in proptest::collection::vec((0u128..50, 0u128..4), 1..30)) {
            let mut ids: Vec<EntryId> = raw.into_iter().map(|(t, e)| id(t, e)).collect();
            ids.sort();
            ids.dedup();
            let shorts = ShortId::compute(&ids);
            for (i, (a, sa)) in ids.iter().zip(&shorts).enumerate() {
                prop_assert!(sa.as_str().len() >= MIN_SHORT_LEN);
                prop_assert!(a.to_string().starts_with(sa.as_str()));
                for (j, b) in ids.iter().enumerate() {
                    if i != j {
                        prop_assert!(!b.to_string().starts_with(sa.as_str()), "{sa} also prefixes {b}");
                    }
                }
                // minimal: one character shorter (if still >= 4) would be ambiguous
                if sa.as_str().len() > MIN_SHORT_LEN {
                    let shorter = &sa.as_str()[..sa.as_str().len() - 1];
                    prop_assert!(ids.iter().enumerate().any(|(j, b)| j != i && b.to_string().starts_with(shorter)));
                }
            }
        }

        #[test]
        fn roundtrip(ms in 0u128..=TS_MASK, e in 0u128..=RAND_MASK) {
            let a = id(ms, e);
            prop_assert_eq!(EntryId::parse(&a.to_string()).unwrap(), a);
        }
    }

    #[test]
    fn entry_ref_parsing() {
        assert_eq!("3".parse(), Ok(EntryRef::Index(3)));
        assert_eq!("12345".parse(), Ok(EntryRef::Index(12345)));
        assert_eq!("01jb".parse(), Ok(EntryRef::ShortId("01JB".to_string())));
        assert_eq!(
            "ab12cd".parse(),
            Ok(EntryRef::ShortId("AB12CD".to_string()))
        );
        assert!("abc".parse::<EntryRef>().is_err());
        assert!("".parse::<EntryRef>().is_err());
        assert!("ab!d".parse::<EntryRef>().is_err());
        assert!("abcu".parse::<EntryRef>().is_err());
        assert!("99999999999999".parse::<EntryRef>().is_err());
    }
}
