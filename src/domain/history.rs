//! The `History` aggregate: de-duplication, the count and pin caps, expiry. Pure, no I/O.
//!
//! Commands return what changed (ids only, Constitution III) so the store knows which content
//! files to write and unlink.

use std::time::Duration;

use thiserror::Error;
use time::OffsetDateTime;

use super::entry::{Content, Entry};
use super::ids::{EntryId, EntryRef};

/// The limits the aggregate enforces. Built from the validated `config::Caps` by the store
/// (the domain does not depend on `config`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Maximum number of entries, pinned included.
    pub max_entries: u32,
    /// Maximum number of pinned entries; below `max_entries`.
    pub max_pinned: u32,
    /// Unpinned entries expire this long after `last_copied`.
    pub ttl: Duration,
}

/// What `record` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recorded {
    /// A new entry was added at index 0; `evicted` lists the oldest unpinned entries removed
    /// to respect the count cap.
    Inserted {
        /// The new entry.
        id: EntryId,
        /// Entries removed by the cap, oldest first.
        evicted: Vec<EntryId>,
    },
    /// The content was already there: the entry moved to index 0 with a refreshed time.
    Refreshed {
        /// The existing entry.
        id: EntryId,
    },
}

/// Why a command was refused.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum HistoryError {
    /// No such entry.
    #[error("no such entry")]
    NotFound,
    /// The pin cap is reached.
    #[error("pin limit reached")]
    PinLimit,
}

/// Why a reference did not resolve to exactly one entry.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// Nothing matches.
    #[error("no such entry")]
    NotFound,
    /// A short id matches several entries.
    #[error("ambiguous reference ({} candidates)", .0.len())]
    Ambiguous(Vec<EntryId>),
}

/// The ordered set of entries, newest (`last_copied`) first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History {
    limits: Limits,
    entries: Vec<Entry>,
}

