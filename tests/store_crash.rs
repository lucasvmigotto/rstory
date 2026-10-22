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
use rstory::store::Store;
use rstory::store::fs::{Fault, FaultFs, RealFs};
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

fn scenario(
    inserts: usize,
    crash_at: u64,
    fault: Fault,
    recovery_crash: Option<u64>,
) -> Result<(), TestCaseError> {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("store");
    let faulty = FaultFs::new(crash_at, fault);
    let acked = workload(&dir, &faulty, inserts);
    faulty.power_off();

    if let Some(n) = recovery_crash {
        // The machine dies again while the next open is recovering.
        let again = FaultFs::new(n, fault);
        drop(Store::open_exclusive(&dir, Caps::default(), again.clone()));
        again.power_off();
    }
    check(&dir, &acked, inserts)
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn acknowledged_inserts_survive(
        (inserts, crash_at, recovery_at) in (1usize..8).prop_flat_map(|n| (Just(n), 0..=total_mutations(n), 0u64..6)),
    ) {
        for fault in [Fault::Crash, Fault::PowerLoss] {
            scenario(inserts, crash_at, fault, None)?;
            scenario(inserts, crash_at, fault, Some(recovery_at))?;
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
