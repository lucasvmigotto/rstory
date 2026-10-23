//! Restore: resolve, serve, acknowledge only after the backend confirmed ownership, then move
//! the entry to the top (BR2). The resulting `self_owned` backend event creates nothing.

use serde_json::{Value, json};

use crate::backend::{Backend, ServeError};
use crate::domain::ids::EntryRef;
use crate::ipc::error::{Correlation, ErrorCode, ProtoError};
use crate::store::Mutation;
use crate::store::fs::Fs;

use super::{Daemon, error_kind};

impl<B: Backend, F: Fs> Daemon<B, F> {
    pub(crate) async fn restore(&mut self, r: &EntryRef) -> Result<Value, ProtoError> {
        if self.backend.is_none() {
            return Err(ProtoError::new(
                ErrorCode::Inactive,
                "no capture backend; restore is unavailable",
            ));
        }
        let id = self.resolve(r)?.id;
        let bytes = self.store.content(&id).map_err(|e| {
            let c = Correlation::next();
            tracing::error!(kind = %error_kind(&e), correlation = %c, "restore: content unreadable");
            ProtoError::internal(c)
        })?;
        let Some(backend) = self.backend.as_mut() else {
            return Err(ProtoError::new(ErrorCode::Inactive, "no capture backend"));
        };
        if let Err(e) = backend.serve(bytes.into()).await {
            if e == ServeError::Disconnected {
                self.lose_backend();
            }
            return Err(ProtoError::new(
                ErrorCode::BackendError,
                "the backend could not take ownership of the clipboard",
            ));
        }
        self.commit(vec![Mutation::Touch(id)])?;
        self.publish_history();
        Ok(json!({}))
    }
}
