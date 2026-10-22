//! Persistence of domain state: atomic writes, recovery, permissions.

#[cfg(any(test, feature = "fault-injection"))]
pub mod fault;
pub mod fs;
pub mod index;
pub mod perms;

use std::collections::HashSet;
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use thiserror::Error;
use time::OffsetDateTime;

use crate::config::Caps;
use crate::domain::entry::{Content, Entry, PauseState};
use crate::domain::history::{HistoryError, Recorded};
use crate::domain::ids::EntryId;
use fs::{Fs, FsFile};
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
    /// A history command was refused (not found, pin limit).
    #[error(transparent)]
    History(#[from] HistoryError),
    /// An I/O error.
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
}

/// One change to the store; a batch of them is one transaction (one index commit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutation {
    /// A clipboard copy: new entry, or refresh of a known one.
    RecordCopy(Content),
    /// Restore: move to the top.
    Touch(EntryId),
    /// Pin (idempotent).
    Pin(EntryId),
    /// Unpin (idempotent).
    Unpin(EntryId),
    /// Delete one entry.
    Remove(EntryId),
    /// Delete every unpinned entry.
    Clear,
    /// Delete every entry.
    ClearAll,
    /// Delete the unpinned entries past their ttl.
    Expire,
    /// Change the persisted pause state.
    SetPause(PauseState),
}

