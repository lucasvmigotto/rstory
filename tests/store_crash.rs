//! SC-001: after a crash at any point, every acknowledged insert survives intact and no
//! partial entry or orphan file remains.

use std::collections::BTreeSet;
use std::fs;

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
    let cases = std::env::var("PROPTEST_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(256);
    ProptestConfig { cases, ..ProptestConfig::default() }
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn acknowledged_inserts_survive_a_crash(inserts in 1usize..8, crash_at in 0u64..90) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("store");
        let caps = Caps::default();
        let faulty = FaultFs::new(crash_at, Fault::Crash);

        let mut acked: Vec<(EntryId, Vec<u8>)> = Vec::new();
        if let Ok(mut store) = Store::open_exclusive(&dir, caps, faulty.clone()) {
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

        let store = Store::open_exclusive(&dir, caps, RealFs).expect("store reopens after a crash");
        let attempted: BTreeSet<Vec<u8>> = (0..inserts).map(payload).collect();

        for (id, bytes) in &acked {
            let row = store.entries().iter().find(|e| e.id == *id);
            prop_assert!(row.is_some(), "acknowledged entry lost");
            prop_assert_eq!(&store.content(id).unwrap(), bytes);
        }
        prop_assert!(store.entries().len() <= acked.len() + 1);
        for e in store.entries() {
            let bytes = store.content(&e.id).unwrap();
            prop_assert!(attempted.contains(&bytes), "entry with foreign content");
            prop_assert_eq!(e.hash, ContentHash::of(&bytes));
            prop_assert_eq!(e.size as usize, bytes.len());
        }

        // No orphans: content/ holds exactly the rows' files; the top level has only ours.
        let ids: BTreeSet<String> = store.entries().iter().map(|e| e.id.to_string()).collect();
        let on_disk: BTreeSet<String> = fs::read_dir(dir.join("content")).unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
        prop_assert_eq!(on_disk, ids);
        let top: BTreeSet<String> = fs::read_dir(&dir).unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
        let allowed: BTreeSet<String> =
            ["content", "index.json", "store.lock"].iter().map(ToString::to_string).collect();
        prop_assert!(top.is_subset(&allowed), "unexpected files: {:?}", top);
    }
}
