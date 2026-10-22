//! Owner / serve state machine (stub until the restore story lands).

use std::sync::Arc;
use std::time::Instant;

use x11rb::protocol::xproto::SelectionRequestEvent;

use super::Engine;
use crate::backend::ServeError;

/// Target names answered while serving, in the order TARGETS lists them (FR-006).
pub const SERVED_TARGETS: [&str; 6] = [
    "TARGETS",
    "TIMESTAMP",
    "UTF8_STRING",
    "STRING",
    "TEXT",
    "text/plain;charset=utf-8",
];

/// UTF-8 text as ISO 8859-1; characters above U+00FF become `?`.
#[must_use]
pub fn latin1_lossy(_text: &[u8]) -> Vec<u8> {
    Vec::new()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin1_keeps_latin1_and_replaces_the_rest() {
        assert_eq!(latin1_lossy("caf\u{e9} \u{ff}".as_bytes()), b"caf\xe9 \xff");
        assert_eq!(latin1_lossy("a\u{20ac}b\u{1f600}".as_bytes()), b"a?b?");
        assert_eq!(latin1_lossy(b""), b"");
    }

    #[test]
    fn invalid_utf8_input_does_not_panic() {
        assert_eq!(latin1_lossy(b"a\xffb"), b"a?b");
    }
}