/// What one [`Mutation`] did, in batch order. Ids only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// See [`Recorded`].
    Recorded(Recorded),
    /// The entry moved to the top.
    Touched,
    /// Whether the pin changed anything.
    Pinned(bool),
    /// Whether the unpin changed anything.
    Unpinned(bool),
    /// The entry was deleted.
    Removed,
    /// Ids deleted by `Clear`.
    Cleared(Vec<EntryId>),
    /// Ids deleted by `Expire`.
    Expired(Vec<EntryId>),
    /// The pause state was stored.
    PauseSet,
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
    /// Nothing is written when the store is too open. After locking, orphans are swept both
    /// ways: content files without a row are unlinked, rows without content are dropped.
    ///
    /// # Errors
    /// [`StoreError::StoreTooOpen`], [`StoreError::Locked`], [`StoreError::UnknownVersion`],
    /// [`StoreError::Corrupt`], [`StoreError::Io`].
    pub fn open_exclusive(dir: &Path, caps: Caps, fs: F) -> Result<Self, StoreError> {
        if let Some(path) = perms::first_too_open(&fs, dir)? {
            return Err(StoreError::StoreTooOpen { path });
        }
        ensure_dir(&fs, dir)?;
        ensure_dir(&fs, &dir.join(CONTENT_DIR))?;
        let lock = take_lock(&dir.join(LOCK_FILE))?;

        let index = index::load(&fs, dir)?.unwrap_or_default();
        let mut store = Self {
            fs,
            dir: dir.to_path_buf(),
            caps,
            index,
            _lock: lock,
        };
        store.sweep()?;
        Ok(store)
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
        Ok(self.fs.read(&self.content_path(id))?)
    }

    /// Applies a batch as one transaction. NOT YET IMPLEMENTED (T025).
    ///
    /// # Errors
    /// [`StoreError`].
    pub fn apply(
        &mut self,
        batch: Vec<Mutation>,
        now: OffsetDateTime,
    ) -> Result<Vec<Outcome>, StoreError> {
        unimplemented!("T025")
    }

    /// MINIMAL, TEMPORARY (T022): writes the content file, then commits the index with the new
    /// row prepended. No caps, de-dup or expiry: the `History` card (T024/T025) replaces this
    /// with `apply`.
    ///
    /// # Errors
    /// [`StoreError::TooLarge`], [`StoreError::Io`].
    pub fn insert(
        &mut self,
        content: &Content,
        now: OffsetDateTime,
    ) -> Result<EntryId, StoreError> {
        if u64::try_from(content.len()).map_or(true, |n| n > self.caps.max_entry_bytes) {
            return Err(StoreError::TooLarge {
                size: content.len(),
                max: self.caps.max_entry_bytes,
            });
        }
        let too_large = || StoreError::TooLarge {
            size: content.len(),
            max: self.caps.max_entry_bytes,
        };
        let size = u32::try_from(content.len()).map_err(|_| too_large())?;
        let id = EntryId::new(now);
        if let Err(e) = self.write_content(&id, content.as_bytes()) {
            // Not acknowledged: leave no file behind (a rename may already have happened).
            let _ = self.fs.remove_file(&self.content_path(&id));
            return Err(e.into());
        }
        let row = Entry {
            id,
            hash: content.hash(),
            size,
            created: now,
            last_copied: now,
            pinned: false,
            preview: content.preview(),
        };
        let mut next = self.index.clone();
        next.entries.insert(0, row);
        if let Err(e) = index::commit(&self.fs, &self.dir, &next) {
            // Not acknowledged: drop the unreferenced file now (the sweep covers a crash).
            let _ = self.fs.remove_file(&self.content_path(&id));
            return Err(e.into());
        }
        self.index = next;
        tracing::debug!(id = %id, size = content.len(), "entry stored");
        Ok(id)
    }

    fn content_path(&self, id: &EntryId) -> PathBuf {
        self.dir.join(CONTENT_DIR).join(id.to_string())
    }

    /// Content files are written once: temp, fsync, rename, fsync of the directory.
    fn write_content(&self, id: &EntryId, bytes: &[u8]) -> io::Result<()> {
        let cdir = self.dir.join(CONTENT_DIR);
        let tmp = cdir.join(format!("{TMP_PREFIX}{id}"));
        let result = (|| {
            let mut f = self.fs.create_new(&tmp, perms::FILE_MODE)?;
            f.write_all(bytes)?;
            f.sync_all()?;
            drop(f);
            self.fs.rename(&tmp, &self.content_path(id))?;
            self.fs.sync_dir(&cdir)
        })();
        if result.is_err() {
            let _ = self.fs.remove_file(&tmp);
        }
        result
    }

    fn sweep(&mut self) -> Result<(), StoreError> {
        let _ = self.fs.remove_file(&self.dir.join(index::INDEX_TMP));
        let cdir = self.dir.join(CONTENT_DIR);
        let mut present = HashSet::new();
        let mut removed = 0usize;
        for p in self.fs.read_dir(&cdir)? {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            let row = EntryId::parse(name)
                .ok()
                .and_then(|id| self.index.entries.iter().find(|e| e.id == id));
            match row {
                // A torn file (power loss) is as good as none: drop the row and the file.
                Some(e) if self.fs.len(&p)? != u64::from(e.size) => {
                    self.fs.remove_file(&p)?;
                    removed += 1;
                }
                Some(e) => {
                    present.insert(e.id);
                }
                None if name.starts_with(TMP_PREFIX) || EntryId::parse(name).is_ok() => {
                    self.fs.remove_file(&p)?;
                    removed += 1;
                }
                None => {} // not ours: leave it alone
            }
        }
        if removed > 0 {
            self.fs.sync_dir(&cdir)?;
        }
        let before = self.index.entries.len();
        let mut next = self.index.clone();
        next.entries.retain(|e| present.contains(&e.id));
        let dropped = before - next.entries.len();
        if dropped > 0 {
            index::commit(&self.fs, &self.dir, &next)?;
            self.index = next;
        }
        if removed > 0 || dropped > 0 {
            tracing::warn!(
                orphan_files = removed,
                dropped_rows = dropped,
                "store recovery sweep"
            );
        }
        Ok(())
    }
}

const TMP_PREFIX: &str = ".tmp-";

/// Creates `path` when missing and syncs its parent, so the new directory entry is durable.
fn ensure_dir<F: Fs>(fs: &F, path: &Path) -> io::Result<()> {
    match fs.mode(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            fs.create_dir_all(path, perms::DIR_MODE)?;
            match path.parent() {
                Some(p) if p.as_os_str().is_empty() => fs.sync_dir(Path::new(".")),
                Some(parent) => fs.sync_dir(parent),
                None => Ok(()),
            }
        }
        Err(e) => Err(e),
    }
}

/// Non-blocking `flock(LOCK_EX)` on the lock file; released when the handle is dropped.
fn take_lock(path: &Path) -> Result<std::fs::File, StoreError> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .mode(perms::FILE_MODE)
        .open(path)?;
    match rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => Ok(file),
        Err(rustix::io::Errno::WOULDBLOCK) => Err(StoreError::Locked),
        Err(e) => Err(io::Error::from(e).into()),
    }
}
