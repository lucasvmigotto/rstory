//! Capture pipeline: backend event, filters, store insert (batched), `history_changed`.
//!
//! The queue holds at most [`QUEUE_BOUND`] pending changes; on overflow the oldest are dropped
//! and counted in `dropped_burst`. Everything pending is committed as one `Store::apply`.

use crate::backend::{Backend, BackendEvent, ReadError};
use crate::domain::entry::Content;
use crate::domain::redact::Redacted;
use crate::store::fs::Fs;
use crate::store::{Mutation, Outcome};

use super::{Daemon, QUEUE_BOUND, error_kind};

/// One pending clipboard change (no content: that is read when the change is processed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    /// The owner's offered type names.
    pub types: Vec<String>,
    /// Whether the owner is this daemon (a restore).
    pub self_owned: bool,
}

/// What the filters decide about a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Read and record it.
    Record,
    /// Sensitive hint: skip without reading (spec 005).
    SkipSensitive,
}

/// The filter hook: spec 005 plugs the sensitive-content filter in here. The pause gate runs
/// before it and is not part of the hook.
pub type Filter = Box<dyn FnMut(&Pending) -> Decision + Send>;

/// The default hook: everything passes.
#[must_use]
pub fn pass_through() -> Filter {
    Box::new(|_| Decision::Record)
}

impl<B: Backend, F: Fs> Daemon<B, F> {
    /// Handles one backend event, then drains whatever else is already waiting.
    pub(crate) async fn on_backend_event(&mut self, first: Option<BackendEvent>) {
        let mut next = first;
        loop {
            match next {
                None | Some(BackendEvent::Disconnected) => {
                    self.lose_backend();
                    return;
                }
                Some(BackendEvent::SelectionCleared) => {}
                Some(BackendEvent::SelectionChanged { types, self_owned }) => {
                    if self.queue.len() >= QUEUE_BOUND {
                        self.queue.pop_front();
                        self.counters.dropped_burst += 1;
                    }
                    self.queue.push_back(Pending { types, self_owned });
                }
            }
            next = match self.backend.as_mut().and_then(|b| b.events().try_next()) {
                Some(ev) => Some(ev),
                None => break,
            };
        }
        self.process_queue().await;
    }

    async fn process_queue(&mut self) {
        let mut batch: Vec<Mutation> = Vec::new();
        while let Some(change) = self.queue.pop_front() {
            if change.self_owned {
                // Our own restore: it was moved to the top when ownership was confirmed.
                continue;
            }
            if self.rec.is_paused() {
                self.counters.skipped_paused += 1;
                tracing::debug!(decision = "skipped_paused", "capture.decision");
                continue;
            }
            if (self.filter)(&change) == Decision::SkipSensitive {
                self.counters.skipped_sensitive += 1;
                tracing::debug!(decision = "skipped_sensitive", "capture.decision");
                continue;
            }
            let limit = usize::try_from(self.caps.max_entry_bytes).unwrap_or(usize::MAX);
            let Some(backend) = self.backend.as_mut() else {
                break;
            };
            match backend.read_text(limit).await {
                Ok(bytes) => {
                    let hidden = Redacted(&bytes);
                    match Content::new(bytes.clone(), self.caps.max_entry_bytes) {
                        Ok(content) => {
                            tracing::debug!(
                                decision = "record",
                                bytes = hidden.len(),
                                "capture.decision"
                            );
                            batch.push(Mutation::RecordCopy(content));
                        }
                        Err(_) => self.counters.skipped_oversize += 1,
                    }
                }
                Err(ReadError::TooLarge) => {
                    self.counters.skipped_oversize += 1;
                    tracing::debug!(decision = "skipped_oversize", "capture.decision");
                }
                Err(ReadError::Disconnected) => {
                    self.lose_backend();
                    break;
                }
                Err(e) => tracing::debug!(decision = "ignored", reason = %e, "capture.decision"),
            }
        }
        if batch.is_empty() {
            return;
        }
        match self.store.apply(batch, self.clock.now()) {
            Ok(outcomes) => {
                if outcomes.iter().any(|o| matches!(o, Outcome::Recorded(_))) {
                    self.publish_history();
                }
            }
            Err(e) => tracing::error!(kind = %error_kind(&e), "capture commit failed"),
        }
    }
}
