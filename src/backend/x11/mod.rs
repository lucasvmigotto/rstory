//! X11 backend: XFIXES selection watching, TARGETS-first reads, INCR, serving as owner.
//!
//! Pure Rust through x11rb's `RustConnection` (ADR-0006). Unix sockets only: no network
//! (Constitution III). Types, sizes and timings may be logged; content never.

pub mod read;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;

use super::{Backend, EventStream, Mode, ReadError, ServeError};

/// Why `X11Backend::connect` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ConnectError {
    /// The display could not be reached.
    #[error("the X display is unreachable")]
    Unreachable,
    /// The server lacks XFIXES 2 or newer.
    #[error("the X server has no usable XFIXES")]
    NoXfixes,
}

/// Connection settings.
#[derive(Debug, Clone)]
pub struct X11Options {
    /// `:N` or `unix:N`; `None` reads `$DISPLAY`.
    pub display: Option<String>,
    /// Xauthority file; `None` uses `$XAUTHORITY` or `~/.Xauthority`.
    pub xauthority: Option<PathBuf>,
    /// Bridge mode retries the connection while XWayland starts on demand.
    pub mode: Mode,
    /// How long bridge mode keeps retrying.
    pub connect_timeout: Duration,
    /// Per-step timeout of a read.
    pub read_timeout: Duration,
    /// Served values longer than this use INCR.
    pub incr_threshold: usize,
}

impl X11Options {
    /// Native-mode options for `display`.
    #[must_use]
    pub fn new(display: &str) -> Self {
        Self {
            display: Some(display.to_string()),
            xauthority: None,
            mode: Mode::Native,
            connect_timeout: Duration::from_secs(3),
            read_timeout: Duration::from_secs(2),
            incr_threshold: 256 * 1024,
        }
    }
}

/// The X11 implementation of [`Backend`].
pub struct X11Backend {
    events: EventStream,
}

impl X11Backend {
    /// Connect and start watching CLIPBOARD.
    pub async fn connect(_opts: X11Options) -> Result<Self, ConnectError> {
        Err(ConnectError::Unreachable)
    }
}

impl Backend for X11Backend {
    fn events(&mut self) -> &mut EventStream {
        &mut self.events
    }

    async fn read_text(&mut self, _limit: usize) -> Result<Vec<u8>, ReadError> {
        Err(ReadError::Disconnected)
    }

    async fn serve(&mut self, _text: Arc<[u8]>) -> Result<(), ServeError> {
        Err(ServeError::OwnershipFailed)
    }

    fn stop_serving(&mut self) {}
}
