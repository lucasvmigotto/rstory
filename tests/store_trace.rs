//! The durability recipe is part of the contract: pin the exact mutating-call sequence.

use rstory::config::Caps;
use rstory::domain::entry::Content;
use rstory::store::Store;
use rstory::store::fs::{FaultFs, Op};
use time::OffsetDateTime;

#[test]
fn one_insert_makes_exactly_the_documented_calls() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    let fs = FaultFs::recording();
    let mut store = Store::open_exclusive(&dir, Caps::default(), fs.clone()).unwrap();
    let before = fs.trace().len();

    let content = Content::new(b"trace me".to_vec(), 1 << 20).unwrap();
    let now = OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap();
    let id = store.insert(&content, now).unwrap();

    let cdir = dir.join("content");
    let ctmp = cdir.join(format!(".tmp-{id}"));
    let itmp = dir.join("index.json.tmp");
    let expected = vec![
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
