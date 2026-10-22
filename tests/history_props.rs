//! SC-002: the history invariants (H1-H10) hold after every step of random command sequences.

use std::time::Duration;

use proptest::prelude::*;
use rstory::domain::entry::Content;
use rstory::domain::history::{History, HistoryError, Limits, Recorded};
use rstory::domain::ids::EntryId;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
enum Cmd {
    Record(u8),
    Touch(usize),
    Pin(usize),
    Unpin(usize),
    Remove(usize),
    Clear,
    ClearAll,
    Expire,
    Tick(i32),
}

fn cmd() -> impl Strategy<Value = Cmd> {
    prop_oneof![
        6 => (0u8..40).prop_map(Cmd::Record),
        2 => any::<usize>().prop_map(Cmd::Touch),
        3 => any::<usize>().prop_map(Cmd::Pin),
        2 => any::<usize>().prop_map(Cmd::Unpin),
        1 => any::<usize>().prop_map(Cmd::Remove),
        1 => Just(Cmd::Clear),
        1 => Just(Cmd::ClearAll),
        2 => Just(Cmd::Expire),
        4 => (-3_600i32..7_200).prop_map(Cmd::Tick),
    ]
}

fn limits() -> impl Strategy<Value = Limits> {
    (2u32..=12).prop_flat_map(|max_entries| {
        (Just(max_entries), 0..max_entries, 1u64..=4).prop_map(|(e, p, h)| Limits {
            max_entries: e,
            max_pinned: p,
            ttl: Duration::from_secs(h * 3600),
        })
    })
}

fn content(k: u8) -> Content {
    Content::new(format!("fixture content {k}").into_bytes(), 1 << 20).unwrap()
}

fn ids(h: &History) -> Vec<EntryId> {
    h.entries().iter().map(|e| e.id).collect()
}

fn pinned(h: &History) -> Vec<EntryId> {
    h.entries()
        .iter()
        .filter(|e| e.pinned)
        .map(|e| e.id)
        .collect()
}

fn invariants(h: &History, l: Limits) -> Result<(), TestCaseError> {
    prop_assert!(
        h.len() <= l.max_entries as usize,
        "H1: len {} > cap",
        h.len()
    );
    prop_assert!(
        pinned(h).len() <= l.max_pinned as usize,
        "H3: too many pins"
    );
    let mut hashes: Vec<_> = h.entries().iter().map(|e| e.hash.to_string()).collect();
    hashes.sort();
    let n = hashes.len();
    hashes.dedup();
    prop_assert_eq!(hashes.len(), n, "H4: duplicate hash");
    let times: Vec<_> = h.entries().iter().map(|e| e.last_copied).collect();
    prop_assert!(
        times.windows(2).all(|w| w[0] >= w[1]),
        "H7: not ordered by last_copied desc"
    );
    let mut seen = ids(h);
    seen.sort();
    seen.dedup();
    prop_assert_eq!(seen.len(), h.len(), "duplicate ids");
    Ok(())
}

proptest! {
    #[test]
    fn invariants_hold_after_every_step(l in limits(), cmds in proptest::collection::vec(cmd(), 1..80)) {
        let mut h = History::new(l);
        // 4_102_444_800 is 2100-01-01: far in the future, so reopening a store never expires its
        // entries against the wall clock (a future timestamp counts as "now" for expiry).
        let mut now = OffsetDateTime::from_unix_timestamp(4_102_444_800).unwrap();
        let mut counter = 0u128;
        for c in cmds {
            let before = ids(&h);
            let pins_before = pinned(&h);
            let pick = |i: usize| before.get(i % before.len().max(1)).copied();
            match c {
                Cmd::Record(k) => {
                    counter += 1;
                    let id = EntryId::from_parts(counter, 0);
                    let known = h.entries().iter().any(|e| e.hash == content(k).hash());
                    let r = h.record(&content(k), id, now);
                    match r {
                        Recorded::Refreshed { id: rid } => {
                            prop_assert!(known, "H4: refreshed an unknown hash");
                            prop_assert_eq!(h.len(), before.len());
                            prop_assert_eq!(h.entries()[0].id, rid);
                        }
                        Recorded::Inserted { id: nid, evicted } => {
                            prop_assert!(!known);
                            prop_assert_eq!(h.entries()[0].id, nid);
                            for e in &evicted {
                                prop_assert!(!pins_before.contains(e), "H2: a pin was evicted");
                            }
                        }
                    }
                    for p in &pins_before {
                        prop_assert!(ids(&h).contains(p), "H2: pin lost on record");
                    }
                }
                Cmd::Touch(i) => {
                    if let Some(id) = pick(i) {
                        h.touch(&id, now).unwrap();
                        prop_assert_eq!(h.len(), before.len(), "H10: touch must not create");
                        prop_assert_eq!(h.entries()[0].id, id);
                        let mut a = before.clone(); a.sort();
                        let mut b = ids(&h); b.sort();
                        prop_assert_eq!(a, b);
                    }
                    prop_assert_eq!(h.touch(&EntryId::from_parts(u128::MAX >> 90, 1), now), Err(HistoryError::NotFound));
                }
                Cmd::Pin(i) => {
                    if let Some(id) = pick(i) {
                        let r = h.pin(&id);
                        if pins_before.contains(&id) {
                            prop_assert_eq!(r, Ok(false), "pin is idempotent");
                        } else if pins_before.len() >= l.max_pinned as usize {
                            prop_assert_eq!(r, Err(HistoryError::PinLimit));
                            prop_assert_eq!(pinned(&h), pins_before.clone(), "refused pin changes nothing");
                        } else {
                            prop_assert_eq!(r, Ok(true));
                        }
                        prop_assert_eq!(ids(&h), before, "pin never reorders");
                    }
                }
                Cmd::Unpin(i) => {
                    if let Some(id) = pick(i) {
                        let r = h.unpin(&id);
                        prop_assert_eq!(r, Ok(pins_before.contains(&id)));
                        prop_assert_eq!(h.unpin(&id), Ok(false), "unpin is idempotent");
                    }
                }
                Cmd::Remove(i) => {
                    if let Some(id) = pick(i) {
                        h.remove(&id).unwrap();
                        prop_assert!(!ids(&h).contains(&id));
                        prop_assert_eq!(h.remove(&id), Err(HistoryError::NotFound));
                    }
                }
                Cmd::Clear => {
                    let removed = h.clear();
                    prop_assert_eq!(pinned(&h), pins_before.clone(), "pins survive clear");
                    prop_assert_eq!(h.len(), pins_before.len());
                    prop_assert_eq!(removed.len(), before.len() - pins_before.len());
                }
                Cmd::ClearAll => {
                    let removed = h.clear_all();
                    prop_assert!(h.is_empty());
                    prop_assert_eq!(removed.len(), before.len());
                }
                Cmd::Expire => {
                    let removed = h.expire(now);
                    for e in h.entries() {
                        if !e.pinned {
                            let age = (now - e.last_copied).max(time::Duration::ZERO);
                            prop_assert!(age < time::Duration::try_from(l.ttl).unwrap(), "H5: stale unpinned entry");
                        }
                    }
                    for p in &pins_before {
                        prop_assert!(!removed.contains(p), "expiry spared pins");
                    }
                }
                Cmd::Tick(s) => now += time::Duration::seconds(i64::from(s)),
            }
            invariants(&h, l)?;
        }
    }
}
