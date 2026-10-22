//! INCR receive (ICCCM 2.7.2): the owner announces a large value, we delete the property to
//! start it, then read chunk after chunk (each `PropertyNotify(NewValue)`) until an empty one.
//! The caller's limit is checked on every chunk.

use std::time::Instant;

use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _};

use super::Engine;
use super::read::Stage;
use crate::backend::ReadError;

impl Engine {
    /// The INCR announcement was read (which deleted the property and started the transfer).
    pub(super) fn begin_incr(&mut self) {
        let timeout = self.read_timeout;
        if let Some(op) = self.read.as_mut()
            && let Stage::Convert { name, .. } = &op.stage
        {
            op.stage = Stage::Incr {
                name: name.clone(),
                buf: Vec::new(),
                ty: 0,
                deadline: Instant::now() + timeout,
            };
        }
    }

    /// A chunk landed in our data property.
    pub(super) fn incr_new_value(&mut self) {
        let Some(op) = self.read.as_ref() else { return };
        let Stage::Incr { buf, .. } = &op.stage else {
            return;
        };
        let room = op.limit.saturating_sub(buf.len());
        let long_len = u32::try_from(room / 4 + 2).unwrap_or(u32::MAX / 4);
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
        let timeout = self.read_timeout;
        let Some(op) = self.read.as_mut() else { return };
        let limit = op.limit;
        let Stage::Incr {
            buf, ty, deadline, ..
        } = &mut op.stage
        else {
            return;
        };
        if r.bytes_after > 0 || buf.len() + r.value.len() > limit {
            let _ = self.conn.delete_property(self.win, self.atoms.prop_data);
            self.complete(Err(ReadError::TooLarge));
            return;
        }
        if r.value.is_empty() {
            let (ty, bytes) = (*ty, std::mem::take(buf));
            self.finish_data(ty, bytes);
            return;
        }
        *ty = r.type_;
        buf.extend_from_slice(&r.value);
        *deadline = Instant::now() + timeout;
    }
}
