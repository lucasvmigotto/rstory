//! Fault injection for crash and power-loss tests. Compiled only for tests and the
//! `fault-injection` feature; never part of a release build.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use super::fs::{Fs, FsFile, RealFile, RealFs};

/// What [`FaultFs`] does once its budget of mutating calls is spent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// The next mutating call returns an injected error; later calls proceed normally.
    Fail,
    /// The process dies: that call and every later mutation is skipped (and returns an error);
    /// reads keep working so the on-disk state can be inspected and reopened. Everything
    /// already written stays, as it would for a killed process on a live kernel.
    Crash,
    /// The machine loses power. Like `Crash`, and in addition whatever was not made durable is
    /// rolled back on the real filesystem (see [`FaultFs`]).
    PowerLoss,
}

/// A mutating call, as recorded by [`FaultFs::trace`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// `create_dir_all`
    CreateDirAll(PathBuf),
    /// `create_new`
    CreateNew(PathBuf),
    /// `write_all` on the file at this path
    WriteAll(PathBuf),
    /// `sync_all` on the file at this path
    SyncAll(PathBuf),
    /// `rename(from, to)`
    Rename(PathBuf, PathBuf),
    /// `sync_dir`
    SyncDir(PathBuf),
    /// `remove_file`
    RemoveFile(PathBuf),
    /// `set_mode`
    SetMode(PathBuf),
}

impl Op {
    fn name(&self) -> &'static str {
        match self {
            Self::CreateDirAll(_) => "create_dir_all",
            Self::CreateNew(_) => "create_new",
            Self::WriteAll(_) => "write_all",
            Self::SyncAll(_) => "sync_all",
            Self::Rename(..) => "rename",
            Self::SyncDir(_) => "sync_dir",
            Self::RemoveFile(_) => "remove_file",
            Self::SetMode(_) => "set_mode",
        }
    }
}

/// A directory-entry change that is not durable until the directory is synced.
#[derive(Debug)]
enum DirOp {
    /// A file was created; undo removes it.
    Create { path: PathBuf },
    /// A file was renamed over `to`, whose durable content was `replaced`; undo renames back
    /// and restores that content.
    Rename {
        from: PathBuf,
        to: PathBuf,
        replaced: Option<Vec<u8>>,
    },
    /// A file was unlinked; undo recreates it with its durable content.
    Remove {
        path: PathBuf,
        backup: Option<Vec<u8>>,
    },
}

impl DirOp {
    fn dir(&self) -> Option<&Path> {
        match self {
            Self::Create { path } | Self::Remove { path, .. } => path.parent(),
            Self::Rename { to, .. } => to.parent(),
        }
    }
}

/// What has been done and is not yet durable.
#[derive(Debug, Default)]
struct Journal {
    trace: Vec<Op>,
    /// Files with data written since their last `sync_all`: bytes that are durable.
    synced_len: HashMap<PathBuf, u64>,
    /// Directory-entry changes since the last `sync_dir` of their directory, oldest first.
    dir_ops: Vec<DirOp>,
}

impl Journal {
    /// The bytes of `path` that would survive a power loss now.
    fn durable_content(&self, path: &Path) -> Option<Vec<u8>> {
        let mut bytes = fs::read(path).ok()?;
        if let Some(&n) = self.synced_len.get(path) {
            bytes.truncate(usize::try_from(n).unwrap_or(usize::MAX));
        }
        Some(bytes)
    }

    /// Reverts the real filesystem to its durable state.
    fn roll_back(&mut self) {
        for (path, len) in self.synced_len.drain() {
            if let Ok(f) = OpenOptions::new().write(true).open(&path) {
                let _ = f.set_len(len);
            }
        }
        while let Some(op) = self.dir_ops.pop() {
            match op {
                DirOp::Create { path } => {
                    let _ = fs::remove_file(path);
                }
                DirOp::Rename { from, to, replaced } => {
                    let _ = fs::rename(&to, &from);
                    if let Some(bytes) = replaced {
                        restore(&to, &bytes);
                    }
                }
                DirOp::Remove { path, backup } => {
                    if let Some(bytes) = backup {
                        restore(&path, &bytes);
                    }
                }
            }
        }
    }
}

