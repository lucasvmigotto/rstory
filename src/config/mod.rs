//! `config.toml` loading, validation and defaults (caps bounds).

use std::time::Duration;

use serde::Deserialize;
use thiserror::Error;

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;

/// History caps, validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps {
    /// Maximum number of entries, pinned ones included (2 to 500, default 20).
    pub max_entries: u32,
    /// Maximum number of pinned entries (0 to `max_entries` - 1, default 10).
    pub max_pinned: u32,
    /// Maximum size of one entry in bytes (1024 to 1 MiB, default 1 MiB).
    pub max_entry_bytes: u64,
    /// Time to live of unpinned entries (1 hour to 90 days, default 7 days).
    pub ttl: Duration,
}

impl Caps {
    /// Lowest and highest accepted `max_entries`.
    pub const MAX_ENTRIES_RANGE: (u32, u32) = (2, 500);
    /// Lowest and highest accepted `max_entry_bytes`.
    pub const MAX_ENTRY_BYTES_RANGE: (u64, u64) = (1024, 1_048_576);
    /// Shortest and longest accepted `ttl`.
    pub const TTL_RANGE: (Duration, Duration) =
        (Duration::from_secs(HOUR), Duration::from_secs(90 * DAY));

    const DEFAULT_MAX_ENTRIES: u32 = 20;
    const DEFAULT_MAX_PINNED: u32 = 10;

    /// Parses and validates the caps from the text of a `config.toml`.
    ///
    /// Unknown keys are ignored (other specs add their own). Missing keys take their defaults.
    ///
    /// # Errors
    /// [`ConfigError`] naming the offending key and the allowed range.
    pub fn from_toml(text: &str) -> Result<Self, ConfigError> {
        let raw: Raw =
            toml::from_str(text).map_err(|e| ConfigError::Syntax(e.message().to_string()))?;
        Self::from_raw(raw)
    }

    fn from_raw(raw: Raw) -> Result<Self, ConfigError> {
        let (lo, hi) = Self::MAX_ENTRIES_RANGE;
        let max_entries = raw.max_entries.unwrap_or(Self::DEFAULT_MAX_ENTRIES);
        if !(lo..=hi).contains(&max_entries) {
            return Err(range_error("max_entries", max_entries, lo, hi));
        }

        let max_pinned = match raw.max_pinned {
            Some(p) => {
                if p >= max_entries {
                    return Err(ConfigError::OutOfRange {
                        key: "max_pinned",
                        value: p.to_string(),
                        allowed: format!("0 to {} (max_entries - 1)", max_entries - 1),
                    });
                }
                p
            }
            None => Self::DEFAULT_MAX_PINNED.min(max_entries - 1),
        };

        let (lo, hi) = Self::MAX_ENTRY_BYTES_RANGE;
        let max_entry_bytes = raw.max_entry_bytes.unwrap_or(hi);
        if !(lo..=hi).contains(&max_entry_bytes) {
            return Err(range_error("max_entry_bytes", max_entry_bytes, lo, hi));
        }

        let ttl = match raw.ttl {
            Some(s) => parse_duration(&s).map_err(|reason| ConfigError::BadValue {
                key: "ttl",
                value: s.clone(),
                reason,
            })?,
            None => Duration::from_secs(7 * DAY),
        };
        let (lo, hi) = Self::TTL_RANGE;
        if ttl < lo || ttl > hi {
            return Err(ConfigError::OutOfRange {
                key: "ttl",
                value: format!("{}s", ttl.as_secs()),
                allowed: "1h to 90d".to_string(),
            });
        }

        Ok(Self {
            max_entries,
            max_pinned,
            max_entry_bytes,
            ttl,
        })
    }
}

impl Default for Caps {
    fn default() -> Self {
        Self {
            max_entries: Self::DEFAULT_MAX_ENTRIES,
            max_pinned: Self::DEFAULT_MAX_PINNED,
            max_entry_bytes: Self::MAX_ENTRY_BYTES_RANGE.1,
            ttl: Duration::from_secs(7 * DAY),
        }
    }
}

fn range_error(
    key: &'static str,
    value: impl ToString,
    lo: impl ToString,
    hi: impl ToString,
) -> ConfigError {
    ConfigError::OutOfRange {
        key,
        value: value.to_string(),
        allowed: format!("{} to {}", lo.to_string(), hi.to_string()),
    }
}

#[derive(Debug, Deserialize)]
struct Raw {
    max_entries: Option<u32>,
    max_pinned: Option<u32>,
    max_entry_bytes: Option<u64>,
    ttl: Option<String>,
}

/// Why a configuration was rejected.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    /// The text is not valid TOML, or a key has the wrong type.
    #[error("config.toml: {0}")]
    Syntax(String),
    /// A value is outside its allowed range.
    #[error("{key} = {value} is out of range; allowed: {allowed}")]
    OutOfRange {
        /// The configuration key.
        key: &'static str,
        /// The offending value.
        value: String,
        /// The allowed range, for humans.
        allowed: String,
    },
    /// A value could not be understood.
    #[error("{key} = {value:?} is invalid: {reason}")]
    BadValue {
        /// The configuration key.
        key: &'static str,
        /// The offending value.
        value: String,
        /// What is wrong with it.
        reason: String,
    },
}

