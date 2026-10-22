//! The filesystem surface the store needs, with a real and a fault-injecting implementation.
//!
//! The atomic-write recipe is: `create_new` (mode 0600) in the target directory, `write_all`,
//! `sync_all`, `rename`, `sync_dir`. Every step is a separate call so that [`FaultFs`] can fail
//! or "lose power" between any two of them.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

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

    fn mode(&self, path: &Path) -> io::Result<u32> {
        Ok(fs::metadata(path)?.mode() & 0o7777)
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }
}

/// What [`FaultFs`] does once its budget of mutating calls is spent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// The next mutating call returns an injected error; later calls proceed normally.
    Fail,
    /// Power loss: that call and every later mutation is skipped (and returns an error);
    /// reads keep working so the on-disk state can be inspected and reopened.
    Crash,
}

#[derive(Debug)]
struct FaultState {
    /// Mutating calls allowed to succeed before the fault.
    budget: u64,
    fault: Fault,
    calls: AtomicU64,
    crashed: AtomicBool,
}

impl FaultState {
    /// Counts one mutating call; `Err` means the call must not touch the disk.
    fn gate(&self, what: &str) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(crash_error(what));
        }
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        if n < self.budget {
            return Ok(());
        }
        match self.fault {
            Fault::Fail if n == self.budget => Err(io::Error::other(format!(
                "injected fault at mutating call {} ({what})",
                n + 1
            ))),
            Fault::Fail => Ok(()),
            Fault::Crash => {
                self.crashed.store(true, Ordering::SeqCst);
                Err(crash_error(what))
            }
        }
    }
}

fn crash_error(what: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::BrokenPipe,
        format!("simulated crash before {what}"),
    )
}

/// Wraps [`RealFs`] and injects a fault after the N-th mutating call.
///
/// Mutating calls are: `create_dir_all`, `create_new`, `write_all`, `sync_all`, `rename`,
/// `sync_dir`, `remove_file`, `set_mode`.
#[derive(Debug, Clone)]
pub struct FaultFs {
    inner: RealFs,
    state: Arc<FaultState>,
}

impl FaultFs {
    /// Lets `succeed` mutating calls through, then applies `fault`.
    #[must_use]
    pub fn new(succeed: u64, fault: Fault) -> Self {
        Self {
            inner: RealFs,
            state: Arc::new(FaultState {
                budget: succeed,
                fault,
                calls: AtomicU64::new(0),
                crashed: AtomicBool::new(false),
            }),
        }
    }

    /// Number of mutating calls attempted so far (including the faulted ones).
    #[must_use]
    pub fn mutations(&self) -> u64 {
        self.state.calls.load(Ordering::SeqCst)
    }

    /// Whether a [`Fault::Crash`] has happened.
    #[must_use]
    pub fn crashed(&self) -> bool {
        self.state.crashed.load(Ordering::SeqCst)
    }
}

/// A file opened by [`FaultFs`].
#[derive(Debug)]
pub struct FaultFile {
    inner: RealFile,
    state: Arc<FaultState>,
}

impl FsFile for FaultFile {
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        self.state.gate("write_all")?;
        self.inner.write_all(data)
    }

    fn sync_all(&mut self) -> io::Result<()> {
        self.state.gate("sync_all")?;
        self.inner.sync_all()
    }
}

impl Fs for FaultFs {
    type File = FaultFile;

    fn create_dir_all(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.state.gate("create_dir_all")?;
        self.inner.create_dir_all(path, mode)
    }

    fn create_new(&self, path: &Path, mode: u32) -> io::Result<FaultFile> {
        self.state.gate("create_new")?;
        let inner = self.inner.create_new(path, mode)?;
        Ok(FaultFile {
            inner,
            state: Arc::clone(&self.state),
        })
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.state.gate("rename")?;
        self.inner.rename(from, to)
    }

    fn sync_dir(&self, path: &Path) -> io::Result<()> {
        self.state.gate("sync_dir")?;
        self.inner.sync_dir(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.inner.read(path)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.state.gate("remove_file")?;
        self.inner.remove_file(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        self.inner.read_dir(path)
    }

    fn mode(&self, path: &Path) -> io::Result<u32> {
        self.inner.mode(path)
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.state.gate("set_mode")?;
        self.inner.set_mode(path, mode)
    }
}

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

    #[test]
    fn fault_fs_counts_mutations_without_faulting_within_budget() {
        let t = tempfile::tempdir().unwrap();
        let fs = FaultFs::new(5, Fault::Crash);
        atomic_write(&fs, t.path(), "f", b"data").unwrap();
        assert_eq!(fs.mutations(), 5);
        assert!(!fs.crashed());
        assert_eq!(fs.read(&t.path().join("f")).unwrap(), b"data");
    }

    #[test]
    fn fault_fs_fail_errors_once_then_recovers() {
        let t = tempfile::tempdir().unwrap();
        let fs = FaultFs::new(2, Fault::Fail);
        let e = atomic_write(&fs, t.path(), "f", b"data").unwrap_err();
        assert!(e.to_string().contains("injected fault"));
        // The temp file was created and written but never synced or renamed.
        assert!(!t.path().join("f").exists());
        assert!(t.path().join("f.tmp").exists());
        // Later calls proceed.
        fs.remove_file(&t.path().join("f.tmp")).unwrap();
        assert!(!fs.crashed());
    }

    #[test]
    fn fault_fs_crash_stops_all_further_mutations_but_not_reads() {
        let t = tempfile::tempdir().unwrap();
        let fs = FaultFs::new(3, Fault::Crash);
        // Dies at the 4th call (rename): the temp file exists, the target does not.
        assert!(atomic_write(&fs, t.path(), "f", b"data").is_err());
        assert!(fs.crashed());
        assert!(fs.remove_file(&t.path().join("f.tmp")).is_err());
        assert!(t.path().join("f.tmp").exists());
        assert!(fs.create_new(&t.path().join("g"), 0o600).is_err());
        assert!(!t.path().join("g").exists());
        assert_eq!(fs.read(&t.path().join("f.tmp")).unwrap(), b"data");
        assert_eq!(fs.read_dir(t.path()).unwrap().len(), 1);
    }

    #[test]
    fn fault_fs_crash_at_zero_touches_nothing() {
        let t = tempfile::tempdir().unwrap();
        let fs = FaultFs::new(0, Fault::Crash);
        assert!(fs.create_dir_all(&t.path().join("d"), 0o700).is_err());
        assert!(!t.path().join("d").exists());
    }
}
