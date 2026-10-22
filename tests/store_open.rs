//! Opening: orphan sweep both ways, unknown version, exclusive lock.

use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use rstory::config::Caps;
use rstory::domain::entry::Content;
use rstory::domain::ids::EntryId;
use rstory::store::fs::RealFs;
use rstory::store::{Store, StoreError};
use time::OffsetDateTime;

fn now(n: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_800_000_000 + n).unwrap()
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
