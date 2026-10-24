//! `SO_PEERCRED`: who is on the other end of a Unix socket (Constitution III).

use std::io;

use tokio::net::UnixStream;

/// The kernel-reported credentials of a peer at connect time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerCred {
    /// Process id of the peer.
    pub pid: u32,
    /// User id of the peer.
    pub uid: u32,
}

/// Credentials of the process that connected `stream` (or of the listener, for a client socket).
///
/// # Errors
/// The `getsockopt` failure.
pub fn peer_cred(stream: &UnixStream) -> io::Result<PeerCred> {
    let cred = rustix::net::sockopt::socket_peercred(stream)?;
    Ok(PeerCred {
        pid: u32::try_from(cred.pid.as_raw_nonzero().get()).unwrap_or(0),
        uid: cred.uid.as_raw(),
    })
}
