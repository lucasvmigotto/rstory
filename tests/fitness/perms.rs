//! F9: directory 0700, files 0600; a store that is too open is refused and left untouched.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use rstory::config::Caps;
use rstory::domain::entry::Content;
use rstory::store::fs::RealFs;
use rstory::store::{Store, StoreError};
use time::OffsetDateTime;

fn mode(p: &Path) -> u32 {
    fs::metadata(p).unwrap().permissions().mode() & 0o7777
}

fn open(dir: &Path) -> Result<Store<RealFs>, StoreError> {
    Store::open_exclusive(dir, Caps::default(), RealFs)
}

fn populate(dir: &Path) {
    let mut s = open(dir).unwrap();
    for i in 0..3 {
        let c = Content::new(format!("entry {i}").into_bytes(), 1 << 20).unwrap();
        s.insert(
            &c,
            OffsetDateTime::from_unix_timestamp(4_102_444_800 + i).unwrap(),
        )
        .unwrap();
    }
}

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        out.push(p.clone());
        if p.is_dir() {
            walk(&p, out);
        }
    }
}

#[test]
fn created_store_is_0700_dirs_and_0600_files() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    populate(&dir);
    assert_eq!(mode(&dir), 0o700);
    let mut all = Vec::new();
    walk(&dir, &mut all);
    assert!(all.len() >= 6, "lock, index, content dir and 3 files");
    for p in all {
        let want = if p.is_dir() { 0o700 } else { 0o600 };
        assert_eq!(mode(&p), want, "{}", p.display());
    }
}

#[test]
fn open_directory_is_refused_and_nothing_is_written() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();

    match open(&dir) {
        Err(StoreError::StoreTooOpen { path }) => assert_eq!(path, dir),
        other => panic!("expected StoreTooOpen, got {:?}", other.map(|_| ())),
    }
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
    assert_eq!(
        mode(&dir),
        0o755,
        "permissions are reported, not silently fixed"
    );
}

#[test]
fn open_files_are_refused_and_named() {
    for rel in ["index.json", "store.lock", "content/@ENTRY"] {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("s");
        populate(&dir);
        let rel = if rel.ends_with("@ENTRY") {
            let first = fs::read_dir(dir.join("content"))
                .unwrap()
                .next()
                .unwrap()
                .unwrap();
            format!("content/{}", first.file_name().to_str().unwrap())
        } else {
            rel.to_string()
        };
        let victim = dir.join(&rel);
        fs::set_permissions(&victim, fs::Permissions::from_mode(0o644)).unwrap();
        let before = fs::read(&victim).unwrap();

        match open(&dir) {
            Err(StoreError::StoreTooOpen { path }) => assert_eq!(path, victim),
            other => panic!("{rel}: expected StoreTooOpen, got {:?}", other.map(|_| ())),
        }
        assert_eq!(fs::read(&victim).unwrap(), before);
        assert_eq!(mode(&victim), 0o644);
    }
}

#[test]
fn open_content_directory_is_refused() {
    let t = tempfile::tempdir().unwrap();
    let dir = t.path().join("s");
    populate(&dir);
    let c = dir.join("content");
    fs::set_permissions(&c, fs::Permissions::from_mode(0o750)).unwrap();
    match open(&dir) {
        Err(StoreError::StoreTooOpen { path }) => assert_eq!(path, c),
        other => panic!("expected StoreTooOpen, got {:?}", other.map(|_| ())),
    }
}
