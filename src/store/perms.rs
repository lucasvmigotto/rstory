//! Permission policy: directory 0700, files 0600; anything group- or other-accessible is refused.

use std::io;
use std::path::{Path, PathBuf};

use super::fs::Fs;

/// Mode of the store directory.
pub const DIR_MODE: u32 = 0o700;
/// Mode of every store file.
pub const FILE_MODE: u32 = 0o600;

/// Returns the first path (the directory, its files, `content/` and its files) that is
/// accessible to group or others, or `None`. Missing paths are fine.
///
/// # Errors
/// I/O errors other than "not found".
pub fn first_too_open<F: Fs>(fs: &F, dir: &Path) -> io::Result<Option<PathBuf>> {
    unimplemented!("T022")
}
