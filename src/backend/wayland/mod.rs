//! Wayland backend: `ext_data_control_v1` with a `zwlr_data_control_v1` fallback.
//!
//! Pure Rust through `wayland-client`'s own backend (Constitution I). Types, sizes and timings
//! may be logged; content never (Constitution III).

pub mod protocols;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;

use super::{Backend, EventStream, ReadError, ServeError};
pub use protocols::Protocol;

/// Why `WaylandBackend::connect` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ConnectError {
    /// The compositor could not be reached.
    #[error("the Wayland display is unreachable")]
    Unreachable,
    /// The compositor offers neither data-control protocol, or the seat is missing.
    #[error("the compositor offers no data-control protocol")]
    NoDataControl,
}

/// Connection settings.
#[derive(Debug, Clone)]
pub struct WaylandOptions {
    /// Socket to connect to; `None` uses `$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY`.
    pub socket: Option<PathBuf>,
    /// Bind only this protocol; `None` prefers `ext`, then `zwlr`.
    pub protocol: Option<Protocol>,
    /// How long one read may take.
    pub read_timeout: Duration,
    /// How long one `send` write to a reader may take.
    pub write_timeout: Duration,
}

impl WaylandOptions {
    /// Defaults: environment socket, either protocol, 2 s reads.
    #[must_use]
    pub fn new() -> Self {
        Self {
            socket: None,
            protocol: None,
            read_timeout: Duration::from_secs(2),
            write_timeout: Duration::from_secs(10),
        }
    }

    /// Connect to this socket path.
    #[must_use]
    pub fn socket(mut self, path: impl Into<PathBuf>) -> Self {
        self.socket = Some(path.into());
        self
    }

    /// Bind only `protocol` (the other one is ignored even if offered).
    #[must_use]
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }
}

impl Default for WaylandOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// The Wayland implementation of [`Backend`].
pub struct WaylandBackend {
    events: EventStream,
}

impl WaylandBackend {
    /// Connect, bind data-control and start watching the selection.
    ///
    /// # Errors
    /// `Unreachable` when the compositor does not accept a connection, `NoDataControl` when it
    /// offers no data-control manager.
    pub async fn connect(_opts: WaylandOptions) -> Result<Self, ConnectError> {
        let _ = EventStream::channel();
        Err(ConnectError::Unreachable)
    }

    /// The protocol this connection speaks.
    #[must_use]
    pub fn protocol(&self) -> Protocol {
        Protocol::Zwlr
    }
}

impl Backend for WaylandBackend {
    fn events(&mut self) -> &mut EventStream {
        &mut self.events
    }

    async fn read_text(&mut self, _limit: usize) -> Result<Vec<u8>, ReadError> {
        Err(ReadError::Disconnected)
    }

    async fn serve(&mut self, _text: Arc<[u8]>) -> Result<(), ServeError> {
        Err(ServeError::Disconnected)
    }

    fn stop_serving(&mut self) {}
}
