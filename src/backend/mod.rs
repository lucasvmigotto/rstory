//! Clipboard backends: the `Backend` trait and the Wayland and X11 implementations.
//!
//! The trait exposes no display-server types (FR-001). Nothing here ever carries clipboard
//! content in an error, a log field or a `Debug` output (Constitution III): errors are plain
//! kinds, and content only travels as the `Ok` value of `read_text`.

use std::future::Future;
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::mpsc;

#[cfg(any(test, feature = "testing"))]
pub mod fake;
pub mod x11;

/// Which protocol a backend speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    /// Wayland data-control (spec 003).
    Wayland,
    /// X11 / XWayland (spec 002).
    X11,
    /// No usable backend.
    None,
}

impl BackendKind {
    /// Stable name for `status` and `doctor`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wayland => "wayland",
            Self::X11 => "x11",
            Self::None => "none",
        }
    }
}

/// How the backend reaches the clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The session's own protocol.
    Native,
    /// X11 client on a Wayland session without data-control (ADR-0005).
    XwaylandBridge,
    /// Not applicable (`BackendKind::None`).
    None,
}

impl Mode {
    /// Stable name for `status` and `doctor`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::XwaylandBridge => "xwayland_bridge",
            Self::None => "none",
        }
    }
}

/// What a backend reports to the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendEvent {
    /// A new owner appeared. `types` are the owner's offered names verbatim (X11 target atom
    /// names or Wayland MIME types). `self_owned` is true when the owner is this backend.
    SelectionChanged {
        types: Vec<String>,
        self_owned: bool,
    },
    /// The selection has no owner.
    SelectionCleared,
    /// The display connection is gone. Terminal for that instance.
    Disconnected,
}

/// Why `read_text` failed. No variant carries content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ReadError {
    /// More than `limit` bytes; the transfer was aborted.
    #[error("selection is larger than the limit")]
    TooLarge,
    /// The owner refused the conversion.
    #[error("the owner refused the conversion")]
    ConversionRefused,
    /// There is no owner, or it offers no text type.
    #[error("no text is offered")]
    NoText,
    /// The owner did not answer in time.
    #[error("the owner did not answer in time")]
    Timeout,
    /// The display connection is gone.
    #[error("disconnected from the display server")]
    Disconnected,
}

/// Why `serve` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ServeError {
    /// Ownership could not be acquired or was not confirmed.
    #[error("could not become the selection owner")]
    OwnershipFailed,
    /// The display connection is gone.
    #[error("disconnected from the display server")]
    Disconnected,
}

/// Receiving half of a backend's event stream.
#[derive(Debug)]
pub struct EventStream {
    rx: mpsc::UnboundedReceiver<BackendEvent>,
}

impl EventStream {
    /// A connected sender / stream pair.
    #[must_use]
    pub fn channel() -> (mpsc::UnboundedSender<BackendEvent>, Self) {
        let (tx, rx) = mpsc::unbounded_channel();
        (tx, Self { rx })
    }

    /// The next event, or `None` once the backend is gone and the queue is drained.
    pub async fn next(&mut self) -> Option<BackendEvent> {
        self.rx.recv().await
    }

    /// An event that is already queued, without waiting.
    pub fn try_next(&mut self) -> Option<BackendEvent> {
        self.rx.try_recv().ok()
    }
}

/// A clipboard backend. Used by the daemon only (fitness F1).
pub trait Backend {
    /// Ownership changes, once each, in order.
    fn events(&mut self) -> &mut EventStream;

    /// Read the current selection's best text type as UTF-8, aborting with `TooLarge` once
    /// `limit` bytes are exceeded. Never logs content.
    fn read_text(
        &mut self,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<u8>, ReadError>> + Send;

    /// Become the owner and serve `text`. Resolves `Ok` only after ownership is confirmed;
    /// serving continues until another owner appears.
    fn serve(&mut self, text: Arc<[u8]>) -> impl Future<Output = Result<(), ServeError>> + Send;

    /// Give up ownership if held.
    fn stop_serving(&mut self);
}