fn restore(path: &Path, bytes: &[u8]) {
    if let Ok(mut f) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
    {
        let _ = f.write_all(bytes);
    }
}

#[derive(Debug)]
struct FaultState {
    /// Mutating calls allowed to succeed before the fault.
    budget: u64,
    fault: Fault,
    calls: AtomicU64,
    crashed: AtomicBool,
    journal: Mutex<Journal>,
}

impl FaultState {
    fn journal(&self) -> MutexGuard<'_, Journal> {
        self.journal.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Counts one mutating call; `Err` means the call must not touch the disk.
    fn gate(&self, op: Op) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(crash_error(op.name()));
        }
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        let fail = self.fault == Fault::Fail;
        if n < self.budget || (fail && n > self.budget) {
            self.journal().trace.push(op);
            return Ok(());
        }
        if fail {
            return Err(io::Error::other(format!(
                "injected fault at mutating call {} ({})",
                n + 1,
                op.name()
            )));
        }
        self.power_off();
        Err(crash_error(op.name()))
    }

    fn power_off(&self) {
        if !self.crashed.swap(true, Ordering::SeqCst) && self.fault == Fault::PowerLoss {
            self.journal().roll_back();
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
///
/// # Power-loss model
///
/// With [`Fault::PowerLoss`] the wrapper journals what is not yet durable and, at the crash
/// point (or on [`FaultFs::power_off`]), undoes it on the real filesystem:
///
/// - file data written since the file's last `sync_all` is truncated away (a file never synced
///   ends up empty, if its directory entry survives at all);
/// - directory-entry changes (`create_new`, `rename`, `remove_file`) not followed by `sync_dir`
///   of their directory are reverted: an unsynced create disappears, an unsynced rename goes
///   back to the old name and the replaced file reappears with its durable content, an
///   unsynced unlink reappears with its durable content.
///
/// Deliberately simple: `create_dir_all` and `set_mode` are treated as durable, a file handle
/// is assumed to be dropped before its file is renamed, and renames stay inside one directory.
/// That is all the store does.
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
                journal: Mutex::new(Journal::default()),
            }),
        }
    }

    /// A pass-through that only records: never faults.
    #[must_use]
    pub fn recording() -> Self {
        Self::new(u64::MAX, Fault::Crash)
    }

    /// The mutating calls that were allowed to run, in order.
    #[must_use]
    pub fn trace(&self) -> Vec<Op> {
        self.state.journal().trace.clone()
    }

    /// Simulates the process or machine dying now, after the last call (for `PowerLoss`,
    /// rolling back whatever is not durable). Has no effect if the fault already happened.
    pub fn power_off(&self) {
        self.state.power_off();
    }

    /// Number of mutating calls attempted so far (including the faulted ones).
    #[must_use]
    pub fn mutations(&self) -> u64 {
        self.state.calls.load(Ordering::SeqCst)
    }

    /// Whether a [`Fault::Crash`] or [`Fault::PowerLoss`] has happened.
    #[must_use]
    pub fn crashed(&self) -> bool {
        self.state.crashed.load(Ordering::SeqCst)
    }
}

/// A file opened by [`FaultFs`].
#[derive(Debug)]
pub struct FaultFile {
    inner: RealFile,
    path: PathBuf,
    state: Arc<FaultState>,
}

