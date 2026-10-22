//! Index v1: the serde types and `load`/`commit` through the `Fs` trait.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::fs::{Fs, FsFile};
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
        Self { version: VERSION, entries: Vec::new(), state: State::default() }
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
    unimplemented!("T023")
}

/// Atomically replaces `<dir>/index.json`: temp (0600), write, fsync, rename, fsync dir.
///
/// # Errors
/// [`IndexError::Io`]; on failure the previous index is intact.
pub fn commit<F: Fs>(fs: &F, dir: &Path, index: &Index) -> Result<(), IndexError> {
    unimplemented!("T023")
}
