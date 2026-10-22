//! The read-only handle clients use while the daemon is down (BR14).

use std::marker::PhantomData;
use std::path::Path;

use super::StoreError;
use super::fs::{Fs, RealFs};
use crate::config::Caps;
use crate::domain::entry::{Entry, PauseState};
use crate::domain::history::{History, ResolveError};
use crate::domain::ids::{EntryId, EntryRef, ShortId};

/// A shared-locked, read-only view of the store. NOT YET IMPLEMENTED (T026).
#[derive(Debug)]
pub struct ReadOnlyStore<F: Fs> {
    _fs: PhantomData<F>,
    _lock: Option<std::fs::File>,
}

impl ReadOnlyStore<RealFs> {
    /// Opens the store at `dir` read-only with the default caps.
    ///
    /// # Errors
    /// [`StoreError`].
    pub fn open(_dir: &Path) -> Result<Self, StoreError> {
        unimplemented!("T026")
    }
}

impl<F: Fs> ReadOnlyStore<F> {
    /// Opens the store at `dir` read-only.
    ///
    /// # Errors
    /// [`StoreError`].
    pub fn open_with(_dir: &Path, _caps: Caps, _fs: F) -> Result<Self, StoreError> {
        unimplemented!("T026")
    }

    /// The visible entries, newest first.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        unimplemented!("T026")
    }

    /// The history view.
    #[must_use]
    pub fn history(&self) -> &History {
        unimplemented!("T026")
    }

    /// Short ids for the listing, in order.
    #[must_use]
    pub fn short_ids(&self) -> Vec<ShortId> {
        unimplemented!("T026")
    }

    /// Resolves an index or short id.
    ///
    /// # Errors
    /// [`ResolveError`].
    pub fn resolve(&self, _r: &EntryRef) -> Result<&Entry, ResolveError> {
        unimplemented!("T026")
    }

    /// Reads an entry's content.
    ///
    /// # Errors
    /// [`StoreError::Io`].
    pub fn content(&self, _id: &EntryId) -> Result<Vec<u8>, StoreError> {
        unimplemented!("T026")
    }

    /// Pins above the pin cap, if any.
    #[must_use]
    pub fn pins_over_cap(&self) -> Option<(usize, usize)> {
        unimplemented!("T026")
    }

    /// The persisted pause state.
    #[must_use]
    pub fn pause(&self) -> PauseState {
        unimplemented!("T026")
    }

    /// Whether the directory and files had safe permissions.
    #[must_use]
    pub fn perms_ok(&self) -> bool {
        unimplemented!("T026")
    }
}