impl FsFile for FaultFile {
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        self.state.gate(Op::WriteAll(self.path.clone()))?;
        let len = fs::metadata(&self.path).map_or(0, |m| m.len());
        self.state
            .journal()
            .synced_len
            .entry(self.path.clone())
            .or_insert(len);
        self.inner.write_all(data)
    }

    fn sync_all(&mut self) -> io::Result<()> {
        self.state.gate(Op::SyncAll(self.path.clone()))?;
        self.inner.sync_all()?;
        self.state.journal().synced_len.remove(&self.path);
        Ok(())
    }
}

impl Fs for FaultFs {
    type File = FaultFile;

    fn create_dir_all(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.state.gate(Op::CreateDirAll(path.to_path_buf()))?;
        self.inner.create_dir_all(path, mode)
    }

    fn create_new(&self, path: &Path, mode: u32) -> io::Result<FaultFile> {
        self.state.gate(Op::CreateNew(path.to_path_buf()))?;
        let inner = self.inner.create_new(path, mode)?;
        let mut j = self.state.journal();
        j.synced_len.insert(path.to_path_buf(), 0);
        j.dir_ops.push(DirOp::Create {
            path: path.to_path_buf(),
        });
        Ok(FaultFile {
            inner,
            path: path.to_path_buf(),
            state: Arc::clone(&self.state),
        })
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.state
            .gate(Op::Rename(from.to_path_buf(), to.to_path_buf()))?;
        let replaced = self.state.journal().durable_content(to);
        self.inner.rename(from, to)?;
        let mut j = self.state.journal();
        if let Some(n) = j.synced_len.remove(from) {
            j.synced_len.insert(to.to_path_buf(), n);
        } else {
            j.synced_len.remove(to);
        }
        j.dir_ops.push(DirOp::Rename {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
            replaced,
        });
        Ok(())
    }

    fn sync_dir(&self, path: &Path) -> io::Result<()> {
        self.state.gate(Op::SyncDir(path.to_path_buf()))?;
        self.inner.sync_dir(path)?;
        self.state
            .journal()
            .dir_ops
            .retain(|op| op.dir() != Some(path));
        Ok(())
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.inner.read(path)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.state.gate(Op::RemoveFile(path.to_path_buf()))?;
        let backup = self.state.journal().durable_content(path);
        self.inner.remove_file(path)?;
        let mut j = self.state.journal();
        j.synced_len.remove(path);
        j.dir_ops.push(DirOp::Remove {
            path: path.to_path_buf(),
            backup,
        });
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        self.inner.read_dir(path)
    }

    fn len(&self, path: &Path) -> io::Result<u64> {
        self.inner.len(path)
    }

