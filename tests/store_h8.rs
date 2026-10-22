//! H8: removed, cleared and expired content is gone from disk when `apply` returns.

use std::path::{Path, PathBuf};

use rstory::config::Caps;
use rstory::domain::entry::Content;
use rstory::domain::ids::EntryId;
use rstory::store::fs::RealFs;
use rstory::store::{Mutation, Store};
use time::OffsetDateTime;

fn at(secs: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(4_102_444_800 + secs).unwrap()
}

fn file(dir: &Path, id: &EntryId) -> PathBuf {
    dir.join("content").join(id.to_string())
}

fn record(s: &mut Store<RealFs>, text: &str, secs: i64) -> EntryId {
    s.insert(
        &Content::new(text.as_bytes().to_vec(), 1 << 20).unwrap(),
        at(secs),
    )
    .unwrap()
}

fn setup() -> (tempfile::TempDir, std::path::PathBuf, Store<RealFs>) {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let s = Store::open_exclusive(&dir, Caps::default(), RealFs).unwrap();
    (t, dir, s)
}

#[test]
fn remove_unlinks_immediately() {
    let (_t, dir, mut s) = setup();
    let (a, b) = (record(&mut s, "a", 0), record(&mut s, "b", 1));
    s.apply(vec![Mutation::Remove(a)], at(2)).unwrap();
    assert!(!file(&dir, &a).exists());
    assert!(file(&dir, &b).exists());
}

#[test]
fn clear_unlinks_unpinned_and_keeps_pinned_files() {
    let (_t, dir, mut s) = setup();
    let (a, b, c) = (
        record(&mut s, "a", 0),
        record(&mut s, "b", 1),
        record(&mut s, "c", 2),
    );
    s.apply(vec![Mutation::Pin(b)], at(3)).unwrap();
    s.apply(vec![Mutation::Clear], at(4)).unwrap();
    assert!(!file(&dir, &a).exists() && !file(&dir, &c).exists());
    assert!(file(&dir, &b).exists());
    s.apply(vec![Mutation::ClearAll], at(5)).unwrap();
    assert!(!file(&dir, &b).exists());
}

#[test]
fn expire_unlinks_stale_unpinned_entries_only() {
    let (_t, dir, mut s) = setup();
    let (old, pinned, fresh) = (
        record(&mut s, "old", 0),
        record(&mut s, "pinned", 1),
        record(&mut s, "fresh", 8 * 86_400),
    );
    s.apply(vec![Mutation::Pin(pinned)], at(8 * 86_400 + 1))
        .unwrap();
    s.apply(vec![Mutation::Expire], at(8 * 86_400 + 2)).unwrap();
    assert!(!file(&dir, &old).exists());
    assert!(file(&dir, &pinned).exists() && file(&dir, &fresh).exists());
}

#[test]
fn eviction_by_the_cap_unlinks_immediately() {
    let (_t, dir, mut s) = setup();
    let first = record(&mut s, "first", 0);
    for i in 1..=20 {
        record(&mut s, &format!("filler {i}"), i);
    }
    assert!(!file(&dir, &first).exists());
}
