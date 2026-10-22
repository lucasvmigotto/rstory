//! The filesystem surface the store needs, with a real implementation.
//!
//! The atomic-write recipe is: `create_new` (mode 0600) in the target directory, `write_all`,
//! `sync_all`, `rename`, `sync_dir`. Every step is a separate call so that the fault-injecting
//! `FaultFs` (see `store::fault`) can fail or "lose power" between any two of them.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// An open, writable file created through [`Fs::create_new`].
pub trait FsFile {
    /// Writes the whole buffer.
    fn write_all(&mut self, data: &[u8]) -> io::Result<()>;
    /// Flushes data and metadata to stable storage.
    fn sync_all(&mut self) -> io::Result<()>;
}

/// What the store needs from a filesystem, and nothing more.
pub trait Fs {
    /// The file handle type.
    type File: FsFile;

    /// Creates `path` and missing parents; the final directory ends up with `mode`.
    fn create_dir_all(&self, path: &Path, mode: u32) -> io::Result<()>;
    /// Creates a new file (failing if it exists) with `mode`.
    fn create_new(&self, path: &Path, mode: u32) -> io::Result<Self::File>;
    /// Atomically renames `from` to `to`.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    /// Flushes a directory's entries to stable storage.
    fn sync_dir(&self, path: &Path) -> io::Result<()>;
    /// Reads a whole file.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    /// Removes a file.
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    /// Lists the entries of a directory as full paths, sorted.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
    /// Returns the size of a file in bytes.
    fn len(&self, path: &Path) -> io::Result<u64>;
    /// Returns the permission bits (`mode & 0o7777`) of a path.
    fn mode(&self, path: &Path) -> io::Result<u32>;
    /// Sets the permission bits of a path.
    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()>;
}

/// The real filesystem (std, with rustix for `fsync`).
#[derive(Debug, Clone, Copy, Default)]
pub struct RealFs;

/// A file opened by [`RealFs`].
#[derive(Debug)]
pub struct RealFile(fs::File);

impl FsFile for RealFile {
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        self.0.write_all(data)
    }

    fn sync_all(&mut self) -> io::Result<()> {
        rustix::fs::fsync(&self.0).map_err(io::Error::from)
    }
}

impl Fs for RealFs {
    type File = RealFile;

    fn create_dir_all(&self, path: &Path, mode: u32) -> io::Result<()> {
        DirBuilder::new().recursive(true).mode(mode).create(path)?;
        // The umask may have narrowed the mode, and an existing directory keeps its own.
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }

    fn create_new(&self, path: &Path, mode: u32) -> io::Result<RealFile> {
        let f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .open(path)?;
        Ok(RealFile(f))
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn sync_dir(&self, path: &Path) -> io::Result<()> {
        let d = fs::File::open(path)?;
        rustix::fs::fsync(&d).map_err(io::Error::from)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut v = fs::read_dir(path)?
            .map(|e| e.map(|e| e.path()))
            .collect::<io::Result<Vec<_>>>()?;
        v.sort();
        Ok(v)
    }

    fn len(&self, path: &Path) -> io::Result<u64> {
        Ok(fs::metadata(path)?.len())
    }

    fn mode(&self, path: &Path) -> io::Result<u32> {
        Ok(fs::metadata(path)?.mode() & 0o7777)
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }
}

#[cfg(any(test, feature = "fault-injection"))]
pub use super::fault::{Fault, FaultFile, FaultFs, Op};

#[cfg(test)]
mod tests {
    use super::*;

    /// The full atomic-write recipe; 5 mutating calls.
    fn atomic_write<F: Fs>(fs: &F, dir: &Path, name: &str, data: &[u8]) -> io::Result<()> {
        let tmp = dir.join(format!("{name}.tmp"));
        let mut f = fs.create_new(&tmp, 0o600)?;
        f.write_all(data)?;
        f.sync_all()?;
        fs.rename(&tmp, &dir.join(name))?;
        fs.sync_dir(dir)
    }

    #[test]
    fn real_fs_atomic_write_roundtrip_with_modes() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("a/b");
        RealFs.create_dir_all(&dir, 0o700).unwrap();
        assert_eq!(RealFs.mode(&dir).unwrap(), 0o700);
        atomic_write(&RealFs, &dir, "index.json", b"{}").unwrap();
        let p = dir.join("index.json");
        assert_eq!(RealFs.read(&p).unwrap(), b"{}");
        assert_eq!(RealFs.mode(&p).unwrap(), 0o600);
        assert_eq!(RealFs.read_dir(&dir).unwrap(), vec![p.clone()]);
        RealFs.set_mode(&p, 0o640).unwrap();
        assert_eq!(RealFs.mode(&p).unwrap(), 0o640);
        RealFs.remove_file(&p).unwrap();
        assert!(RealFs.read(&p).is_err());
    }

    #[test]
    fn real_fs_create_new_refuses_existing_file() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("x");
        RealFs.create_new(&p, 0o600).unwrap();
        let e = RealFs.create_new(&p, 0o600).err().unwrap();
        assert_eq!(e.kind(), io::ErrorKind::AlreadyExists);
    }
}
