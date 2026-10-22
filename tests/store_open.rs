//! Opening: orphan sweep both ways, unknown version, exclusive lock.

use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use rstory::config::Caps;
use rstory::domain::entry::Content;
use rstory::domain::history::HistoryError;
use rstory::domain::ids::EntryId;
use rstory::store::fs::RealFs;
use rstory::store::{Mutation, Store, StoreError};
use time::OffsetDateTime;

fn now(n: i64) -> OffsetDateTime {
    // 4_102_444_800 is 2100-01-01: far in the future, so reopening a store never expires its
    // entries against the wall clock (a future timestamp counts as "now" for expiry).
    OffsetDateTime::from_unix_timestamp(4_102_444_800 + n).unwrap()
}

/// Like `fs::write`, but 0600 so the store accepts the file.
fn write_private(path: impl AsRef<Path>, bytes: &[u8]) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    f.write_all(bytes).unwrap();
}

fn open(dir: &Path) -> Result<Store<RealFs>, StoreError> {
    Store::open_exclusive(dir, Caps::default(), RealFs)
}

fn insert(s: &mut Store<RealFs>, text: &str, n: i64) -> EntryId {
    s.insert(
        &Content::new(text.as_bytes().to_vec(), 1 << 20).unwrap(),
        now(n),
    )
    .unwrap()
}

#[test]
fn content_without_a_row_is_swept() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let (a, b) = {
        let mut s = open(&dir).unwrap();
        (insert(&mut s, "one", 1), insert(&mut s, "two", 2))
    };
    let stray = EntryId::new(now(3));
    write_private(dir.join("content").join(stray.to_string()), b"stray");
    write_private(dir.join("content").join(".tmp-leftover"), b"half");
    write_private(dir.join("index.json.tmp"), b"half");

    let s = open(&dir).unwrap();
    assert_eq!(s.entries().len(), 2);
    assert!(!dir.join("content").join(stray.to_string()).exists());
    assert!(!dir.join("content").join(".tmp-leftover").exists());
    assert!(!dir.join("index.json.tmp").exists());
    assert_eq!(s.content(&a).unwrap(), b"one");
    assert_eq!(s.content(&b).unwrap(), b"two");
}

#[test]
fn row_without_content_is_dropped_and_the_index_recommitted() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let (a, b) = {
        let mut s = open(&dir).unwrap();
        (insert(&mut s, "one", 1), insert(&mut s, "two", 2))
    };
    fs::remove_file(dir.join("content").join(a.to_string())).unwrap();

    {
        let s = open(&dir).unwrap();
        assert_eq!(s.entries().len(), 1);
        assert_eq!(s.entries()[0].id, b);
    }
    let raw = fs::read_to_string(dir.join("index.json")).unwrap();
    assert!(
        !raw.contains(&a.to_string()),
        "dropped row must be gone from disk"
    );
    assert!(raw.contains(&b.to_string()));
}

#[test]
fn unknown_version_is_refused_and_the_file_is_untouched() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    drop(open(&dir).unwrap());
    let original = br#"{"version":2,"entries":[],"future":{"x":1}}"#;
    write_private(dir.join("index.json"), original);

    let err = open(&dir).unwrap_err();
    assert!(matches!(err, StoreError::UnknownVersion(2)), "{err:?}");
    assert_eq!(fs::read(dir.join("index.json")).unwrap(), original);
    assert!(!dir.join("index.json.tmp").exists());
}

#[test]
fn second_exclusive_open_is_locked_until_the_first_drops() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let first = open(&dir).unwrap();
    assert!(matches!(open(&dir).unwrap_err(), StoreError::Locked));
    drop(first);
    assert!(open(&dir).is_ok());
}

#[test]
fn reopen_keeps_entries_newest_first() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let (a, b) = {
        let mut s = open(&dir).unwrap();
        (insert(&mut s, "one", 1), insert(&mut s, "two", 2))
    };
    let s = open(&dir).unwrap();
    let ids: Vec<_> = s.entries().iter().map(|e| e.id).collect();
    assert_eq!(ids, [b, a]);
}

#[test]
fn torn_content_file_drops_its_row_and_is_unlinked() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let (a, b) = {
        let mut s = open(&dir).unwrap();
        (
            insert(&mut s, "one: a longer text", 1),
            insert(&mut s, "two", 2),
        )
    };
    // A power loss left the first content file shorter than its row says.
    let f = fs::OpenOptions::new()
        .write(true)
        .open(dir.join("content").join(a.to_string()))
        .unwrap();
    f.set_len(3).unwrap();
    drop(f);

    {
        let s = open(&dir).unwrap();
        let ids: Vec<_> = s.entries().iter().map(|e| e.id).collect();
        assert_eq!(ids, [b]);
    }
    assert!(!dir.join("content").join(a.to_string()).exists());
    let raw = fs::read_to_string(dir.join("index.json")).unwrap();
    assert!(!raw.contains(&a.to_string()));
}

fn open_with(dir: &Path, caps: Caps) -> Store<RealFs> {
    Store::open_exclusive(dir, caps, RealFs).unwrap()
}

