//! Persistence of domain state: atomic writes, recovery, permissions.

#[cfg(any(test, feature = "fault-injection"))]
pub mod fault;
pub mod fs;
pub mod index;
pub mod perms;

use std::collections::{BTreeSet, HashSet};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use thiserror::Error;
use time::OffsetDateTime;

use crate::config::Caps;
use crate::domain::entry::{Content, Entry, PauseState};
use crate::domain::history::{History, HistoryError, Limits, Recorded};
use crate::domain::ids::EntryId;
use fs::{Fs, FsFile};
use index::{Index, IndexError, State};

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
    history: History,
    pause: PauseState,
    /// Content files whose unlink failed after a commit; retried by the next `apply`.
    pending_unlink: BTreeSet<EntryId>,
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
        Self::open_exclusive_at(dir, caps, fs, OffsetDateTime::now_utc())
    }

    /// [`Store::open_exclusive`] with an explicit clock (for expiry on open).
    ///
    /// After the sweep the history is brought within the caps (expired entries, then the oldest
    /// unpinned over `max_entries`), through the same durable path as `apply`.
    ///
    /// # Errors
    /// As [`Store::open_exclusive`].
    pub fn open_exclusive_at(
        dir: &Path,
        caps: Caps,
        fs: F,
        now: OffsetDateTime,
    ) -> Result<Self, StoreError> {
        if let Some(path) = perms::first_too_open(&fs, dir)? {
            return Err(StoreError::StoreTooOpen { path });
        }
        ensure_dir(&fs, dir)?;
        ensure_dir(&fs, &dir.join(CONTENT_DIR))?;
        let lock = take_lock(&dir.join(LOCK_FILE))?;

        let index = index::load(&fs, dir)?.unwrap_or_default();
        let index = sweep(&fs, dir, index)?;
        let limits = Limits {
            max_entries: caps.max_entries,
            max_pinned: caps.max_pinned,
            ttl: caps.ttl,
        };
        let mut store = Self {
            fs,
            dir: dir.to_path_buf(),
            caps,
            history: History::from_entries(limits, index.entries),
            pause: index.state.pause,
            pending_unlink: BTreeSet::new(),
            _lock: lock,
        };
        let mut enforced = store.history.clone();
        enforced.enforce_limits(now);
        let pause = store.pause;
        store.persist(enforced, pause, &[])?;
        Ok(store)
    }

    /// The persisted pause state.
    #[must_use]
    pub fn pause(&self) -> PauseState {
        self.pause
    }

    /// Number of content files whose unlink failed and is waiting for a retry.
    #[must_use]
    pub fn pending_unlinks(&self) -> usize {
        self.pending_unlink.len()
    }

    /// Retries the unlinks that failed earlier (the daemon's housekeeping calls this); returns
    /// how many are still pending. `apply` does it first on every call.
    pub fn retry_pending_unlinks(&mut self) -> usize {
        if self.pending_unlink.is_empty() {
            return 0;
        }
        let ids: Vec<EntryId> = self.pending_unlink.iter().copied().collect();
        self.pending_unlink.clear();
        self.unlink(&ids);
        self.pending_unlink.len()
    }

    /// The history aggregate (read-only view).
    #[must_use]
    pub fn history(&self) -> &History {
        &self.history
    }

    /// The rows, newest first.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        self.history.entries()
    }

    /// Reads the content of an entry.
    ///
    /// # Errors
    /// [`StoreError::Io`] (not found when absent).
    pub fn content(&self, id: &EntryId) -> Result<Vec<u8>, StoreError> {
        Ok(self.fs.read(&self.content_path(id))?)
    }

    /// Applies a batch as one transaction: the history rules run on a copy, content files for
    /// the new entries are written first, then ONE index commit, then the content of evicted,
    /// removed and expired entries is unlinked (and the directory synced). A crash between the
    /// commit and the unlinks leaves orphans that the next open sweeps.
    ///
    /// Nothing changes when a mutation is refused (`NotFound`, `PinLimit`) or I/O fails
    /// before the commit.
    ///
    /// # Errors
    /// [`StoreError::TooLarge`], [`StoreError::History`], [`StoreError::Io`].
    pub fn apply(
        &mut self,
        batch: Vec<Mutation>,
        now: OffsetDateTime,
    ) -> Result<Vec<Outcome>, StoreError> {
        self.retry_pending_unlinks();
        for m in &batch {
            if let Mutation::RecordCopy(c) = m
                && u64::from(c.size()) > self.caps.max_entry_bytes
            {
                return Err(StoreError::TooLarge {
                    size: c.len(),
                    max: self.caps.max_entry_bytes,
                });
            }
        }

        let mut history = self.history.clone();
        let mut pause = self.pause;
        let mut pending: Vec<(EntryId, &Content)> = Vec::new();
        let mut outcomes = Vec::with_capacity(batch.len());
        for m in &batch {
            outcomes.push(match m {
                Mutation::RecordCopy(c) => {
                    let id = EntryId::new(now);
                    let r = history.record(c, id, now);
                    if matches!(r, Recorded::Inserted { .. }) {
                        pending.push((id, c));
                    }
                    Outcome::Recorded(r)
                }
                Mutation::Touch(id) => {
                    history.touch(id, now)?;
                    Outcome::Touched
                }
                Mutation::Pin(id) => Outcome::Pinned(history.pin(id)?),
                Mutation::Unpin(id) => Outcome::Unpinned(history.unpin(id)?),
                Mutation::Remove(id) => {
                    history.remove(id)?;
                    Outcome::Removed
                }
                Mutation::Clear => Outcome::Cleared(history.clear()),
                Mutation::ClearAll => Outcome::Cleared(history.clear_all()),
                Mutation::Expire => Outcome::Expired(history.expire(now)),
                Mutation::SetPause(p) => {
                    pause = *p;
                    Outcome::PauseSet
                }
            });
        }

        // Entries inserted and evicted within the same batch never reach the disk.
        let kept: HashSet<EntryId> = history.entries().iter().map(|e| e.id).collect();
        let to_write: Vec<_> = pending.iter().filter(|(id, _)| kept.contains(id)).collect();
        let mut written: Vec<EntryId> = Vec::new();
        for (id, content) in &to_write {
            written.push(*id);
            if let Err(e) = self.write_content(id, content.as_bytes()) {
                self.discard(&written);
                return Err(e.into());
            }
        }

        self.persist(history, pause, &written)?;
        tracing::debug!(
            mutations = batch.len(),
            written = written.len(),
            "batch applied"
        );
        Ok(outcomes)
    }

    /// Makes `history` and `pause` the stored state: one index commit if anything changed
    /// (discarding `written` content files if it fails), then the unlink of every entry that
    /// is gone and a sync of the content directory.
    fn persist(
        &mut self,
        history: History,
        pause: PauseState,
        written: &[EntryId],
    ) -> Result<(), StoreError> {
        let kept: HashSet<EntryId> = history.entries().iter().map(|e| e.id).collect();
        let to_unlink: Vec<EntryId> = self
            .history
            .entries()
            .iter()
            .map(|e| e.id)
            .filter(|id| !kept.contains(id))
            .collect();
        if history != self.history || pause != self.pause {
            let index = Index {
                version: index::VERSION,
                entries: history.entries().to_vec(),
                state: State { pause },
            };
            if let Err(e) = index::commit(&self.fs, &self.dir, &index) {
                self.discard(written);
                return Err(e.into());
            }
            self.history = history;
            self.pause = pause;
        }
        self.unlink(&to_unlink);
        Ok(())
    }

    /// Convenience: records one copy (a batch of one) and returns the entry's id.
    ///
    /// # Errors
    /// As [`Store::apply`].
    pub fn insert(
        &mut self,
        content: &Content,
        now: OffsetDateTime,
    ) -> Result<EntryId, StoreError> {
        let mut out = self.apply(vec![Mutation::RecordCopy(content.clone())], now)?;
        match out.pop() {
            Some(Outcome::Recorded(Recorded::Inserted { id, .. } | Recorded::Refreshed { id })) => {
                Ok(id)
            }
            _ => Err(StoreError::Io(io::Error::other(
                "record produced no outcome",
            ))),
        }
    }

    /// Best-effort removal of content files that were written but never committed.
    fn discard(&self, ids: &[EntryId]) {
        for id in ids {
            let _ = self.fs.remove_file(&self.content_path(id));
        }
    }

    /// After the commit: unlink the content of removed entries and sync the directory. A
    /// failure keeps the id in `pending_unlink` (retried later) and leaves at worst an orphan
    /// for the next open's sweep, so it is logged, not returned.
    fn unlink(&mut self, ids: &[EntryId]) {
        let mut done = false;
        for id in ids {
            match self.fs.remove_file(&self.content_path(id)) {
                Ok(()) => done = true,
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => {
                    tracing::warn!(id = %id, kind = ?e.kind(), "could not unlink content; will retry");
                    self.pending_unlink.insert(*id);
                }
            }
        }
        if done && let Err(e) = self.fs.sync_dir(&self.dir.join(CONTENT_DIR)) {
            tracing::warn!(kind = ?e.kind(), "could not sync the content directory");
        }
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
}