/// Parses a human duration: an integer followed by `s`, `m`, `h` or `d` ("7d", "1h", "90m").
fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    let split = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let (num, unit) = s.split_at(split);
    let n: u64 = num
        .parse()
        .map_err(|_| "expected a number followed by s, m, h or d (e.g. \"7d\")".to_string())?;
    let mult = match unit {
        "s" => 1,
        "m" => MINUTE,
        "h" => HOUR,
        "d" => DAY,
        _ => return Err("unit must be one of s, m, h, d (e.g. \"7d\")".to_string()),
    };
    n.checked_mul(mult)
        .map(Duration::from_secs)
        .ok_or_else(|| "too large".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_of(e: &ConfigError) -> &'static str {
        match e {
            ConfigError::OutOfRange { key, .. } | ConfigError::BadValue { key, .. } => key,
            ConfigError::Syntax(_) => "",
        }
    }

    #[test]
    fn defaults_when_empty() {
        let c = Caps::from_toml("").unwrap();
        assert_eq!(c, Caps::default());
        assert_eq!(c.max_entries, 20);
        assert_eq!(c.max_pinned, 10);
        assert_eq!(c.max_entry_bytes, 1_048_576);
        assert_eq!(c.ttl, Duration::from_secs(7 * 86_400));
    }

    #[test]
    fn parses_all_keys_and_ignores_unknown() {
        let c = Caps::from_toml("max_entries = 50\nmax_pinned = 5\nmax_entry_bytes = 2048\nttl = \"36h\"\npicker_command = \"x\"").unwrap();
        assert_eq!(
            (c.max_entries, c.max_pinned, c.max_entry_bytes),
            (50, 5, 2048)
        );
        assert_eq!(c.ttl, Duration::from_secs(36 * 3600));
    }

    #[test]
    fn max_entries_bounds() {
        for ok in [2, 500] {
            assert!(Caps::from_toml(&format!("max_entries = {ok}\nmax_pinned = 1")).is_ok());
        }
        for bad in [0, 1, 501] {
            let e = Caps::from_toml(&format!("max_entries = {bad}")).unwrap_err();
            assert_eq!(key_of(&e), "max_entries");
            assert!(e.to_string().contains("2 to 500"), "{e}");
        }
    }

    #[test]
    fn max_pinned_bounds_and_must_be_below_entries() {
        assert!(Caps::from_toml("max_pinned = 0").is_ok());
        assert!(Caps::from_toml("max_entries = 20\nmax_pinned = 19").is_ok());
        let e = Caps::from_toml("max_entries = 20\nmax_pinned = 20").unwrap_err();
        assert_eq!(key_of(&e), "max_pinned");
        assert!(e.to_string().contains("0 to 19"), "{e}");
        assert!(Caps::from_toml("max_entries = 2\nmax_pinned = 2").is_err());
        assert!(Caps::from_toml("max_pinned = -1").is_err());
    }

    #[test]
    fn default_pin_cap_shrinks_with_small_entry_cap() {
        let c = Caps::from_toml("max_entries = 5").unwrap();
        assert_eq!(c.max_pinned, 4);
        assert_eq!(Caps::from_toml("max_entries = 2").unwrap().max_pinned, 1);
    }

    #[test]
    fn max_entry_bytes_bounds() {
        for ok in [1024, 1_048_576] {
            assert!(Caps::from_toml(&format!("max_entry_bytes = {ok}")).is_ok());
        }
        for bad in [1023, 1_048_577] {
            let e = Caps::from_toml(&format!("max_entry_bytes = {bad}")).unwrap_err();
            assert_eq!(key_of(&e), "max_entry_bytes");
            assert!(e.to_string().contains("1024 to 1048576"), "{e}");
        }
    }

    #[test]
    fn ttl_bounds() {
        for ok in ["1h", "60m", "3600s", "90d", "2160h"] {
            assert!(Caps::from_toml(&format!("ttl = \"{ok}\"")).is_ok(), "{ok}");
        }
        for bad in ["59m", "3599s", "91d", "0d", "2161h"] {
            let e = Caps::from_toml(&format!("ttl = \"{bad}\"")).unwrap_err();
            assert_eq!(key_of(&e), "ttl", "{bad}");
            assert!(e.to_string().contains("1h to 90d"), "{e}");
        }
    }

    #[test]
    fn ttl_rejects_garbage() {
        for bad in ["", "d", "7", "7w", "-1d", "1.5h", "7 d x"] {
            let e = Caps::from_toml(&format!("ttl = \"{bad}\"")).unwrap_err();
            assert!(
                matches!(e, ConfigError::BadValue { key: "ttl", .. }),
                "{bad}: {e}"
            );
        }
    }

    #[test]
    fn syntax_and_type_errors() {
        assert!(matches!(
            Caps::from_toml("max_entries = "),
            Err(ConfigError::Syntax(_))
        ));
        assert!(matches!(
            Caps::from_toml("max_entries = \"x\""),
            Err(ConfigError::Syntax(_))
        ));
    }
}