fn caps(max_entries: u32, max_pinned: u32) -> Caps {
    Caps {
        max_entries,
        max_pinned,
        ..Caps::default()
    }
}

fn content_names(dir: &Path) -> std::collections::BTreeSet<String> {
    fs::read_dir(dir.join("content"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect()
}

#[test]
fn reopening_with_a_lower_count_cap_evicts_immediately() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    {
        let mut s = open(&dir).unwrap();
        for i in 0..20 {
            insert(&mut s, &format!("entry {i}"), i);
        }
    }
    let s = open_with(&dir, caps(5, 4));
    let ids: std::collections::BTreeSet<String> =
        s.entries().iter().map(|e| e.id.to_string()).collect();
    assert_eq!(ids.len(), 5);
    assert_eq!(
        content_names(&dir),
        ids,
        "evicted files are gone before any further call"
    );
    let raw: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("index.json")).unwrap()).unwrap();
    assert_eq!(
        raw["entries"].as_array().unwrap().len(),
        5,
        "and so are their rows"
    );
}

#[test]
fn pins_above_a_lowered_pin_cap_are_kept_and_new_pins_blocked() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let ids: Vec<EntryId> = {
        let mut s = open_with(&dir, caps(20, 15));
        let ids: Vec<_> = (0..14)
            .map(|i| insert(&mut s, &format!("e{i}"), i))
            .collect();
        let pins = ids[..12].iter().map(|id| Mutation::Pin(*id)).collect();
        s.apply(pins, now(100)).unwrap();
        ids
    };
    let mut s = open_with(&dir, caps(20, 10));
    assert_eq!(s.entries().len(), 14);
    assert_eq!(
        s.entries().iter().filter(|e| e.pinned).count(),
        12,
        "no pin is dropped"
    );
    assert_eq!(s.history().pins_over_cap(), Some((12, 10)));

    let refused = |s: &mut Store<RealFs>, id| {
        matches!(
            s.apply(vec![Mutation::Pin(id)], now(200)),
            Err(StoreError::History(HistoryError::PinLimit))
        )
    };
    assert!(refused(&mut s, ids[12]), "13th pin refused");
    s.apply(vec![Mutation::Unpin(ids[0])], now(201)).unwrap(); // 11 pinned
    assert!(refused(&mut s, ids[12]));
    s.apply(vec![Mutation::Unpin(ids[1])], now(202)).unwrap(); // 10 pinned
    assert!(refused(&mut s, ids[12]), "at the cap is still refused");
    s.apply(vec![Mutation::Unpin(ids[2])], now(203)).unwrap(); // 9 pinned
    s.apply(vec![Mutation::Pin(ids[12])], now(204)).unwrap();
}

#[test]
fn ten_pins_with_a_lowered_count_cap_are_kept_and_new_copies_still_recorded() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    {
        let mut s = open(&dir).unwrap();
        let ids: Vec<_> = (0..10)
            .map(|i| insert(&mut s, &format!("p{i}"), i))
            .collect();
        s.apply(ids.iter().map(|id| Mutation::Pin(*id)).collect(), now(50))
            .unwrap();
    }
    let mut s = open_with(&dir, caps(5, 4));
    assert_eq!(s.entries().len(), 10, "all pins kept");
    assert!(s.entries().iter().all(|e| e.pinned));

    let a = insert(&mut s, "first new copy", 100);
    assert_eq!(
        s.content(&a).unwrap(),
        b"first new copy",
        "recorded despite the full cap"
    );
    assert_eq!(s.entries().len(), 11);

    let b = insert(&mut s, "second new copy", 101);
    assert_eq!(s.content(&b).unwrap(), b"second new copy");
    assert!(
        s.entries().iter().all(|e| e.id != a),
        "the oldest unpinned was evicted"
    );
    assert!(!dir.join("content").join(a.to_string()).exists());
    assert_eq!(s.entries().len(), 11);
    assert_eq!(s.entries().iter().filter(|e| e.pinned).count(), 10);

    // A restart must not lose the latest copy.
    drop(s);
    let s = open_with(&dir, caps(5, 4));
    assert_eq!(s.entries().len(), 11);
    assert_eq!(s.entries()[0].id, b);
    assert_eq!(s.content(&b).unwrap(), b"second new copy");
}

#[test]
fn pins_filling_the_cap_still_keep_the_newest_copy_across_restarts() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    {
        let mut s = open(&dir).unwrap();
        let ids: Vec<_> = (0..5)
            .map(|i| insert(&mut s, &format!("p{i}"), i))
            .collect();
        s.apply(ids.iter().map(|id| Mutation::Pin(*id)).collect(), now(50))
            .unwrap();
    }
    let mut s = open_with(&dir, caps(5, 4));
    let x = insert(&mut s, "latest copy", 100);
    drop(s);
    let s = open_with(&dir, caps(5, 4));
    assert_eq!(s.entries().len(), 6);
    assert_eq!(s.content(&x).unwrap(), b"latest copy");
}
