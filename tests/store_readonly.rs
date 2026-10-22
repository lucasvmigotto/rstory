//! US3: clients read while the daemon is down; a read-only handle never writes.
//!
//! "Never writes" is by construction: `ReadOnlyStore` has only `&self` methods and no access
//! to the `Fs` mutation calls, so there is nothing to call. The tests below also check that
//! the files are byte-identical and nothing is created.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use rstory::config::Caps;
use rstory::domain::entry::{Content, PauseState};
use rstory::domain::history::ResolveError;
use rstory::domain::ids::{EntryId, EntryRef, ShortId};
use rstory::store::fs::RealFs;
use rstory::store::readonly::ReadOnlyStore;
use rstory::store::{Store, StoreError};
use time::OffsetDateTime;

fn at(n: i64) -> OffsetDateTime {
    // 4_102_444_800 is 2100-01-01: far in the future, so reopening a store never expires its
    // entries against the wall clock (a future timestamp counts as "now" for expiry).
    OffsetDateTime::from_unix_timestamp(4_102_444_800 + n).unwrap()
}

/// A store with three entries; the writer is dropped (daemon down).
fn populated(dir: &Path) -> [EntryId; 3] {
    let mut s = Store::open_exclusive(dir, Caps::default(), RealFs).unwrap();
    let mut ins = |text: &str, n| {
        s.insert(
            &Content::new(text.as_bytes().to_vec(), 1 << 20).unwrap(),
            at(n),
        )
        .unwrap()
    };
    [ins("first", 1), ins("second", 2), ins("third", 3)]
}

fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut v = Vec::new();
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            for c in fs::read_dir(&p).unwrap() {
                let c = c.unwrap().path();
                v.push((c.display().to_string(), fs::read(&c).unwrap()));
            }
        } else {
            v.push((p.display().to_string(), fs::read(&p).unwrap()));
        }
    }
    v.sort();
    v
}

#[test]
fn lists_newest_first_with_previews_content_and_refs() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let [a, b, c] = populated(&dir);

    let ro = ReadOnlyStore::open(&dir).unwrap();
    let ids: Vec<_> = ro.entries().iter().map(|e| e.id).collect();
    assert_eq!(ids, [c, b, a]);
    assert_eq!(ro.entries()[0].preview.as_str(), "third");
    assert_eq!(ro.content(&b).unwrap(), b"second");

    assert_eq!(ro.resolve(&EntryRef::Index(0)).unwrap().id, c);
    assert_eq!(ro.resolve(&EntryRef::Index(3)), Err(ResolveError::NotFound));
    let shorts = ShortId::compute(&ids);
    assert_eq!(ro.short_ids(), shorts);
    let r: EntryRef = shorts[2].as_str().parse().unwrap();
    assert_eq!(ro.resolve(&r).unwrap().id, a);

    assert!(ro.perms_ok());
    assert_eq!(ro.pause(), PauseState::Recording);
    assert_eq!(ro.pins_over_cap(), None);
}

#[test]
fn refused_while_the_daemon_holds_the_exclusive_lock() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    populated(&dir);
    let daemon = Store::open_exclusive(&dir, Caps::default(), RealFs).unwrap();
    assert!(matches!(
        ReadOnlyStore::open(&dir),
        Err(StoreError::DaemonRunning)
    ));
    drop(daemon);
    assert!(ReadOnlyStore::open(&dir).is_ok());
}

/// Behaviour we want and document: a reader holds a shared lock, so the daemon's exclusive
/// open reports `Locked` ("a client is reading; retry"). Retrying is spec 004's job.
#[test]
fn a_reader_blocks_the_writer_and_readers_coexist() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    populated(&dir);
    let first = ReadOnlyStore::open(&dir).unwrap();
    let second = ReadOnlyStore::open(&dir).unwrap();
    assert!(matches!(
        Store::open_exclusive(&dir, Caps::default(), RealFs),
        Err(StoreError::Locked)
    ));
    drop((first, second));
    assert!(Store::open_exclusive(&dir, Caps::default(), RealFs).is_ok());
}

#[test]
fn too_open_permissions_are_reported_but_the_store_stays_readable() {
    for (rel, mode) in [("index.json", 0o644), ("", 0o755)] {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("s");
        let [_, _, c] = populated(&dir);
        let victim = dir.join(rel);
        fs::set_permissions(&victim, fs::Permissions::from_mode(mode)).unwrap();
        let before = snapshot(&dir);

        let ro = ReadOnlyStore::open(&dir).unwrap();
        assert!(!ro.perms_ok(), "{rel:?} at {mode:o}");
        assert_eq!(ro.entries()[0].id, c);
        drop(ro);
        assert_eq!(snapshot(&dir), before, "nothing written");
        assert_eq!(
            fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            mode
        );
    }
}

#[test]
fn a_missing_store_is_an_empty_history_and_nothing_is_created() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("does-not-exist");
    let ro = ReadOnlyStore::open(&dir).unwrap();
    assert!(ro.entries().is_empty());
    assert!(ro.perms_ok());
    assert!(!dir.exists());
}

#[test]
fn an_empty_directory_gets_no_lock_file_or_content_dir() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    let ro = ReadOnlyStore::open(&dir).unwrap();
    assert!(ro.entries().is_empty());
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
}

#[test]
fn an_unknown_index_version_is_refused_and_left_untouched() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    populated(&dir);
    let future = br#"{"version":2,"entries":[],"state":{}}"#;
    fs::write(dir.join("index.json"), future).unwrap();
    fs::set_permissions(dir.join("index.json"), fs::Permissions::from_mode(0o600)).unwrap();
    let before = snapshot(&dir);
    assert!(matches!(
        ReadOnlyStore::open(&dir),
        Err(StoreError::UnknownVersion(2))
    ));
    assert_eq!(snapshot(&dir), before);
}

#[test]
fn orphans_are_ignored_and_rows_without_content_are_hidden_not_repaired() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let [a, b, c] = populated(&dir);
    fs::remove_file(dir.join("content").join(b.to_string())).unwrap();
    let stray = dir
        .join("content")
        .join(EntryId::from_parts(9, 9).to_string());
    fs::write(&stray, b"stray").unwrap();
    fs::set_permissions(&stray, fs::Permissions::from_mode(0o600)).unwrap();
    let before = snapshot(&dir);

    let ro = ReadOnlyStore::open(&dir).unwrap();
    let ids: Vec<_> = ro.entries().iter().map(|e| e.id).collect();
    assert_eq!(ids, [c, a], "the row without content is hidden");
    drop(ro);
    assert_eq!(snapshot(&dir), before, "no sweep, no rewrite");
    assert!(stray.exists());
}