    fn mode(&self, path: &Path) -> io::Result<u32> {
        self.inner.mode(path)
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.state.gate(Op::SetMode(path.to_path_buf()))?;
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

    fn write_file<F: Fs>(fs: &F, path: &Path, data: &[u8], sync: bool) {
        let mut f = fs.create_new(path, 0o600).unwrap();
        f.write_all(data).unwrap();
        if sync {
            f.sync_all().unwrap();
        }
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

    #[test]
    fn power_loss_drops_a_file_whose_directory_was_not_synced() {
        let t = tempfile::tempdir().unwrap();
        let fs = FaultFs::new(u64::MAX, Fault::PowerLoss);
        write_file(&fs, &t.path().join("a"), b"data", true);
        write_file(&fs, &t.path().join("b"), b"data", true);
        fs.sync_dir(t.path()).unwrap();
        write_file(&fs, &t.path().join("c"), b"data", true); // synced data, unsynced entry
        fs.power_off();
        assert!(t.path().join("a").exists() && t.path().join("b").exists());
        assert!(!t.path().join("c").exists());
    }

    #[test]
    fn power_loss_truncates_unsynced_data() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("a");
        let fs = FaultFs::new(u64::MAX, Fault::PowerLoss);
        let mut f = fs.create_new(&p, 0o600).unwrap();
        f.write_all(b"durable").unwrap();
        f.sync_all().unwrap();
        f.write_all(b"-volatile").unwrap();
        fs.sync_dir(t.path()).unwrap();
        fs.power_off();
        assert_eq!(fs::read(&p).unwrap(), b"durable");

        let q = t.path().join("never-synced");
        let fs = FaultFs::new(u64::MAX, Fault::PowerLoss);
        write_file(&fs, &q, b"lost", false);
        fs.sync_dir(t.path()).unwrap(); // the entry survives, the data does not
        fs.power_off();
        assert_eq!(fs::read(&q).unwrap(), b"");
    }

    #[test]
    fn power_loss_reverts_an_unsynced_rename_and_restores_the_replaced_file() {
        let t = tempfile::tempdir().unwrap();
        let (old, tmp) = (t.path().join("index"), t.path().join("index.tmp"));
        let fs = FaultFs::new(u64::MAX, Fault::PowerLoss);
        write_file(&fs, &old, b"old", true);
        fs.sync_dir(t.path()).unwrap();
        write_file(&fs, &tmp, b"new", true);
        fs.rename(&tmp, &old).unwrap();
        assert_eq!(fs::read(&old).unwrap(), b"new");
        fs.power_off();
        assert_eq!(fs::read(&old).unwrap(), b"old");
        assert!(!tmp.exists(), "the unsynced temp file disappears too");
    }

    #[test]
    fn power_loss_keeps_a_synced_rename() {
        let t = tempfile::tempdir().unwrap();
        let (old, tmp) = (t.path().join("index"), t.path().join("index.tmp"));
        let fs = FaultFs::new(u64::MAX, Fault::PowerLoss);
        write_file(&fs, &old, b"old", true);
        write_file(&fs, &tmp, b"new", true);
        fs.rename(&tmp, &old).unwrap();
        fs.sync_dir(t.path()).unwrap();
        fs.power_off();
        assert_eq!(fs::read(&old).unwrap(), b"new");
    }

    #[test]
    fn power_loss_brings_back_an_unsynced_unlink() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("a");
        let fs = FaultFs::new(u64::MAX, Fault::PowerLoss);
        write_file(&fs, &p, b"keep me", true);
        fs.sync_dir(t.path()).unwrap();
        fs.remove_file(&p).unwrap();
        assert!(!p.exists());
        fs.power_off();
        assert_eq!(fs::read(&p).unwrap(), b"keep me");
    }

    #[test]
    fn power_loss_at_the_budget_rolls_back_and_blocks_later_calls() {
        let t = tempfile::tempdir().unwrap();
        let fs = FaultFs::new(3, Fault::PowerLoss);
        // create_new, write_all, sync_all ok; rename is the crash point.
        assert!(atomic_write(&fs, t.path(), "f", b"data").is_err());
        assert!(fs.crashed());
        assert!(!t.path().join("f").exists());
        assert!(
            !t.path().join("f.tmp").exists(),
            "unsynced create rolled back"
        );
    }

    #[test]
    fn crash_keeps_unsynced_state_unlike_power_loss() {
        let t = tempfile::tempdir().unwrap();
        let fs = FaultFs::new(u64::MAX, Fault::Crash);
        write_file(&fs, &t.path().join("a"), b"data", false);
        fs.power_off();
        assert_eq!(fs::read(t.path().join("a")).unwrap(), b"data");
    }

    #[test]
    fn recording_lists_the_calls_in_order() {
        let t = tempfile::tempdir().unwrap();
        let fs = FaultFs::recording();
        atomic_write(&fs, t.path(), "f", b"x").unwrap();
        let (tmp, f) = (t.path().join("f.tmp"), t.path().join("f"));
        assert_eq!(
            fs.trace(),
            [
                Op::CreateNew(tmp.clone()),
                Op::WriteAll(tmp.clone()),
                Op::SyncAll(tmp.clone()),
                Op::Rename(tmp, f),
                Op::SyncDir(t.path().to_path_buf()),
            ]
        );
    }
}
