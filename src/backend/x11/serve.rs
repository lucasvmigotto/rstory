//! Owner / serve state machine (stub until the restore story lands).

use std::sync::Arc;
use std::time::Instant;

use x11rb::protocol::xproto::SelectionRequestEvent;

use super::Engine;
use crate::backend::ServeError;

/// What we currently serve.
#[derive(Default)]
pub struct ServeState {
    text: Option<Arc<[u8]>>,
}

impl ServeState {
    pub fn active(&self) -> bool {
        self.text.is_some()
    }
    pub fn clear(&mut self) {
        self.text = None;
    }
    pub fn set_acquired(&mut self, _time: u32) {}
    pub fn type_names(&self) -> Vec<String> {
        Vec::new()
    }
    pub fn text(&self) -> Arc<[u8]> {
        self.text.clone().unwrap_or_else(|| Arc::from(&b""[..]))
    }
    pub fn next_deadline(&self) -> Option<Instant> {
        None
    }
}

impl Engine {
    pub(super) fn cmd_serve(&mut self, _text: Arc<[u8]>) -> Result<(), ServeError> {
        Err(ServeError::OwnershipFailed)
    }
    pub(super) fn cmd_stop(&mut self) {}
    pub(super) fn serve_request(&mut self, _e: &SelectionRequestEvent) {}
    pub(super) fn serve_property_deleted(&mut self, _window: u32, _atom: u32) {}
    pub(super) fn serve_deadline_passed(&mut self, _now: Instant) {}
}
