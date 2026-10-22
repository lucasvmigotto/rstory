//! Owner / serve state machine (FR-005, FR-006).
//!
//! `serve` becomes the CLIPBOARD owner on our hidden window and answers `SelectionRequest`s for
//! `TARGETS`, `TIMESTAMP`, `UTF8_STRING`, `STRING` (Latin-1, lossy), `TEXT` and
//! `text/plain;charset=utf-8`. Everything else, `MULTIPLE` included, is refused. Values above
//! the INCR threshold go out in chunks, one transfer per (requestor, property), so several
//! requestors can be served at once. Only sizes are logged.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use x11rb::CURRENT_TIME;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::{
    AtomEnum, ChangeWindowAttributesAux, ConnectionExt as _, EventMask, PropMode,
    SELECTION_NOTIFY_EVENT, SelectionNotifyEvent, SelectionRequestEvent,
};
use x11rb::wrapper::ConnectionExt as _;

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

/// A requestor that stops deleting the property for this long loses its transfer.
const TRANSFER_IDLE: Duration = Duration::from_secs(5);

/// UTF-8 text as ISO 8859-1; characters above U+00FF become `?`.
#[must_use]
pub fn latin1_lossy(text: &[u8]) -> Vec<u8> {
    String::from_utf8_lossy(text)
        .chars()
        .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
        .collect()
}

struct Transfer {
    data: Vec<u8>,
    pos: usize,
    ty: u32,
    /// The empty terminating chunk has been written.
    finished: bool,
    deadline: Instant,
}

/// What we currently serve.
#[derive(Default)]
pub struct ServeState {
    text: Option<Arc<[u8]>>,
    acquired: u32,
    transfers: HashMap<(u32, u32), Transfer>,
}

impl ServeState {
    pub fn active(&self) -> bool {
        self.text.is_some()
    }

    pub fn clear(&mut self) {
        self.text = None;
        self.acquired = 0;
        self.transfers.clear();
    }

    pub fn set_acquired(&mut self, time: u32) {
        self.acquired = time;
    }

    pub fn type_names(&self) -> Vec<String> {
        SERVED_TARGETS.iter().map(|t| (*t).to_string()).collect()
    }

    pub fn text(&self) -> Arc<[u8]> {
        self.text.clone().unwrap_or_else(|| Arc::from(&b""[..]))
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        self.transfers.values().map(|t| t.deadline).min()
    }
}

/// `a` is earlier than `b` on the wrapping 32-bit X server clock.
fn before(a: u32, b: u32) -> bool {
    a != b && b.wrapping_sub(a) < 0x8000_0000
}

impl Engine {
    pub(super) fn cmd_serve(&mut self, text: Arc<[u8]>) -> Result<(), ServeError> {
        self.serve.clear();
        self.serve.text = Some(text);
        let failed = self
            .conn
            .set_selection_owner(self.win, self.atoms.clipboard, CURRENT_TIME)
            .is_err();
        if failed {
            self.disconnect();
            return Err(ServeError::Disconnected);
        }
        let owner = self
            .conn
            .get_selection_owner(self.atoms.clipboard)
            .map_err(ReplyError::from)
            .and_then(|c| c.reply());
        match owner {
            Ok(o) if o.owner == self.win => Ok(()),
            Ok(_) => {
                self.serve.clear();
                Err(ServeError::OwnershipFailed)
            }
            Err(e) => {
                self.fatal(&e);
                self.serve.clear();
                Err(if self.dead {
                    ServeError::Disconnected
                } else {
                    ServeError::OwnershipFailed
                })
            }
        }
    }

    pub(super) fn cmd_stop(&mut self) {
        if !self.serve.active() {
            return;
        }
        self.serve.clear();
        let failed = self
            .conn
            .set_selection_owner(0u32, self.atoms.clipboard, CURRENT_TIME)
            .is_err();
        if failed {
            self.disconnect();
        }
    }

    pub(super) fn serve_request(&mut self, e: &SelectionRequestEvent) {
        let prop = if e.property == 0 {
            e.target
        } else {
            e.property
        };
        let ok = self.answer(e, prop);
        let reply = SelectionNotifyEvent {
            response_type: SELECTION_NOTIFY_EVENT,
            sequence: 0,
            time: e.time,
            requestor: e.requestor,
            selection: e.selection,
            target: e.target,
            property: if ok { prop } else { 0 },
        };
        let sent = self
            .conn
            .send_event(false, e.requestor, EventMask::NO_EVENT, reply)
            .map(x11rb::cookie::VoidCookie::ignore_error);
        if sent.is_err() {
            self.disconnect();
        }
    }

