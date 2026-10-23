//! `rstory tray`: the tray as a client of the control socket, for a daemon that runs without its
//! own tray (`tray = false`). The bridge gives the tray the same [`Link`] it has in the daemon.

use std::io;
use std::path::Path;

use tokio::task::JoinHandle;

use crate::ipc::server::Link;

/// Connects to the control socket and returns a [`Link`] that forwards requests to it and
/// republishes its events. The task ends when the daemon goes away.
///
/// # Errors
/// The connection failure.
pub async fn bridge(_socket: &Path) -> io::Result<(Link, JoinHandle<()>)> {
    todo!()
}
