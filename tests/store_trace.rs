//! The durability recipe is part of the contract: pin the exact mutating-call sequence.

use std::os::unix::fs::PermissionsExt;

use rstory::config::Caps;
use rstory::domain::entry::Content;
use rstory::store::fs::{FaultFs, Op, RealFs};
use rstory::store::{Mutation, Store};
use time::OffsetDateTime;

#[test]
fn one_insert_makes_exactly_the_documented_calls() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let fs = FaultFs::recording();
    let mut store = Store::open_exclusive(&dir, Caps::default(), fs.clone()).unwrap();
    let before = fs.trace().len();

    let content = Content::new(b"trace me".to_vec(), 1 << 20).unwrap();
    let now = OffsetDateTime::from_unix_timestamp(4_102_444_800).unwrap();
    let id = store.insert(&content, now).unwrap();

    let cdir = dir.join("content");
    let ctmp = cdir.join(format!(".tmp-{id}"));
    let itmp = dir.join("index.json.tmp");
    let expected = [
        Op::CreateNew(ctmp.clone()),
        Op::WriteAll(ctmp.clone()),
        Op::SyncAll(ctmp.clone()),
        Op::Rename(ctmp, cdir.join(id.to_string())),
        Op::SyncDir(cdir),
        Op::CreateNew(itmp.clone()),
        Op::WriteAll(itmp.clone()),
        Op::SyncAll(itmp.clone()),
        Op::Rename(itmp, dir.join("index.json")),
        Op::SyncDir(dir),
    ];
    assert_eq!(fs.trace()[before..], expected[..]);
}

#[test]
fn opening_a_store_with_an_orphan_unlinks_it_and_syncs_the_directory() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    drop(Store::open_exclusive(&dir, Caps::default(), RealFs).unwrap());
    let orphan = dir
        .join("content")
        .join(rstory::domain::ids::EntryId::from_parts(7, 7).to_string());
    std::fs::write(&orphan, b"orphan").unwrap();
    std::fs::set_permissions(&orphan, std::fs::Permissions::from_mode(0o600)).unwrap();

    let fs = FaultFs::recording();
    drop(Store::open_exclusive(&dir, Caps::default(), fs.clone()).unwrap());

    assert_eq!(
        fs.trace(),
        [
            Op::RemoveFile(dir.join("index.json.tmp")), // stale-temp sweep (absent here)
            Op::RemoveFile(orphan.clone()),
            Op::SyncDir(dir.join("content")),
        ]
    );
    assert!(!orphan.exists());
}

#[test]
fn a_remove_commits_the_index_then_unlinks_and_syncs_the_content_directory() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let fs = FaultFs::recording();
    let mut store = Store::open_exclusive(&dir, Caps::default(), fs.clone()).unwrap();
    let now = OffsetDateTime::from_unix_timestamp(4_102_444_800).unwrap();
    let id = store
        .insert(&Content::new(b"gone soon".to_vec(), 1 << 20).unwrap(), now)
        .unwrap();
    let before = fs.trace().len();

    store.apply(vec![Mutation::Remove(id)], now).unwrap();

    let itmp = dir.join("index.json.tmp");
    let expected = [
        Op::CreateNew(itmp.clone()),
        Op::WriteAll(itmp.clone()),
        Op::SyncAll(itmp.clone()),
        Op::Rename(itmp, dir.join("index.json")),
        Op::SyncDir(dir.clone()),
        Op::RemoveFile(dir.join("content").join(id.to_string())),
        Op::SyncDir(dir.join("content")),
    ];
    assert_eq!(fs.trace()[before..], expected[..]);
}
