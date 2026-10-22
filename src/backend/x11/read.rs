//! Target choice, text decoding and the read state machine (FR-003, FR-004).
//!
//! A read is: TARGETS (already known from the change notification) -> best text target ->
//! `ConvertSelection` -> one property, or INCR (see `incr.rs`). The limit is enforced while the
//! data arrives, never after. Only sizes and timings are logged.

use std::time::Instant;

use tokio::sync::oneshot;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, SelectionNotifyEvent};

use super::{Engine, TargetsState};
use crate::backend::{BackendEvent, ReadError};

/// Text targets in order of preference.
pub const PREFERENCE: [&str; 4] = ["UTF8_STRING", "text/plain;charset=utf-8", "STRING", "TEXT"];

/// The best text target among the owner's offered `types`.
#[must_use]
pub fn pick_target(types: &[String]) -> Option<&'static str> {
    PREFERENCE
        .into_iter()
        .find(|p| types.iter().any(|t| t == p))
}

/// ISO 8859-1 bytes as UTF-8 text.
#[must_use]
pub fn latin1_to_utf8(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

/// UTF-8 bytes with invalid sequences replaced by U+FFFD, and the number of replacements.
#[must_use]
pub fn utf8_lossy(bytes: Vec<u8>) -> (Vec<u8>, usize) {
    match String::from_utf8(bytes) {
        Ok(s) => (s.into_bytes(), 0),
        Err(e) => {
            let bytes = e.into_bytes();
            let replaced = bytes
                .utf8_chunks()
                .filter(|c| !c.invalid().is_empty())
                .count();
            (
                String::from_utf8_lossy(&bytes).into_owned().into_bytes(),
                replaced,
            )
        }
    }
}

pub(crate) enum Stage {
    /// Waiting for the TARGETS conversion of the change notification.
    WaitTargets,
    /// `ConvertSelection` sent, waiting for `SelectionNotify`.
    Convert { name: String, deadline: Instant },
    /// INCR transfer in progress.
    Incr {
        name: String,
        buf: Vec<u8>,
        ty: u32,
        deadline: Instant,
    },
}

pub(crate) struct ReadOp {
    pub limit: usize,
    pub reply: oneshot::Sender<Result<Vec<u8>, ReadError>>,
    pub stage: Stage,
    pub started: Instant,
}

impl Engine {
    pub(super) fn cmd_read(
        &mut self,
        limit: usize,
        reply: oneshot::Sender<Result<Vec<u8>, ReadError>>,
    ) {
        if let Some(old) = self.read.take() {
            let _ = old.reply.send(Err(ReadError::ConversionRefused));
        }
        if self.sel.owner == self.win && self.serve.active() {
            let text = self.serve.text();
            let _ = reply.send(if text.len() > limit {
                Err(ReadError::TooLarge)
            } else {
                Ok(text.to_vec())
            });
            return;
        }
        let op = ReadOp {
            limit,
            reply,
            stage: Stage::WaitTargets,
            started: Instant::now(),
        };
        match &self.sel.targets {
            TargetsState::None => {
                let _ = op.reply.send(Err(ReadError::NoText));
            }
            TargetsState::Failed => {
                let _ = op.reply.send(Err(ReadError::ConversionRefused));
            }
            TargetsState::Pending { .. } => self.read = Some(op),
            TargetsState::Known(_) => self.start_read(op),
        }
    }

    fn start_read(&mut self, mut op: ReadOp) {
        let TargetsState::Known(types) = &self.sel.targets else {
            return;
        };
        let Some(name) = pick_target(types) else {
            let _ = op.reply.send(Err(ReadError::NoText));
            return;
        };
        let Some(target) = self.intern(name) else {
            return;
        };
        let failed = self
            .conn
            .convert_selection(
                self.win,
                self.atoms.clipboard,
                target,
                self.atoms.prop_data,
                self.sel.time,
            )
            .is_err();
        if failed {
            self.read = Some(op);
            self.disconnect();
            return;
        }
        op.stage = Stage::Convert {
            name: name.to_string(),
            deadline: Instant::now() + self.read_timeout,
        };
        self.read = Some(op);
    }

    /// The TARGETS conversion for the current owner finished (or timed out).
    pub(super) fn finish_targets(&mut self, answered: bool) {
        if !matches!(self.sel.targets, TargetsState::Pending { .. }) {
            return;
        }
        let mut types = Vec::new();
        let mut ok = false;
        if answered {
            let r = self
                .conn
                .get_property(
                    true,
                    self.win,
                    self.atoms.prop_targets,
                    AtomEnum::ANY,
                    0,
                    1024,
                )
                .map_err(ReplyError::from)
                .and_then(|c| c.reply());
            match r {
                Ok(r) if r.format == 32 => {
                    ok = true;
                    let atoms: Vec<u32> = r
                        .value
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|c| u32::from_ne_bytes(*c))
                        .take(256)
                        .collect();
                    for a in atoms {
                        if let Some(n) = self.atom_name(a) {
                            types.push(n);
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    if self.fatal(&e) {
                        return;
                    }
                }
            }
        }
        if ok {
            self.sel.targets = TargetsState::Known(types.clone());
        } else {
            self.sel.targets = TargetsState::Failed;
        }
        tracing::debug!(
            types = types.len(),
            self_owned = false,
            "x11.selection_changed"
        );
        self.emit(BackendEvent::SelectionChanged {
            types,
            self_owned: false,
        });
        if let Some(op) = self.read.take() {
            if matches!(op.stage, Stage::WaitTargets) {
                if ok {
                    self.start_read(op);
                } else {
                    let _ = op.reply.send(Err(ReadError::ConversionRefused));
                }
            } else {
                self.read = Some(op);
            }
        }
    }

    pub(super) fn on_selection_notify(&mut self, e: &SelectionNotifyEvent) {
        let a = &self.atoms;
        if e.target == a.targets && matches!(self.sel.targets, TargetsState::Pending { .. }) {
            self.finish_targets(e.property != 0);
            return;
        }
        let converting = matches!(
            self.read.as_ref().map(|o| &o.stage),
            Some(Stage::Convert { .. })
        );
        if !converting {
            if e.property != 0 {
                let _ = self.conn.delete_property(self.win, e.property);
            }
            return;
        }
        if e.property == 0 {
            self.complete(Err(ReadError::ConversionRefused));
            return;
        }
        let Some(limit) = self.read.as_ref().map(|o| o.limit) else {
            return;
        };
        let long_len = u32::try_from(limit / 4 + 2).unwrap_or(u32::MAX / 4);
        let r = self
            .conn
            .get_property(
                true,
                self.win,
                self.atoms.prop_data,
                AtomEnum::ANY,
                0,
                long_len,
            )
            .map_err(ReplyError::from)
            .and_then(|c| c.reply());
        let r = match r {
            Ok(r) => r,
            Err(e) => {
                if !self.fatal(&e) {
                    self.complete(Err(ReadError::ConversionRefused));
                }
                return;
            }
        };
        if r.type_ == self.atoms.incr {
            let hint = r
                .value
                .as_chunks::<4>()
                .0
                .first()
                .map_or(0, |c| u32::from_ne_bytes(*c) as usize);
            if hint > limit {
                self.complete(Err(ReadError::TooLarge));
                return;
            }
            self.begin_incr();
            return;
        }
        if r.bytes_after > 0 || r.value.len() > limit {
            let _ = self.conn.delete_property(self.win, self.atoms.prop_data);
            self.complete(Err(ReadError::TooLarge));
            return;
        }
        self.finish_data(r.type_, r.value);
    }

    /// All bytes are in: decode per target / returned type and answer the caller.
    pub(super) fn finish_data(&mut self, ty: u32, bytes: Vec<u8>) {
        let Some(op) = self.read.as_ref() else { return };
        let (name, limit) = match &op.stage {
            Stage::Convert { name, .. } | Stage::Incr { name, .. } => (name.clone(), op.limit),
            Stage::WaitTargets => return,
        };
        let received = bytes.len();
        let string_atom: u32 = AtomEnum::STRING.into();
        let latin1 = match name.as_str() {
            "STRING" => Some(true),
            "UTF8_STRING" | "text/plain;charset=utf-8" => Some(false),
            _ if ty == self.atoms.utf8 => Some(false),
            _ if ty == string_atom => Some(true),
            _ => None,
        };
        let out = match latin1 {
            Some(true) => latin1_to_utf8(&bytes).into_bytes(),
            Some(false) => {
                let (out, replaced) = utf8_lossy(bytes);
                if replaced > 0 {
                    tracing::debug!(replaced, "x11.read invalid utf-8 replaced");
                }
                out
            }
            None => {
                self.complete(Err(ReadError::ConversionRefused));
                return;
            }
        };
        if out.len() > limit {
            self.complete(Err(ReadError::TooLarge));
            return;
        }
        let ms = self
            .read
            .as_ref()
            .map_or(0, |o| o.started.elapsed().as_millis());
        tracing::debug!(target_name = %name, bytes = received, ms = %ms, "x11.read");
        self.complete(Ok(out));
    }

    pub(super) fn complete(&mut self, r: Result<Vec<u8>, ReadError>) {
        if let Some(op) = self.read.take() {
            if let Err(e) = &r {
                tracing::debug!(error = %e, "x11.read failed");
            }
            let _ = op.reply.send(r);
        }
    }

    pub(super) fn read_deadline(&self) -> Option<Instant> {
        match self.read.as_ref().map(|o| &o.stage) {
            Some(Stage::Convert { deadline, .. } | Stage::Incr { deadline, .. }) => Some(*deadline),
            _ => None,
        }
    }

    pub(super) fn read_deadline_passed(&mut self, now: Instant) {
        if self.read_deadline().is_some_and(|d| now >= d) {
            // The owner went quiet: stop waiting and drop whatever it still writes.
            let _ = self.conn.delete_property(self.win, self.atoms.prop_data);
            self.complete(Err(ReadError::Timeout));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| (*n).to_string()).collect()
    }

    #[test]
    fn prefers_utf8_then_plain_then_string_then_text() {
        assert_eq!(
            pick_target(&t(&["STRING", "UTF8_STRING", "TEXT"])),
            Some("UTF8_STRING")
        );
        assert_eq!(
            pick_target(&t(&["STRING", "text/plain;charset=utf-8"])),
            Some("text/plain;charset=utf-8")
        );
        assert_eq!(pick_target(&t(&["TEXT", "STRING"])), Some("STRING"));
        assert_eq!(pick_target(&t(&["TEXT"])), Some("TEXT"));
    }

    #[test]
    fn no_text_target_means_none() {
        assert_eq!(pick_target(&t(&["image/png", "TARGETS"])), None);
        assert_eq!(pick_target(&[]), None);
    }

    #[test]
    fn latin1_maps_each_byte_to_its_code_point() {
        assert_eq!(latin1_to_utf8(b"caf\xe9 \xff"), "caf\u{e9} \u{ff}");
        assert_eq!(latin1_to_utf8(b""), "");
    }

    #[test]
    fn invalid_utf8_is_replaced_and_counted() {
        assert_eq!(
            utf8_lossy(b"ab\xffcd\xc3".to_vec()),
            ("ab\u{fffd}cd\u{fffd}".as_bytes().to_vec(), 2)
        );
        assert_eq!(
            utf8_lossy("ok \u{e9}".as_bytes().to_vec()),
            ("ok \u{e9}".as_bytes().to_vec(), 0)
        );
    }
}
