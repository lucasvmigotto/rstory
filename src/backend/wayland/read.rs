//! Reading the current selection (FR-003, FR-004).
//!
//! `receive(mime, fd)` hands the owner the write end of a pipe; the read end is a tokio pipe
//! receiver, so a slow or stuck owner only parks its own task, never the connection loop. The
//! byte limit aborts the transfer (dropping the read end gives the owner `EPIPE`), and the
//! whole read has one deadline. Only the MIME type, size and duration are logged.

use std::os::fd::AsFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use rustix::pipe::{PipeFlags, pipe_with};
use tokio::io::AsyncReadExt;
use tokio::net::unix::pipe;
use tokio::sync::oneshot;

use super::State;
use crate::backend::ReadError;

/// Preference order of text types (FR-003).
pub const PREFERENCE: [&str; 5] = [
    "text/plain;charset=utf-8",
    "UTF8_STRING",
    "text/plain",
    "STRING",
    "TEXT",
];

/// The best text type among `types`, if any.
#[must_use]
pub fn pick_mime(types: &[String]) -> Option<&str> {
    PREFERENCE.iter().find_map(|want| {
        types
            .iter()
            .find(|t| t.eq_ignore_ascii_case(want))
            .map(String::as_str)
    })
}

/// UTF-8 text from `bytes`; invalid sequences become U+FFFD.
#[must_use]
pub fn decode(bytes: Vec<u8>) -> Vec<u8> {
    match String::from_utf8(bytes) {
        Ok(s) => s.into_bytes(),
        Err(e) => String::from_utf8_lossy(e.as_bytes())
            .into_owned()
            .into_bytes(),
    }
}

/// Lets a running read notice that its offer is stale or the connection is gone.
#[derive(Debug, Default)]
pub struct Epoch {
    selection: AtomicU64,
    dead: AtomicBool,
}

impl Epoch {
    /// A new selection replaced the one being read.
    pub fn bump(&self) {
        self.selection.fetch_add(1, Ordering::SeqCst);
    }

    /// The connection is gone.
    pub fn kill(&self) {
        self.dead.store(true, Ordering::SeqCst);
    }

    fn now(&self) -> u64 {
        self.selection.load(Ordering::SeqCst)
    }
}

/// Read `rx` to EOF, aborting past `limit` bytes or after `timeout`.
async fn drain(
    mut rx: pipe::Receiver,
    limit: usize,
    timeout: Duration,
) -> Result<Vec<u8>, ReadError> {
    let work = async {
        let mut out = Vec::new();
        let mut chunk = vec![0u8; 64 * 1024];
        loop {
            let n = rx
                .read(&mut chunk)
                .await
                .map_err(|_| ReadError::ConversionRefused)?;
            if n == 0 {
                return Ok(out);
            }
            if out.len() + n > limit {
                return Err(ReadError::TooLarge);
            }
            out.extend_from_slice(&chunk[..n]);
        }
    };
    tokio::time::timeout(timeout, work)
        .await
        .unwrap_or(Err(ReadError::Timeout))
}

impl State {
    /// Start a read of the current selection; the reply arrives from a task.
    pub(super) fn cmd_read(
        &mut self,
        limit: usize,
        reply: oneshot::Sender<Result<Vec<u8>, ReadError>>,
    ) {
        if self.dead {
            let _ = reply.send(Err(ReadError::Disconnected));
            return;
        }
        let Some(offer) = self.current.clone() else {
            let _ = reply.send(Err(ReadError::NoText));
            return;
        };
        let types = offer.mimes();
        let Some(mime) = pick_mime(&types).map(str::to_string) else {
            let _ = reply.send(Err(ReadError::NoText));
            return;
        };
        let opened = pipe_with(PipeFlags::CLOEXEC)
            .map_err(|_| ())
            .and_then(|(r, w)| {
                let rx = pipe::Receiver::from_owned_fd(r).map_err(|_| ())?;
                Ok((rx, w))
            });
        let Ok((rx, write_end)) = opened else {
            let _ = reply.send(Err(ReadError::ConversionRefused));
            return;
        };
        offer.receive(&mime, write_end.as_fd());
        // The compositor holds its own copy once the request is flushed; ours must go, or the
        // read would never see EOF.
        drop(write_end);
        let epoch = Arc::clone(&self.epoch);
        let seen = epoch.now();
        let timeout = self.read_timeout;
        tokio::spawn(async move {
            let started = Instant::now();
            let mut result = drain(rx, limit, timeout).await;
            if epoch.dead.load(Ordering::SeqCst) {
                result = Err(ReadError::Disconnected);
            } else if epoch.now() != seen && result.is_ok() {
                result = Err(ReadError::ConversionRefused);
            }
            tracing::debug!(
                mime = %mime,
                bytes = result.as_ref().map_or(0, Vec::len),
                ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                ok = result.is_ok(),
                "wl.read"
            );
            let _ = reply.send(result.map(decode));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn preference_order_is_fixed() {
        let all = v(&[
            "TEXT",
            "STRING",
            "text/plain",
            "UTF8_STRING",
            "text/plain;charset=utf-8",
        ]);
        assert_eq!(pick_mime(&all), Some("text/plain;charset=utf-8"));
        assert_eq!(pick_mime(&all[..4]), Some("UTF8_STRING"));
        assert_eq!(pick_mime(&all[..3]), Some("text/plain"));
        assert_eq!(pick_mime(&all[..2]), Some("STRING"));
        assert_eq!(pick_mime(&all[..1]), Some("TEXT"));
        assert_eq!(pick_mime(&v(&["image/png", "text/html"])), None);
        assert_eq!(pick_mime(&[]), None);
    }

    #[test]
    fn charset_case_does_not_matter() {
        assert_eq!(
            pick_mime(&v(&["text/plain;charset=UTF-8"])),
            Some("text/plain;charset=UTF-8")
        );
    }

    #[test]
    fn invalid_utf8_is_replaced() {
        assert_eq!(decode(b"ab\xffcd".to_vec()), "ab\u{fffd}cd".as_bytes());
        assert_eq!(decode(b"plain".to_vec()), b"plain");
    }
}
