//! SC-001: after a crash or a power loss at any point, every acknowledged insert survives
//! intact and no partial entry or orphan file remains, even when recovery itself is
//! interrupted.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use proptest::prelude::*;
use rstory::config::Caps;
use rstory::domain::entry::{Content, ContentHash};
use rstory::domain::ids::EntryId;
use rstory::store::fs::{Fault, FaultFs, RealFs};
use rstory::store::{Mutation, Store};
use time::OffsetDateTime;

fn payload(i: usize) -> Vec<u8> {
    format!("synthetic payload number {i} {}", "x".repeat(i * 7)).into_bytes()
}

fn config() -> ProptestConfig {
    let cases = std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(256);
    ProptestConfig {
        cases,
        ..ProptestConfig::default()
    }
}

/// Opens a store over `fs` and inserts until the first error; returns the acknowledged ones.
fn workload(dir: &Path, fs: &FaultFs, inserts: usize) -> Vec<(EntryId, Vec<u8>)> {
    let caps = Caps::default();
    let mut acked = Vec::new();
    if let Ok(mut store) = Store::open_exclusive(dir, caps, fs.clone()) {
        for i in 0..inserts {
            let bytes = payload(i);
            let content = Content::new(bytes.clone(), caps.max_entry_bytes).unwrap();
            let now = OffsetDateTime::from_unix_timestamp(1_800_000_000 + i as i64).unwrap();
            match store.insert(&content, now) {
                Ok(id) => acked.push((id, bytes)),
                Err(_) => break,
            }
        }
    }
    acked
}

/// How many mutating calls the whole workload makes when nothing fails.
fn total_mutations(inserts: usize) -> u64 {
    let tmp = tempfile::tempdir().unwrap();
    let fs = FaultFs::recording();
    let acked = workload(&tmp.path().join("store"), &fs, inserts);
    assert_eq!(acked.len(), inserts);
    fs.mutations()
}

/// Reopens with the real filesystem and checks every invariant.
fn check(dir: &Path, acked: &[(EntryId, Vec<u8>)], inserts: usize) -> Result<(), TestCaseError> {
    let store = Store::open_exclusive(dir, Caps::default(), RealFs).expect("store reopens");
    let attempted: BTreeSet<Vec<u8>> = (0..inserts).map(payload).collect();

    for (id, bytes) in acked {
        prop_assert!(
            store.entries().iter().any(|e| e.id == *id),
            "acknowledged entry lost"
        );
        prop_assert_eq!(&store.content(id).unwrap(), bytes);
    }
    prop_assert!(store.entries().len() <= acked.len() + 1);
    for e in store.entries() {
        let bytes = store.content(&e.id).unwrap();
        prop_assert!(attempted.contains(&bytes), "entry with foreign content");
        prop_assert_eq!(e.hash, ContentHash::of(&bytes));
        prop_assert_eq!(e.size as usize, bytes.len());
    }

    // No orphans: content/ holds exactly the rows' files; the top level holds only ours.
    let names = |p: &Path| -> BTreeSet<String> {
        fs::read_dir(p)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect()
    };
    let ids: BTreeSet<String> = store.entries().iter().map(|e| e.id.to_string()).collect();
    prop_assert_eq!(names(&dir.join("content")), ids);
    let allowed: BTreeSet<String> = ["content", "index.json", "store.lock"]
        .iter()
        .map(ToString::to_string)
        .collect();
    let top = names(dir);
    prop_assert!(top.is_subset(&allowed), "unexpected files: {:?}", top);
    Ok(())
}

/// Copies a directory tree, keeping permission bits.
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir(to).unwrap();
    fs::set_permissions(to, fs::metadata(from).unwrap().permissions()).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &target);
        } else {
            fs::copy(e.path(), &target).unwrap();
        }
    }
}

/// How many mutating calls recovering (opening) the store in `dir` makes.
fn recovery_mutations(dir: &Path) -> u64 {
    let scratch = tempfile::tempdir().unwrap();
    let copy = scratch.path().join("store");
    if dir.exists() {
        copy_tree(dir, &copy);
    }
    let rec = FaultFs::recording();
    drop(Store::open_exclusive(&copy, Caps::default(), rec.clone()));
    rec.mutations()
}

/// `recovery_raw`, when set, picks a crash point inside the next open's own recovery work
/// (reduced modulo the number of calls it makes, so the first and last are reachable).
fn scenario(
    inserts: usize,
    crash_at: u64,
    fault: Fault,
    recovery_raw: Option<u64>,
) -> Result<(), TestCaseError> {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("store");
    let faulty = FaultFs::new(crash_at, fault);
    let acked = workload(&dir, &faulty, inserts);
    faulty.power_off();

    if let Some(raw) = recovery_raw {
        // The machine dies again while the next open is recovering.
        let at = raw % (recovery_mutations(&dir) + 1);
        let again = FaultFs::new(at, fault);
        drop(Store::open_exclusive(&dir, Caps::default(), again.clone()));
        again.power_off();
    }
    check(&dir, &acked, inserts)
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn acknowledged_inserts_survive(
        (inserts, crash_at, recovery_raw) in (1usize..8).prop_flat_map(|n| (Just(n), 0..=total_mutations(n), 0u64..1000)),
    ) {
        for fault in [Fault::Crash, Fault::PowerLoss] {
            scenario(inserts, crash_at, fault, None)?;
            scenario(inserts, crash_at, fault, Some(recovery_raw))?;
        }
    }
}

