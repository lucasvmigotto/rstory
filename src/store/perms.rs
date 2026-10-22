//! Permission policy: directory 0700, files 0600; anything group- or other-accessible is refused.

use std::io;
use std::path::{Path, PathBuf};

use super::CONTENT_DIR;
use super::fs::Fs;

/// Mode of the store directory.
pub const DIR_MODE: u32 = 0o700;
/// Mode of every store file.
pub const FILE_MODE: u32 = 0o600;

const GROUP_OTHER: u32 = 0o077;

fn not_found(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::NotFound
}

/// Returns the first path (the directory, its files, `content/` and its files) that is
/// accessible to group or others, or `None`. Missing paths are fine.
///
/// # Errors
/// I/O errors other than "not found".
pub fn first_too_open<F: Fs>(fs: &F, dir: &Path) -> io::Result<Option<PathBuf>> {
    let content = dir.join(CONTENT_DIR);
    for d in [dir, content.as_path()] {
        match fs.mode(d) {
            Ok(m) if m & GROUP_OTHER != 0 => return Ok(Some(d.to_path_buf())),
            Ok(_) => {}
            Err(e) if not_found(&e) => continue,
            Err(e) => return Err(e),
        }
        for p in fs.read_dir(d)? {
            if p == content {
                continue; // checked as a directory, with its own files
            }
            match fs.mode(&p) {
                Ok(m) if m & GROUP_OTHER != 0 => return Ok(Some(p)),
                Ok(_) => {}
                Err(e) if not_found(&e) => {}
                Err(e) => return Err(e),
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::fs::RealFs;

    #[test]
    fn missing_dir_is_fine() {
        let t = tempfile::tempdir().unwrap();
        assert_eq!(
            first_too_open(&RealFs, &t.path().join("nope")).unwrap(),
            None
        );
    }

    #[test]
    fn reports_the_open_dir_then_files() {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().join("s");
        RealFs.create_dir_all(&d, DIR_MODE).unwrap();
        RealFs
            .create_dir_all(&d.join(CONTENT_DIR), DIR_MODE)
            .unwrap();
        assert_eq!(first_too_open(&RealFs, &d).unwrap(), None);
        let f = d.join(CONTENT_DIR).join("x");
        RealFs.create_new(&f, FILE_MODE).unwrap();
        assert_eq!(first_too_open(&RealFs, &d).unwrap(), None);
        RealFs.set_mode(&f, 0o640).unwrap();
        assert_eq!(first_too_open(&RealFs, &d).unwrap(), Some(f));
        RealFs.set_mode(&d, 0o755).unwrap();
        assert_eq!(first_too_open(&RealFs, &d).unwrap(), Some(d));
    }

    #[test]
    fn tighter_than_required_is_accepted() {
        let t = tempfile::tempdir().unwrap();
        RealFs.set_mode(t.path(), 0o500).unwrap();
        assert_eq!(first_too_open(&RealFs, t.path()).unwrap(), None);
        RealFs.set_mode(t.path(), 0o700).unwrap();
    }
}
