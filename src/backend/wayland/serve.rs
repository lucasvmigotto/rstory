//! Serving text as the selection owner (FR-005).
//!
//! `serve` creates a data-control source offering the text types, sets it as the selection and
//! confirms with a `wl_display.sync` round trip. Every `send` runs in its own task over the
//! shared `Arc<[u8]>`, so a slow reader never blocks the connection loop, and a reader that
//! closes early (`EPIPE`) is ignored. Only sizes are logged.

use std::os::fd::OwnedFd;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::unix::pipe;
use tokio::sync::oneshot;
use wayland_client::QueueHandle;

use super::State;
use super::protocols::Source;
use crate::backend::{BackendEvent, ServeError};

/// The types offered while serving, in order (US2-AS1).
pub const SERVED_MIMES: [&str; 5] = [
    "text/plain;charset=utf-8",
    "text/plain",
    "UTF8_STRING",
    "STRING",
    "TEXT",
];

/// A `serve` waiting for the compositor to process the new source.
pub(super) struct Confirm {
    id: u64,
    reply: oneshot::Sender<Result<(), ServeError>>,
    /// The compositor already echoed our selection back as an event.
    echoed: bool,
}

/// What we currently serve.
#[derive(Default)]
pub struct ServeState {
    next_id: u64,
    active: Option<(u64, Source, Arc<[u8]>)>,
    confirm: Option<Confirm>,
}

impl ServeState {
    /// The offered type names, as owned strings.
    fn types() -> Vec<String> {
        SERVED_MIMES.iter().map(ToString::to_string).collect()
    }

    /// Whether `types` is exactly what our source offers (case-sensitive, any order).
    fn is_ours(types: &[String]) -> bool {
        types.len() == SERVED_MIMES.len()
            && SERVED_MIMES.iter().all(|m| types.iter().any(|t| t == m))
    }

    /// An incoming selection while we await confirmation may be our own source echoed back.
    /// Returns true when it is, and records that the event has been reported.
    pub(super) fn claims_echo(&mut self, types: &[String]) -> bool {
        match &mut self.confirm {
            Some(c) if !c.echoed && Self::is_ours(types) => {
                c.echoed = true;
                true
            }
            _ => false,
        }
    }

    /// Source `id` is still the one we serve.
    fn holds(&self, id: u64) -> bool {
        self.active.as_ref().is_some_and(|(a, _, _)| *a == id)
    }
}

impl State {
    pub(super) fn cmd_serve(
        &mut self,
        text: Arc<[u8]>,
        reply: oneshot::Sender<Result<(), ServeError>>,
        qh: &QueueHandle<State>,
    ) {
        if self.dead {
            let _ = reply.send(Err(ServeError::Disconnected));
            return;
        }
        let (Some(manager), Some(device)) = (self.manager.clone(), self.device.clone()) else {
            let _ = reply.send(Err(ServeError::OwnershipFailed));
            return;
        };
        // A newer serve supersedes one that is still awaiting confirmation.
        if let Some(old) = self.serve.confirm.take() {
            let _ = old.reply.send(Err(ServeError::OwnershipFailed));
        }
        self.serve.next_id += 1;
        let id = self.serve.next_id;
        let source = manager.create_data_source(id, qh);
        for mime in SERVED_MIMES {
            source.offer(mime);
        }
        device.set_selection(Some(&source));
        if let Some((_, old, _)) = self.serve.active.replace((id, source, text)) {
            old.destroy();
        }
        self.conn.display().sync(qh, id);
        self.serve.confirm = Some(Confirm {
            id,
            reply,
            echoed: false,
        });
    }

    /// The compositor processed everything up to our `sync`.
    pub(super) fn sync_done(&mut self, id: u64) {
        let Some(c) = self.serve.confirm.take_if(|c| c.id == id) else {
            return;
        };
        if self.serve.holds(id) {
            if !c.echoed {
                self.emit_self_owned();
            }
            let _ = c.reply.send(Ok(()));
        } else {
            let _ = c.reply.send(Err(ServeError::OwnershipFailed));
        }
    }

    fn emit_self_owned(&self) {
        let types = ServeState::types();
        tracing::debug!(types = %types.join(","), self_owned = true, "wl.selection_changed");
        self.emit(BackendEvent::SelectionChanged {
            types,
            self_owned: true,
        });
    }

    /// Our source lost the selection.
    pub(super) fn source_cancelled(&mut self, id: u64) {
        if !self.serve.holds(id) {
            return;
        }
        if let Some((_, source, _)) = self.serve.active.take() {
            source.destroy();
        }
        if let Some(c) = self.serve.confirm.take_if(|c| c.id == id) {
            let _ = c.reply.send(Err(ServeError::OwnershipFailed));
        }
        tracing::debug!("wl.serve_cancelled");
    }

    /// A reader wants `mime`: answer from a task.
    pub(super) fn source_send(&mut self, id: u64, mime: &str, fd: OwnedFd) {
        let Some((_, _, text)) = self.serve.active.as_ref().filter(|(a, _, _)| *a == id) else {
            return;
        };
        if !SERVED_MIMES.contains(&mime) {
            return;
        }
        let text = Arc::clone(text);
        let timeout = self.write_timeout;
        tokio::spawn(async move {
            let Ok(mut tx) = pipe::Sender::from_owned_fd(fd) else {
                return;
            };
            let write = async {
                tx.write_all(&text).await?;
                tx.shutdown().await
            };
            // EPIPE from a reader that left early, and a reader that stalls past the timeout,
            // both end quietly: neither is ours to fix.
            let ok = tokio::time::timeout(timeout, write)
                .await
                .is_ok_and(|r| r.is_ok());
            tracing::debug!(bytes = text.len(), ok, "wl.send");
        });
    }

    /// Give up ownership if we still hold it.
    pub(super) fn cmd_stop(&mut self) {
        let Some((_, source, _)) = self.serve.active.take() else {
            return;
        };
        if let Some(device) = &self.device {
            device.set_selection(None);
        }
        source.destroy();
        if let Some(c) = self.serve.confirm.take() {
            let _ = c.reply.send(Err(ServeError::OwnershipFailed));
        }
    }

    /// Fail a pending serve because the connection is gone.
    pub(super) fn fail_serve(&mut self) {
        if let Some(c) = self.serve.confirm.take() {
            let _ = c.reply.send(Err(ServeError::Disconnected));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_served_set_is_recognised_in_any_order() {
        let mut types: Vec<String> = SERVED_MIMES.iter().rev().map(ToString::to_string).collect();
        assert!(ServeState::is_ours(&types));
        types.pop();
        assert!(!ServeState::is_ours(&types));
        types.push("image/png".into());
        assert!(!ServeState::is_ours(&types));
    }

    #[test]
    fn an_echo_is_claimed_once_and_only_while_confirming() {
        let (tx, _rx) = oneshot::channel();
        let mut s = ServeState {
            confirm: Some(Confirm {
                id: 1,
                reply: tx,
                echoed: false,
            }),
            ..ServeState::default()
        };
        let ours = ServeState::types();
        assert!(s.claims_echo(&ours));
        assert!(!s.claims_echo(&ours));
        let mut idle = ServeState::default();
        assert!(!idle.claims_echo(&ours));
    }
}
