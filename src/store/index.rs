//! Index v1: the serde types and `load`/`commit` through the `Fs` trait.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::fs::{Fs, FsFile};
use super::perms;
use crate::domain::entry::{Entry, PauseState};

/// The only index version this build understands.
pub const VERSION: u32 = 1;
/// File name of the index inside the store directory.
pub const INDEX_FILE: &str = "index.json";
/// Temporary name used while committing.
pub const INDEX_TMP: &str = "index.json.tmp";

/// Persisted state besides the entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct State {
    /// Pause state.
    pub pause: PauseState,
}

/// The content of `index.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    /// Format version; must be [`VERSION`].
    pub version: u32,
    /// Rows, newest first.
    pub entries: Vec<Entry>,
    /// Other persisted state.
    #[serde(default)]
    pub state: State,
}

impl Default for Index {
    fn default() -> Self {
        Self {
            version: VERSION,
            entries: Vec::new(),
            state: State::default(),
        }
    }
}

/// Why the index could not be loaded or committed.
#[derive(Debug, Error)]
pub enum IndexError {
    /// The file declares a version this build does not know. It is left untouched.
    #[error("unknown index version {0}")]
    UnknownVersion(u32),
    /// The file is not a valid index. Position only: the text may be clipboard-derived.
    #[error("index.json is corrupt at line {line}, column {column}")]
    Corrupt {
        /// 1-based line.
        line: usize,
        /// 1-based column.
        column: usize,
    },
    /// An I/O error.
    #[error("index i/o error: {0}")]
    Io(#[from] io::Error),
}

/// Loads `<dir>/index.json`; `None` when it does not exist.
///
/// # Errors
/// [`IndexError`]. An unknown version is refused before anything else is parsed.
pub fn load<F: Fs>(fs: &F, dir: &Path) -> Result<Option<Index>, IndexError> {
    let bytes = match fs.read(&dir.join(INDEX_FILE)) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    // Look at the version alone first: a newer format must be refused, not misread.
    let head: Head = serde_json::from_slice(&bytes).map_err(corrupt)?;
    if head.version != VERSION {
        return Err(IndexError::UnknownVersion(head.version));
    }
    serde_json::from_slice(&bytes).map(Some).map_err(corrupt)
}

#[derive(Deserialize)]
struct Head {
    version: u32,
}

// serde messages can quote values (previews are content-derived): keep only the position.
fn corrupt(e: serde_json::Error) -> IndexError {
    IndexError::Corrupt {
        line: e.line(),
        column: e.column(),
    }
}

/// Atomically replaces `<dir>/index.json`: temp (0600), write, fsync, rename, fsync dir.
///
/// # Errors
/// [`IndexError::Io`]; on failure the previous index is intact.
pub fn commit<F: Fs>(fs: &F, dir: &Path, index: &Index) -> Result<(), IndexError> {
    let bytes = serde_json::to_vec(index).map_err(|e| io::Error::other(e.to_string()))?;
    let tmp = dir.join(INDEX_TMP);
    let result = (|| {
        let mut f = fs.create_new(&tmp, perms::FILE_MODE)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
        drop(f);
        fs.rename(&tmp, &dir.join(INDEX_FILE))?;
        fs.sync_dir(dir)
    })();
    if result.is_err() {
        // Best effort; after a real crash the open-time sweep removes it.
        let _ = fs.remove_file(&tmp);
    }
    result.map_err(IndexError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entry::Content;
    use crate::domain::ids::EntryId;
    use crate::store::fs::{Fault, FaultFs, RealFs};
    use time::macros::datetime;

    fn sample() -> Index {
        let c = Content::new(b"hello".to_vec(), 100).unwrap();
        let t = datetime!(2026-10-22 12:00:00 UTC);
        Index {
            version: VERSION,
            entries: vec![Entry {
                id: EntryId::from_parts(5, 6),
                hash: c.hash(),
                size: 5,
                created: t,
                last_copied: t,
                pinned: true,
                preview: c.preview(),
            }],
            state: State {
                pause: PauseState::Recording,
            },
        }
    }

    #[test]
    fn missing_index_is_none() {
        let t = tempfile::tempdir().unwrap();
        assert!(load(&RealFs, t.path()).unwrap().is_none());
    }

    #[test]
    fn commit_then_load_roundtrips_and_leaves_no_temp() {
        let t = tempfile::tempdir().unwrap();
        commit(&RealFs, t.path(), &sample()).unwrap();
        assert_eq!(load(&RealFs, t.path()).unwrap().unwrap(), sample());
        assert_eq!(RealFs.read_dir(t.path()).unwrap().len(), 1);
        assert_eq!(RealFs.mode(&t.path().join(INDEX_FILE)).unwrap(), 0o600);
    }

    #[test]
    fn json_shape_is_version_entries_state() {
        let v = serde_json::to_value(sample()).unwrap();
        assert_eq!(v["version"], 1);
        assert_eq!(v["state"]["pause"]["kind"], "recording");
        assert!(v["entries"][0]["pinned"].as_bool().unwrap());
    }

    #[test]
    fn unknown_version_is_refused_without_parsing_the_rest() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(
            t.path().join(INDEX_FILE),
            br#"{"version":9,"entries":"weird"}"#,
        )
        .unwrap();
        assert!(matches!(
            load(&RealFs, t.path()),
            Err(IndexError::UnknownVersion(9))
        ));
    }

    #[test]
    fn corrupt_error_has_a_position_and_no_content() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(
            t.path().join(INDEX_FILE),
            br#"{"version":1,"entries":[{"id":"SECRET-CANARY"}]}"#,
        )
        .unwrap();
        let e = load(&RealFs, t.path()).unwrap_err();
        assert!(matches!(e, IndexError::Corrupt { .. }));
        assert!(!e.to_string().contains("CANARY"));
    }

    #[test]
    fn failed_commit_keeps_the_previous_index() {
        let t = tempfile::tempdir().unwrap();
        commit(&RealFs, t.path(), &Index::default()).unwrap();
        for n in 0..5 {
            let fs = FaultFs::new(n, Fault::Fail);
            assert!(commit(&fs, t.path(), &sample()).is_err() || n == 4, "n={n}");
            let loaded = load(&RealFs, t.path()).unwrap().unwrap();
            assert!(loaded == Index::default() || loaded == sample());
            assert!(!t.path().join(INDEX_TMP).exists(), "temp cleaned, n={n}");
            commit(&RealFs, t.path(), &Index::default()).unwrap();
        }
    }

    #[test]
    fn stale_temp_does_not_block_commit() {
        let t = tempfile::tempdir().unwrap();
        let tmp = t.path().join(INDEX_TMP);
        let mut f = RealFs.create_new(&tmp, 0o600).unwrap();
        f.write_all(b"leftover from a failed cleanup").unwrap();
        drop(f);
        commit(&RealFs, t.path(), &sample()).unwrap();
        assert_eq!(load(&RealFs, t.path()).unwrap().unwrap(), sample());
        assert!(!tmp.exists());
    }
}
