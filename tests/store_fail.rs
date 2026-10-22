//! A failed insert leaves the store consistent and usable.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use rstory::config::Caps;
use rstory::domain::entry::Content;
use rstory::store::fs::{Fault, FaultFs, RealFs};
use rstory::store::{Mutation, Store};
use time::OffsetDateTime;

fn content(text: &str) -> Content {
    Content::new(text.as_bytes().to_vec(), 1 << 20).unwrap()
}

fn at(n: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(4_102_444_800 + n).unwrap()
}

fn names(p: &Path) -> BTreeSet<String> {
    fs::read_dir(p)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect()
}

#[test]
fn failure_at_any_step_of_an_insert_is_cleaned_up() {
    let open_cost = {
        let t = tempfile::tempdir().unwrap();
        let fs = FaultFs::recording();
        drop(Store::open_exclusive(&t.path().join("s"), Caps::default(), fs.clone()).unwrap());
        fs.mutations()
    };

    // An insert makes 10 mutating calls; fail each of them in turn.
    for step in 0..10 {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("s");
        let fs = FaultFs::new(open_cost + step, Fault::Fail);
        let mut store = Store::open_exclusive(&dir, Caps::default(), fs).unwrap();

        let first = store.insert(&content("first"), at(1));
        assert!(first.is_err(), "step {step} should fail the insert");
        let second = store
            .insert(&content("second"), at(2))
            .expect("store is usable afterwards");

        let ids: Vec<_> = store.entries().iter().map(|e| e.id).collect();
        assert_eq!(ids, [second], "step {step}: the failed insert left no row");
        assert_eq!(store.content(&second).unwrap(), b"second");
        let listed: BTreeSet<String> = ids.iter().map(ToString::to_string).collect();
        assert_eq!(
            names(&dir.join("content")),
            listed,
            "step {step}: no stray content files"
        );
        assert!(!dir.join("index.json.tmp").exists(), "step {step}");

        drop(store);
        let reopened = Store::open_exclusive(&dir, Caps::default(), RealFs).unwrap();
        let again: Vec<_> = reopened.entries().iter().map(|e| e.id).collect();
        assert_eq!(again, [second], "step {step}: disk agrees with memory");
    }
}

#[test]
fn a_failed_unlink_is_retried_by_the_next_apply() {
    let open_cost = {
        let t = tempfile::tempdir().unwrap();
        let fs = FaultFs::recording();
        drop(Store::open_exclusive(&t.path().join("s"), Caps::default(), fs.clone()).unwrap());
        fs.mutations()
    };
    for retry_directly in [false, true] {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("s");
        // open, insert (10 calls), then Remove: 5 for the commit, and the 16th call is the unlink.
        let fs = FaultFs::new(open_cost + 10 + 5, Fault::Fail);
        let mut store = Store::open_exclusive(&dir, Caps::default(), fs).unwrap();
        let a = store.insert(&content("a"), at(1)).unwrap();
        store
            .apply(vec![Mutation::Remove(a)], at(2))
            .expect("the commit succeeded");

        let file = dir.join("content").join(a.to_string());
        assert!(file.exists(), "the unlink failed");
        assert_eq!(store.pending_unlinks(), 1);

        if retry_directly {
            assert_eq!(store.retry_pending_unlinks(), 0);
        } else {
            store.insert(&content("b"), at(3)).unwrap();
        }
        assert!(!file.exists(), "retried");
        assert_eq!(store.pending_unlinks(), 0);
    }
}