/// Opening recovery, both ways: unlinks content files without a row (and temp leftovers), and
/// drops rows whose content is missing or torn (size differs from the row), recommitting.
fn sweep<F: Fs>(fs: &F, dir: &Path, index: Index) -> Result<Index, StoreError> {
    let _ = fs.remove_file(&dir.join(index::INDEX_TMP));
    let cdir = dir.join(CONTENT_DIR);
    let mut present = HashSet::new();
    let mut removed = 0usize;
    for p in fs.read_dir(&cdir)? {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        let row = EntryId::parse(name)
            .ok()
            .and_then(|id| index.entries.iter().find(|e| e.id == id));
        match row {
            // A torn file (power loss) is as good as none: drop the row and the file.
            Some(e) if fs.len(&p)? != u64::from(e.size) => {
                fs.remove_file(&p)?;
                removed += 1;
            }
            Some(e) => {
                present.insert(e.id);
            }
            None if name.starts_with(TMP_PREFIX) || EntryId::parse(name).is_ok() => {
                fs.remove_file(&p)?;
                removed += 1;
            }
            None => {} // not ours: leave it alone
        }
    }
    if removed > 0 {
        fs.sync_dir(&cdir)?;
    }
    let mut next = index;
    let before = next.entries.len();
    next.entries.retain(|e| present.contains(&e.id));
    let dropped = before - next.entries.len();
    if dropped > 0 {
        index::commit(fs, dir, &next)?;
    }
    if removed > 0 || dropped > 0 {
        tracing::warn!(
            orphan_files = removed,
            dropped_rows = dropped,
            "store recovery sweep"
        );
    }
    Ok(next)
}

const TMP_PREFIX: &str = ".tmp-";

/// Creates `path` when missing, and syncs the parent of every directory that was created
/// (`create_dir_all` may create several), so each new directory entry is durable.
fn ensure_dir<F: Fs>(fs: &F, path: &Path) -> io::Result<()> {
    let mut missing: Vec<&Path> = Vec::new();
    for p in path.ancestors().filter(|p| !p.as_os_str().is_empty()) {
        match fs.mode(p) {
            Ok(_) => break,
            Err(e) if e.kind() == io::ErrorKind::NotFound => missing.push(p),
            Err(e) => return Err(e),
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    fs.create_dir_all(path, perms::DIR_MODE)?;
    for m in missing {
        let parent = m.parent().filter(|p| !p.as_os_str().is_empty());
        fs.sync_dir(parent.unwrap_or(Path::new(".")))?;
    }
    Ok(())
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
