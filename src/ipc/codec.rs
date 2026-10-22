//! NDJSON framing: one JSON object per line, lines capped at [`MAX_LINE_BYTES`].

use std::io;

use serde::Serialize;
use tokio::io::AsyncRead;

use super::protocol::MAX_LINE_BYTES;

const CHUNK: usize = 8 * 1024;

/// Why reading a line failed.
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    /// A line exceeded the limit. The stream is unusable afterwards.
    #[error("line longer than the limit")]
    TooLong,
    /// The transport failed.
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
}

/// Reads newline-terminated lines without ever buffering more than the limit plus one chunk.
#[derive(Debug)]
pub struct LineReader<R> {
    inner: R,
    buf: Vec<u8>,
    scanned: usize,
    max: usize,
}

impl<R: AsyncRead + Unpin> LineReader<R> {
    /// A reader with the protocol limit.
    #[must_use]
    pub fn new(inner: R) -> Self {
        Self::with_limit(inner, MAX_LINE_BYTES)
    }

    /// A reader with a custom limit (tests).
    #[must_use]
    pub fn with_limit(inner: R, max: usize) -> Self {
        Self {
            inner,
            buf: Vec::new(),
            scanned: 0,
            max,
        }
    }

    /// The next non-empty line without its terminator, or `None` at a clean end of stream. A
    /// final line without a newline is returned as a line.
    ///
    /// # Errors
    /// [`CodecError::TooLong`] when a line exceeds the limit, [`CodecError::Io`].
    pub async fn next_line(&mut self) -> Result<Option<Vec<u8>>, CodecError> {
        let _ = (
            &mut self.inner,
            &mut self.buf,
            self.scanned,
            self.max,
            CHUNK,
        );
        todo!("next_line")
    }
}

/// Serializes `value` as one line, terminator included.
#[must_use]
pub fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    let _ = serde_json::to_vec(value);
    todo!("encode")
}
