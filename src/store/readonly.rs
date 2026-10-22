//! The read-only handle clients use while the daemon is down (BR14).
//!
//! Locking: the handle holds `LOCK_SH` on `store.lock` for as long as it lives, taken
//! non-blocking. While the daemon holds `LOCK_EX` the open fails with
//! [`StoreError::DaemonRunning`] (clients should use the socket). The converse also holds: while
//! any reader is alive the daemon's `open_exclusive` reports [`StoreError::Locked`], so a
//! daemon that starts while a client is reading should say "a client is reading; retry". A
//! short retry loop is the daemon's job (spec 004); readers are meant to be short-lived.
//!
//! The handle never writes: it creates no file or directory, never sweeps, never recommits.
//! Orphan content files are ignored, and rows whose content is missing (or torn) are hidden
//! from the listing. There are no mutating methods, by construction.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use super::fs::{Fs, RealFs};
use super::{LOCK_FILE, StoreError, content_file, index, perms};
use crate::config::Caps;
use crate::domain::entry::{Entry, PauseState};
use crate::domain::history::{History, Limits, ResolveError};
use crate::domain::ids::{EntryId, EntryRef, ShortId};

/// A shared-locked, read-only view of the store.
#[derive(Debug)]
pub struct ReadOnlyStore<F: Fs> {
    fs: F,
    dir: PathBuf,
    history: History,
    pause: PauseState,
    perms_ok: bool,
    _lock: Option<File>,
}

impl ReadOnlyStore<RealFs> {
    /// Opens the store at `dir` read-only on the real filesystem. The pin cap used by
    /// [`ReadOnlyStore::pins_over_cap`] is the default one; use [`ReadOnlyStore::open_with`]
    /// to pass the configured caps.
    ///
    /// # Errors
    /// See [`ReadOnlyStore::open_with`].
    pub fn open(dir: &Path) -> Result<Self, StoreError> {
        Self::open_with(dir, Caps::default(), RealFs)
    }
}

impl<F: Fs> ReadOnlyStore<F> {
    /// Opens the store at `dir` read-only. A missing directory is an empty history.
    ///
    /// Too-open permissions only warn: the store stays readable and
    /// [`ReadOnlyStore::perms_ok`] is false so `doctor` can say it.
    ///
    /// # Errors
    /// [`StoreError::DaemonRunning`] when the exclusive lock is held,
    /// [`StoreError::UnknownVersion`], [`StoreError::Corrupt`], [`StoreError::Io`].
    pub fn open_with(dir: &Path, caps: Caps, fs: F) -> Result<Self, StoreError> {
        let perms_ok = perms::first_too_open(&fs, dir)?.is_none();
        if !perms_ok {
            tracing::warn!("store permissions are too open (read-only handle)");
        }
        let lock = take_shared_lock(&dir.join(LOCK_FILE))?;

        let index = index::load(&fs, dir)?.unwrap_or_default();
        let mut visible = Vec::with_capacity(index.entries.len());
        for e in index.entries {
            match fs.len(&content_file(dir, &e.id)) {
                Ok(n) if n == u64::from(e.size) => visible.push(e),
                Ok(_) => {}                                             // torn: hidden
                Err(err) if err.kind() == io::ErrorKind::NotFound => {} // hidden
                Err(err) => return Err(err.into()),
            }
        }
        let limits = Limits {
            max_entries: caps.max_entries,
            max_pinned: caps.max_pinned,
            ttl: caps.ttl,
        };
        Ok(Self {
            fs,
            dir: dir.to_path_buf(),
            history: History::from_entries(limits, visible),
            pause: index.state.pause,
            perms_ok,
            _lock: lock,
        })
    }

    /// The visible entries, newest first.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        self.history.entries()
    }

    /// The history view (read methods only are reachable through `&`).
    #[must_use]
    pub fn history(&self) -> &History {
        &self.history
    }

    /// Short ids for the listing, in the same order as [`ReadOnlyStore::entries`].
    #[must_use]
    pub fn short_ids(&self) -> Vec<ShortId> {
        let ids: Vec<EntryId> = self.entries().iter().map(|e| e.id).collect();
        ShortId::compute(&ids)
    }

    /// Resolves an index (0 = newest) or a short id to one entry.
    ///
    /// # Errors
    /// [`ResolveError`].
    pub fn resolve(&self, r: &EntryRef) -> Result<&Entry, ResolveError> {
        self.history.resolve(r)
    }

    /// Reads an entry's content.
    ///
    /// # Errors
    /// [`StoreError::Io`] (not found when absent).
    pub fn content(&self, id: &EntryId) -> Result<Vec<u8>, StoreError> {
        Ok(self.fs.read(&content_file(&self.dir, id))?)
    }

    /// Pins above the pin cap, if any: `(pinned, max_pinned)`.
    #[must_use]
    pub fn pins_over_cap(&self) -> Option<(usize, usize)> {
        self.history.pins_over_cap()
    }

    /// The persisted pause state.
    #[must_use]
    pub fn pause(&self) -> PauseState {
        self.pause
    }

    /// Whether the directory and files had safe permissions (0700/0600).
    #[must_use]
    pub fn perms_ok(&self) -> bool {
        self.perms_ok
    }
}

/// Non-blocking `flock(LOCK_SH)` on an existing lock file, opened read-only. No lock file
/// means no daemon ever ran here: nothing to lock and nothing to create.
fn take_shared_lock(path: &Path) -> Result<Option<File>, StoreError> {
    let file = match OpenOptions::new().read(true).open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    match rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockShared) {
        Ok(()) => Ok(Some(file)),
        Err(rustix::io::Errno::WOULDBLOCK) => Err(StoreError::DaemonRunning),
        Err(e) => Err(io::Error::from(e).into()),
    }
}
