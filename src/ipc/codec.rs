//! NDJSON framing: one JSON object per line, lines capped at [`MAX_LINE_BYTES`].

use std::io;

use serde::Serialize;
use tokio::io::{AsyncRead, AsyncReadExt};

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
        loop {
            if let Some(offset) = self.buf[self.scanned..].iter().position(|b| *b == b'\n') {
                let end = self.scanned + offset;
                let mut line: Vec<u8> = self.buf.drain(..=end).collect();
                self.scanned = 0;
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                if line.len() > self.max {
                    return Err(CodecError::TooLong);
                }
                if line.is_empty() {
                    continue;
                }
                return Ok(Some(line));
            }
            self.scanned = self.buf.len();
            if self.buf.len() > self.max {
                return Err(CodecError::TooLong);
            }
            let mut chunk = [0u8; CHUNK];
            let n = self.inner.read(&mut chunk).await?;
            if n == 0 {
                self.scanned = 0;
                let mut rest = std::mem::take(&mut self.buf);
                if rest.last() == Some(&b'\r') {
                    rest.pop();
                }
                return Ok((!rest.is_empty()).then_some(rest));
            }
            self.buf.extend_from_slice(&chunk[..n]);
        }
    }
}

/// Serializes `value` as one line, terminator included.
#[must_use]
pub fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    let mut out = serde_json::to_vec(value).unwrap_or_else(|_| b"null".to_vec());
    out.push(b'\n');
    out
}