    fn answer(&mut self, e: &SelectionRequestEvent, prop: u32) -> bool {
        if e.selection != self.atoms.clipboard || !self.serve.active() {
            return false;
        }
        if e.time != CURRENT_TIME && self.serve.acquired != 0 && before(e.time, self.serve.acquired)
        {
            return false;
        }
        let a = &self.atoms;
        let (targets, timestamp, utf8, plain, text) =
            (a.targets, a.timestamp, a.utf8, a.plain_utf8, a.text);
        let string: u32 = AtomEnum::STRING.into();
        let value = self.serve.text();
        if e.target == targets {
            let mut list = Vec::with_capacity(SERVED_TARGETS.len());
            for name in SERVED_TARGETS {
                match self.intern(name) {
                    Some(atom) => list.push(atom),
                    None => return false,
                }
            }
            let r = self
                .conn
                .change_property32(PropMode::REPLACE, e.requestor, prop, AtomEnum::ATOM, &list)
                .map(x11rb::cookie::VoidCookie::ignore_error);
            return self.sent(r.is_ok());
        }
        if e.target == timestamp {
            let r = self
                .conn
                .change_property32(
                    PropMode::REPLACE,
                    e.requestor,
                    prop,
                    AtomEnum::INTEGER,
                    &[self.serve.acquired],
                )
                .map(x11rb::cookie::VoidCookie::ignore_error);
            return self.sent(r.is_ok());
        }
        let (data, ty) = if e.target == utf8 || e.target == plain {
            (value.to_vec(), e.target)
        } else if e.target == text {
            (value.to_vec(), utf8)
        } else if e.target == string {
            (latin1_lossy(&value), string)
        } else {
            return false;
        };
        self.send_value(e.requestor, prop, ty, data)
    }

    /// Record a failed write as a lost connection; otherwise pass the result through.
    fn sent(&mut self, ok: bool) -> bool {
        if !ok {
            self.disconnect();
        }
        ok
    }

    fn send_value(&mut self, requestor: u32, prop: u32, ty: u32, data: Vec<u8>) -> bool {
        if data.len() <= self.incr_threshold {
            let r = self
                .conn
                .change_property8(PropMode::REPLACE, requestor, prop, ty, &data)
                .map(x11rb::cookie::VoidCookie::ignore_error);
            return self.sent(r.is_ok());
        }
        tracing::debug!(bytes = data.len(), "x11.serve incr start");
        let len = u32::try_from(data.len()).unwrap_or(u32::MAX);
        let r = self
            .conn
            .change_window_attributes(
                requestor,
                &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
            )
            .map(x11rb::cookie::VoidCookie::ignore_error)
            .and_then(|()| {
                self.conn
                    .change_property32(PropMode::REPLACE, requestor, prop, self.atoms.incr, &[len])
                    .map(x11rb::cookie::VoidCookie::ignore_error)
            });
        if !self.sent(r.is_ok()) {
            return false;
        }
        self.serve.transfers.insert(
            (requestor, prop),
            Transfer {
                data,
                pos: 0,
                ty,
                finished: false,
                deadline: Instant::now() + TRANSFER_IDLE,
            },
        );
        true
    }

    /// The requestor deleted a property: send the next chunk of its INCR transfer, if any.
    pub(super) fn serve_property_deleted(&mut self, window: u32, atom: u32) {
        let chunk = self.incr_threshold;
        let Some(t) = self.serve.transfers.get_mut(&(window, atom)) else {
            return;
        };
        if t.finished {
            self.serve.transfers.remove(&(window, atom));
            self.release_window(window);
            return;
        }
        let end = (t.pos + chunk).min(t.data.len());
        let r = self
            .conn
            .change_property8(PropMode::REPLACE, window, atom, t.ty, &t.data[t.pos..end])
            .map(x11rb::cookie::VoidCookie::ignore_error);
        t.finished = t.pos == end;
        t.pos = end;
        t.deadline = Instant::now() + TRANSFER_IDLE;
        self.sent(r.is_ok());
    }

    pub(super) fn serve_deadline_passed(&mut self, now: Instant) {
        let expired: Vec<_> = self
            .serve
            .transfers
            .iter()
            .filter(|(_, t)| now >= t.deadline)
            .map(|(k, _)| *k)
            .collect();
        for key in expired {
            self.serve.transfers.remove(&key);
            self.release_window(key.0);
        }
    }

    /// Stop listening to a requestor's property changes once no transfer needs it.
    fn release_window(&mut self, window: u32) {
        if self.serve.transfers.keys().any(|(w, _)| *w == window) {
            return;
        }
        let _ = self
            .conn
            .change_window_attributes(
                window,
                &ChangeWindowAttributesAux::new().event_mask(EventMask::NO_EVENT),
            )
            .map(x11rb::cookie::VoidCookie::ignore_error);
    }
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

    #[test]
    fn server_time_comparison_wraps() {
        assert!(before(1, 2));
        assert!(!before(2, 1));
        assert!(before(u32::MAX, 1));
        assert!(!before(5, 5));
    }
}
