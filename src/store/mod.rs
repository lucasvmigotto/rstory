//! Persistence of domain state: atomic writes, recovery, permissions.

pub mod fs;
pub mod index;
pub mod perms;

use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use time::OffsetDateTime;

use crate::config::Caps;
use crate::domain::entry::{Content, Entry};
use crate::domain::ids::EntryId;
use fs::Fs;
use index::{Index, IndexError};

/// Name of the lock file.
pub const LOCK_FILE: &str = "store.lock";
/// Name of the content directory.
pub const CONTENT_DIR: &str = "content";

/// Why a store operation failed.
#[derive(Debug, Error)]
pub enum StoreError {
    /// The directory or a file in it is accessible to group or others.
    #[error("store is too open: {path}")]
    StoreTooOpen {
        /// The offending path.
        path: PathBuf,
    },
    /// The index declares an unknown version.
    #[error("unknown index version {0}")]
    UnknownVersion(u32),
    /// Another handle holds the exclusive lock.
    #[error("the store is locked by another process")]
    Locked,
    /// The content is larger than `max_entry_bytes`.
    #[error("content is {size} bytes; the limit is {max}")]
    TooLarge {
        /// Size of the rejected content.
        size: usize,
        /// The limit.
        max: u64,
    },
    /// `index.json` is not a valid index.
    #[error("index.json is corrupt at line {line}, column {column}")]
    Corrupt {
        /// 1-based line.
        line: usize,
        /// 1-based column.
        column: usize,
    },
    /// An I/O error.
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
}

impl From<IndexError> for StoreError {
    fn from(e: IndexError) -> Self {
        match e {
            IndexError::UnknownVersion(v) => Self::UnknownVersion(v),
            IndexError::Corrupt { line, column } => Self::Corrupt { line, column },
            IndexError::Io(e) => Self::Io(e),
        }
    }
}

/// An exclusively locked store (the daemon's handle).
#[derive(Debug)]
pub struct Store<F: Fs> {
    fs: F,
    dir: PathBuf,
    caps: Caps,
    index: Index,
    _lock: std::fs::File,
}

impl<F: Fs> Store<F> {
    /// Opens (creating if needed) the store at `dir` and holds `LOCK_EX` while alive.
    ///
    /// # Errors
    /// [`StoreError::StoreTooOpen`], [`StoreError::Locked`], [`StoreError::UnknownVersion`],
    /// [`StoreError::Corrupt`], [`StoreError::Io`].
    pub fn open_exclusive(dir: &Path, caps: Caps, fs: F) -> Result<Self, StoreError> {
        unimplemented!("T022")
    }

    /// The rows, newest first.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.index.entries
    }

    /// Reads the content of an entry.
    ///
    /// # Errors
    /// [`StoreError::Io`] (not found when absent).
    pub fn content(&self, id: &EntryId) -> Result<Vec<u8>, StoreError> {
        unimplemented!("T022")
    }

    /// MINIMAL, TEMPORARY (T022): writes the content file, then commits the index with the new
    /// row prepended. No caps, de-dup or expiry: the `History` card (T024/T025) replaces this
    /// with `apply`.
    ///
    /// # Errors
    /// [`StoreError::TooLarge`], [`StoreError::Io`].
    pub fn insert(&mut self, content: &Content, now: OffsetDateTime) -> Result<EntryId, StoreError> {
        unimplemented!("T022")
    }
}
