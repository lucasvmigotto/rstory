//! Capture pipeline: backend event, filters, store insert (batched), `history_changed`.
//!
//! The queue holds at most [`QUEUE_BOUND`] pending changes; on overflow the oldest are dropped
//! and counted in `dropped_burst`. Everything pending is committed as one `Store::apply`.

use crate::backend::{Backend, BackendEvent, ReadError};
use crate::domain::entry::Content;
use crate::domain::filter::{CopyFilter, FilterDecision, PreRead};
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
            let filter = CopyFilter::new(self.caps.max_entry_bytes);
            let limit = match filter.decide(&change.types, self.rec.is_paused()) {
                PreRead::Skip(FilterDecision::SkipPaused) => {
                    self.counters.skipped_paused += 1;
                    tracing::debug!(decision = "skipped_paused", "capture.decision");
                    continue;
                }
                PreRead::Skip(_) => {
                    self.counters.skipped_sensitive += 1;
                    tracing::debug!(decision = "skipped_sensitive", "capture.decision");
                    continue;
                }
                PreRead::Read { limit } => limit,
            };
            let Some(backend) = self.backend.as_mut() else {
                break;
            };
            let read = backend.read_text(limit).await;
            let decision = match read {
                Ok(bytes) => filter.after_read(bytes),
                Err(ReadError::TooLarge) => FilterDecision::SkipOversize,
                Err(ReadError::NoText) => FilterDecision::IgnoreNoText,
                Err(ReadError::Disconnected) => {
                    self.lose_backend();
                    break;
                }
                Err(e) => {
                    tracing::debug!(decision = "ignored", reason = %e, "capture.decision");
                    continue;
                }
            };
            match decision {
                FilterDecision::Record(bytes) => {
                    tracing::debug!(
                        decision = "record",
                        bytes = Redacted(&bytes).len(),
                        "capture.decision"
                    );
                    match Content::new(bytes, self.caps.max_entry_bytes) {
                        Ok(content) => batch.push(Mutation::RecordCopy(content)),
                        Err(_) => self.counters.skipped_oversize += 1,
                    }
                }
                FilterDecision::SkipOversize => {
                    self.counters.skipped_oversize += 1;
                    tracing::debug!(decision = "skipped_oversize", "capture.decision");
                }
                FilterDecision::IgnoreNoText => {
                    tracing::debug!(decision = "ignored_no_text", "capture.decision");
                }
                FilterDecision::IgnoreEmpty => {
                    tracing::debug!(decision = "ignored_empty", "capture.decision");
                }
                FilterDecision::SkipPaused | FilterDecision::SkipSensitive => {}
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