impl History {
    /// An empty history.
    #[must_use]
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            entries: Vec::new(),
        }
    }

    /// A history over loaded rows (re-sorted newest first; caps are not re-applied).
    #[must_use]
    pub fn from_entries(limits: Limits, entries: Vec<Entry>) -> Self {
        unimplemented!("T024")
    }

    /// The limits in force.
    #[must_use]
    pub fn limits(&self) -> Limits {
        self.limits
    }

    /// The entries, index 0 = newest.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there are no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Records a copy. A known hash is refreshed and moved to index 0; otherwise a new entry
    /// `id` is added at index 0 and the oldest unpinned entries are evicted while the history
    /// is over `max_entries`. The time is never earlier than the newest entry's (a clock that
    /// went backwards must not break the ordering).
    pub fn record(&mut self, content: &Content, id: EntryId, now: OffsetDateTime) -> Recorded {
        unimplemented!("T024")
    }

    /// Restore: moves the entry to index 0 and refreshes `last_copied`.
    ///
    /// # Errors
    /// [`HistoryError::NotFound`].
    pub fn touch(&mut self, id: &EntryId, now: OffsetDateTime) -> Result<(), HistoryError> {
        unimplemented!("T024")
    }

    /// Pins an entry; `Ok(false)` when it already was (idempotent, even at the cap).
    ///
    /// # Errors
    /// [`HistoryError::NotFound`], or [`HistoryError::PinLimit`] when it would exceed the cap.
    pub fn pin(&mut self, id: &EntryId) -> Result<bool, HistoryError> {
        unimplemented!("T024")
    }

    /// Unpins an entry; `Ok(false)` when it was not pinned (idempotent).
    ///
    /// # Errors
    /// [`HistoryError::NotFound`].
    pub fn unpin(&mut self, id: &EntryId) -> Result<bool, HistoryError> {
        unimplemented!("T024")
    }

    /// Removes one entry, pinned or not.
    ///
    /// # Errors
    /// [`HistoryError::NotFound`].
    pub fn remove(&mut self, id: &EntryId) -> Result<(), HistoryError> {
        unimplemented!("T024")
    }

    /// Removes every unpinned entry; returns their ids.
    pub fn clear(&mut self) -> Vec<EntryId> {
        unimplemented!("T024")
    }

    /// Removes every entry, pinned included; returns their ids.
    pub fn clear_all(&mut self) -> Vec<EntryId> {
        unimplemented!("T024")
    }

    /// Removes unpinned entries whose age (`now - last_copied`, never negative: a future
    /// timestamp counts as now) has reached the ttl; returns their ids.
    pub fn expire(&mut self, now: OffsetDateTime) -> Vec<EntryId> {
        unimplemented!("T024")
    }

    /// The instant at which the next unpinned entry expires (`last_copied + ttl`), if any.
    /// `expire(now)` removes it as soon as `now` reaches that instant.
    #[must_use]
    pub fn next_deadline(&self) -> Option<OffsetDateTime> {
        unimplemented!("T024")
    }

    /// Resolves an index (0 = newest) or a short-id prefix to exactly one entry.
    ///
    /// # Errors
    /// [`ResolveError`].
    pub fn resolve(&self, r: &EntryRef) -> Result<&Entry, ResolveError> {
        unimplemented!("T024")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    const T0: OffsetDateTime = datetime!(2026-10-22 12:00:00 UTC);

    fn limits() -> Limits {
        Limits {
            max_entries: 20,
            max_pinned: 10,
            ttl: Duration::from_secs(7 * 86_400),
        }
    }

    fn at(secs: i64) -> OffsetDateTime {
        T0 + time::Duration::seconds(secs)
    }

    fn id(n: u128) -> EntryId {
        EntryId::from_parts(n, 0)
    }

    fn content(n: usize) -> Content {
        Content::new(format!("content {n}").into_bytes(), 1 << 20).unwrap()
    }

    /// Records `n` distinct contents at one-second steps; returns the ids oldest first.
    fn fill(h: &mut History, from: usize, n: usize) -> Vec<EntryId> {
        (from..from + n)
            .map(|i| {
                let id = id(i as u128 + 1);
                h.record(&content(i), id, at(i as i64));
                id
            })
            .collect()
    }

    fn ids(h: &History) -> Vec<EntryId> {
        h.entries().iter().map(|e| e.id).collect()
    }

    #[test]
    fn the_21st_insert_evicts_the_oldest() {
        let mut h = History::new(limits());
        let all = fill(&mut h, 0, 20);
        assert_eq!(h.len(), 20);
        let r = h.record(&content(20), id(99), at(20));
        assert_eq!(
            r,
            Recorded::Inserted {
                id: id(99),
                evicted: vec![all[0]]
            }
        );
        assert_eq!(h.len(), 20);
        assert_eq!(h.entries()[0].id, id(99));
        assert!(!ids(&h).contains(&all[0]));
    }

    #[test]
    fn with_ten_pins_the_oldest_unpinned_is_evicted() {
        let mut h = History::new(limits());
        let pinned = fill(&mut h, 0, 10);
        for p in &pinned {
            assert_eq!(h.pin(p), Ok(true));
        }
        let unpinned = fill(&mut h, 10, 10);
        assert_eq!(h.len(), 20);
        let r = h.record(&content(20), id(99), at(20));
        assert_eq!(
            r,
            Recorded::Inserted {
                id: id(99),
                evicted: vec![unpinned[0]]
            }
        );
        for p in &pinned {
            assert!(ids(&h).contains(p), "pins are never evicted by the cap");
        }
        assert_eq!(h.len(), 20);
    }

    #[test]
    fn the_11th_pin_is_refused_and_changes_nothing() {
        let mut h = History::new(limits());
        let all = fill(&mut h, 0, 11);
        for p in &all[..10] {
            h.pin(p).unwrap();
        }
        let before = h.clone();
        assert_eq!(h.pin(&all[10]), Err(HistoryError::PinLimit));
        assert_eq!(h, before);
        assert_eq!(
            h.pin(&all[0]),
            Ok(false),
            "re-pinning is a no-op even at the cap"
        );
    }

    #[test]
    fn a_duplicate_refreshes_and_moves_to_the_top() {
        let mut h = History::new(limits());
        let all = fill(&mut h, 0, 3);
        let r = h.record(&content(0), id(500), at(100));
        assert_eq!(r, Recorded::Refreshed { id: all[0] });
        assert_eq!(h.len(), 3);
        assert_eq!(ids(&h), [all[0], all[2], all[1]]);
        assert_eq!(h.entries()[0].last_copied, at(100));
        assert_eq!(h.entries()[0].created, at(0), "created never changes");
    }

    #[test]
    fn future_timestamps_count_as_now_for_expiry() {
        let mut h = History::new(Limits {
            ttl: Duration::from_secs(3600),
            ..limits()
        });
        h.record(&content(1), id(1), at(2 * 86_400)); // two days ahead of `now` below
        assert!(h.expire(at(0)).is_empty());
        assert_eq!(h.len(), 1);
        // Once real time reaches it, the normal rule applies.
        assert_eq!(h.expire(at(2 * 86_400 + 3600)), vec![id(1)]);
    }

    #[test]
    fn expiry_is_inclusive_of_the_ttl_and_spares_pins() {
        let mut h = History::new(Limits {
            ttl: Duration::from_secs(3600),
            ..limits()
        });
        let a = fill(&mut h, 0, 2); // at(0), at(1)
        h.pin(&a[0]).unwrap();
        assert_eq!(h.next_deadline(), Some(at(1) + time::Duration::hours(1)));
        assert!(h.expire(at(3599)).is_empty());
        assert_eq!(h.expire(at(1) + time::Duration::hours(1)), vec![a[1]]);
        assert_eq!(ids(&h), [a[0]]);
        assert_eq!(h.next_deadline(), None, "only pinned entries remain");
    }

    #[test]
    fn clear_keeps_pins_and_clear_all_removes_everything() {
        let mut h = History::new(limits());
        let a = fill(&mut h, 0, 4);
        h.pin(&a[1]).unwrap();
        let mut removed = h.clear();
        removed.sort();
        assert_eq!(removed, vec![a[0], a[2], a[3]]);
        assert_eq!(ids(&h), [a[1]]);
        assert_eq!(h.clear_all(), vec![a[1]]);
        assert!(h.is_empty());
    }

    #[test]
    fn touch_moves_to_the_top_without_adding() {
        let mut h = History::new(limits());
        let a = fill(&mut h, 0, 3);
        h.touch(&a[0], at(50)).unwrap();
        assert_eq!(ids(&h), [a[0], a[2], a[1]]);
        assert_eq!(h.entries()[0].last_copied, at(50));
        assert_eq!(h.touch(&id(777), at(60)), Err(HistoryError::NotFound));
    }

    #[test]
    fn a_clock_going_backwards_keeps_the_order() {
        let mut h = History::new(limits());
        fill(&mut h, 0, 3); // newest at(2)
        h.record(&content(9), id(9), at(-1000));
        assert_eq!(h.entries()[0].id, id(9));
        let times: Vec<_> = h.entries().iter().map(|e| e.last_copied).collect();
        assert!(times.windows(2).all(|w| w[0] >= w[1]));
    }

    #[test]
    fn pin_unpin_remove_and_not_found() {
        let mut h = History::new(limits());
        let a = fill(&mut h, 0, 2);
        assert_eq!(h.unpin(&a[0]), Ok(false));
        assert_eq!(h.pin(&a[0]), Ok(true));
        assert_eq!(h.unpin(&a[0]), Ok(true));
        assert_eq!(h.pin(&id(404)), Err(HistoryError::NotFound));
        assert_eq!(h.remove(&a[0]), Ok(()));
        assert_eq!(h.remove(&a[0]), Err(HistoryError::NotFound));
    }

    #[test]
    fn resolve_by_index_and_short_id() {
        let mut h = History::new(limits());
        let a = vec![id(1), id(2)];
        h.record(&content(1), a[0], at(0));
        h.record(&content(2), a[1], at(1));
        assert_eq!(h.resolve(&EntryRef::Index(0)).unwrap().id, a[1]);
        assert_eq!(h.resolve(&EntryRef::Index(1)).unwrap().id, a[0]);
        assert_eq!(h.resolve(&EntryRef::Index(2)), Err(ResolveError::NotFound));
        let s = a[0].to_string();
        assert_eq!(
            h.resolve(&EntryRef::ShortId(s[..26].to_string()))
                .unwrap()
                .id,
            a[0]
        );
        // ids 1 and 2 share all but the last character: a short prefix is ambiguous.
        let amb = h.resolve(&EntryRef::ShortId(s[..10].to_string()));
        assert_eq!(amb, Err(ResolveError::Ambiguous(vec![a[1], a[0]])));
        assert_eq!(
            h.resolve(&EntryRef::ShortId("ZZZZ".to_string())),
            Err(ResolveError::NotFound)
        );
    }
}