#[test]
fn a_brand_new_store_survives_power_loss_right_after_its_first_insert() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("store");
    let fs = FaultFs::new(u64::MAX, Fault::PowerLoss);
    let acked = workload(&dir, &fs, 1);
    assert_eq!(acked.len(), 1);
    fs.power_off();
    let store = Store::open_exclusive(&dir, Caps::default(), RealFs).unwrap();
    assert_eq!(
        store.entries().len(),
        1,
        "the store's own directory entry must be durable"
    );
    assert_eq!(store.content(&acked[0].0).unwrap(), acked[0].1);
}

// ---- batches with evictions: index commit first, unlinks after ----

fn small_caps() -> Caps {
    Caps {
        max_entries: 3,
        max_pinned: 2,
        ..Caps::default()
    }
}

/// Applies each round as one batch; returns the contents (newest first) after the last
/// acknowledged batch and the round that failed in flight, if any.
fn batch_workload(
    dir: &Path,
    fs: &FaultFs,
    rounds: &[Vec<usize>],
) -> (Vec<Vec<u8>>, Option<Vec<usize>>) {
    let caps = small_caps();
    let mut snapshot = Vec::new();
    let Ok(mut store) = Store::open_exclusive(dir, caps, fs.clone()) else {
        return (snapshot, None);
    };
    for (r, round) in rounds.iter().enumerate() {
        let batch = round
            .iter()
            .map(|&i| Mutation::RecordCopy(Content::new(payload(i), caps.max_entry_bytes).unwrap()))
            .collect();
        let now = OffsetDateTime::from_unix_timestamp(1_800_000_000 + 10 * r as i64).unwrap();
        if store.apply(batch, now).is_err() {
            return (snapshot, Some(round.clone()));
        }
        snapshot = store
            .entries()
            .iter()
            .map(|e| store.content(&e.id).unwrap())
            .collect();
    }
    (snapshot, None)
}

fn batch_total_mutations(rounds: &[Vec<usize>]) -> u64 {
    let tmp = tempfile::tempdir().unwrap();
    let fs = FaultFs::recording();
    let (_, inflight) = batch_workload(&tmp.path().join("store"), &fs, rounds);
    assert!(inflight.is_none());
    fs.mutations()
}

/// The contents after recording `round` on `before` (no pins): de-dup moves to the front, the
/// cap of 3 evicts the oldest.
fn model_apply(before: &[Vec<u8>], round: &[usize]) -> Vec<Vec<u8>> {
    let mut v = before.to_vec();
    for &i in round {
        let p = payload(i);
        v.retain(|c| *c != p);
        v.insert(0, p);
        v.truncate(3);
    }
    v
}

fn batch_check(
    dir: &Path,
    snapshot: &[Vec<u8>],
    inflight: &Option<Vec<usize>>,
) -> Result<(), TestCaseError> {
    let store = Store::open_exclusive(dir, small_caps(), RealFs).expect("store reopens");
    let attempted: BTreeSet<Vec<u8>> = (0..8).map(payload).collect();
    prop_assert!(store.entries().len() <= 3, "count cap");
    let mut contents = Vec::new();
    for e in store.entries() {
        let bytes = store.content(&e.id).unwrap();
        prop_assert!(attempted.contains(&bytes));
        prop_assert_eq!(e.hash, ContentHash::of(&bytes));
        contents.push(bytes);
    }
    let distinct: BTreeSet<_> = contents.iter().collect();
    prop_assert_eq!(distinct.len(), contents.len(), "de-dup");
    // Atomic: either the last acknowledged state, or that state plus the in-flight batch.
    let mut allowed = vec![snapshot.to_vec()];
    if let Some(round) = inflight {
        allowed.push(model_apply(snapshot, round));
    }
    prop_assert!(
        allowed.contains(&contents),
        "state is neither before nor after the batch"
    );
    // Evicted entries' files are gone: orphans left by a crash between commit and unlink were swept.
    let on_disk: BTreeSet<String> = fs::read_dir(dir.join("content"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    let ids: BTreeSet<String> = store.entries().iter().map(|e| e.id.to_string()).collect();
    prop_assert_eq!(on_disk, ids);
    Ok(())
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn batches_with_evictions_survive(
        (rounds, crash_at, recovery_raw) in proptest::collection::vec(proptest::collection::vec(0usize..8, 1..4), 1..5)
            .prop_flat_map(|r| {
                let total = batch_total_mutations(&r);
                (Just(r), 0..=total, 0u64..1000)
            }),
    ) {
        for fault in [Fault::Crash, Fault::PowerLoss] {
            for recovery in [None, Some(recovery_raw)] {
                let tmp = tempfile::tempdir().unwrap();
                let dir = tmp.path().join("store");
                let faulty = FaultFs::new(crash_at, fault);
                let (snapshot, inflight) = batch_workload(&dir, &faulty, &rounds);
                faulty.power_off();
                if let Some(raw) = recovery {
                    let at = raw % (recovery_mutations(&dir) + 1);
                    let again = FaultFs::new(at, fault);
                    drop(Store::open_exclusive(&dir, small_caps(), again.clone()));
                    again.power_off();
                }
                batch_check(&dir, &snapshot, &inflight)?;
            }
        }
    }
}
