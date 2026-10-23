//! Protocol handlers: list, show, pin, unpin, rm, clear, pause, resume, status, open_picker.

use std::time::Duration;

use serde_json::{Value, json};

use crate::backend::Backend;
use crate::domain::entry::Entry;
use crate::domain::history::{HistoryError, ResolveError};
use crate::domain::ids::{EntryId, EntryRef, ShortId};
use crate::domain::pause::PauseError;
use crate::ipc::error::{Correlation, ErrorCode, ProtoError};
use crate::ipc::protocol::{Command, EntrySummary, Status, VERSION};
use crate::launch::{self, LaunchPlan};
use crate::store::fs::Fs;
use crate::store::{Mutation, Outcome, StoreError};

use super::{Daemon, error_kind};

fn json_value<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

impl<B: Backend, F: Fs> Daemon<B, F> {
    pub(crate) async fn handle(&mut self, command: Command) -> Result<Value, ProtoError> {
        let started = std::time::Instant::now();
        let op = command.op();
        let result = match command {
            Command::List { limit } => Ok(self.list(limit)),
            Command::Show(r) => self.show(&r),
            Command::Restore(r) => self.restore(&r).await,
            Command::Pin(r) => self.pin(&r, true),
            Command::Unpin(r) => self.pin(&r, false),
            Command::Rm(r) => self.rm(&r),
            Command::Clear { all } => self.clear(all),
            Command::Pause { duration } => self.pause(duration),
            Command::Resume => self.resume(),
            Command::Status => Ok(json_value(&self.status())),
            Command::OpenPicker => self.open_picker(),
            Command::Subscribe { .. } => Ok(json!({})),
        };
        tracing::debug!(
            op,
            ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            ok = result.is_ok(),
            "request"
        );
        result
    }

    /// Runs a batch, mapping store failures to protocol errors (logged without content).
    pub(crate) fn commit(&mut self, batch: Vec<Mutation>) -> Result<Vec<Outcome>, ProtoError> {
        self.store
            .apply(batch, self.clock.now())
            .map_err(|e| match e {
                StoreError::History(HistoryError::NotFound) => {
                    ProtoError::new(ErrorCode::NotFound, "no such entry")
                }
                StoreError::History(HistoryError::PinLimit) => ProtoError::new(
                    ErrorCode::PinLimit,
                    format!("pin limit reached ({})", self.caps.max_pinned),
                ),
                other => {
                    let c = Correlation::next();
                    tracing::error!(kind = %error_kind(&other), correlation = %c, "store error");
                    ProtoError::internal(c)
                }
            })
    }

    pub(crate) fn resolve(&self, r: &EntryRef) -> Result<&Entry, ProtoError> {
        self.store.history().resolve(r).map_err(|e| match e {
            ResolveError::NotFound => ProtoError::new(
                ErrorCode::NotFound,
                match r {
                    EntryRef::Index(n) => format!("no entry at index {n}"),
                    EntryRef::ShortId(s) => format!("no entry {s}"),
                },
            ),
            ResolveError::Ambiguous(ids) => {
                let ShortIds(all) = ShortIds::of(self);
                let names: Vec<String> = ids
                    .iter()
                    .filter_map(|id| all.iter().find(|(i, _)| i == id).map(|(_, s)| s.clone()))
                    .collect();
                ProtoError::new(
                    ErrorCode::Ambiguous,
                    format!("ambiguous; candidates: {}", names.join(", ")),
                )
            }
        })
    }

    fn summaries(&self) -> Vec<EntrySummary> {
        let entries = self.store.entries();
        let ids: Vec<EntryId> = entries.iter().map(|e| e.id).collect();
        let shorts = ShortId::compute(&ids);
        entries
            .iter()
            .zip(shorts)
            .enumerate()
            .map(|(i, (e, short))| EntrySummary {
                id: e.id.to_string(),
                short_id: short.to_string(),
                index: u32::try_from(i).unwrap_or(u32::MAX),
                size: e.size,
                created: e.created,
                last_copied: e.last_copied,
                pinned: e.pinned,
                preview: e.preview.as_str().to_string(),
            })
            .collect()
    }

    fn summary_of(&self, id: &EntryId) -> Option<EntrySummary> {
        self.summaries()
            .into_iter()
            .find(|s| s.id == id.to_string())
    }

    fn list(&self, limit: Option<u32>) -> Value {
        let mut all = self.summaries();
        if let Some(n) = limit {
            all.truncate(n as usize);
        }
        json!({ "entries": all })
    }

    fn show(&self, r: &EntryRef) -> Result<Value, ProtoError> {
        let id = self.resolve(r)?.id;
        let bytes = self.store.content(&id).map_err(|e| {
            let c = Correlation::next();
            tracing::error!(kind = %error_kind(&e), correlation = %c, "show: content unreadable");
            ProtoError::internal(c)
        })?;
        let entry = self.summary_of(&id);
        Ok(json!({"entry": entry, "content": String::from_utf8_lossy(&bytes)}))
    }

    fn pin(&mut self, r: &EntryRef, pin: bool) -> Result<Value, ProtoError> {
        let id = self.resolve(r)?.id;
        let m = if pin {
            Mutation::Pin(id)
        } else {
            Mutation::Unpin(id)
        };
        let outcomes = self.commit(vec![m])?;
        let changed = matches!(
            outcomes.first(),
            Some(Outcome::Pinned(true) | Outcome::Unpinned(true))
        );
        if changed {
            self.publish_history();
        }
        Ok(json!({"entry": self.summary_of(&id)}))
    }

    fn rm(&mut self, r: &EntryRef) -> Result<Value, ProtoError> {
        let id = self.resolve(r)?.id;
        self.commit(vec![Mutation::Remove(id)])?;
        self.publish_history();
        Ok(json!({}))
    }

    fn clear(&mut self, all: bool) -> Result<Value, ProtoError> {
        let m = if all {
            Mutation::ClearAll
        } else {
            Mutation::Clear
        };
        let removed = match self.commit(vec![m])?.pop() {
            Some(Outcome::Cleared(ids)) => ids.len(),
            _ => 0,
        };
        if removed > 0 {
            self.publish_history();
        }
        Ok(json!({"removed": removed}))
    }

    fn pause(&mut self, duration: Option<Duration>) -> Result<Value, ProtoError> {
        let before = self.rec;
        let changed = self
            .rec
            .pause(self.clock.now(), duration)
            .map_err(|e: PauseError| ProtoError::bad_request(e.to_string()))?;
        self.persist_pause(before, changed)
    }

    fn resume(&mut self) -> Result<Value, ProtoError> {
        let before = self.rec;
        let changed = self.rec.resume();
        self.persist_pause(before, changed)
    }

    fn persist_pause(
        &mut self,
        before: crate::domain::pause::RecordingState,
        changed: bool,
    ) -> Result<Value, ProtoError> {
        if changed {
            if let Err(e) = self.commit(vec![Mutation::SetPause(self.rec.state())]) {
                self.rec = before;
                return Err(e);
            }
            self.publish_pause();
        }
        Ok(json!({"state": self.state_view()}))
    }

    pub(crate) fn status(&self) -> Status {
        let h = self.store.history();
        Status {
            state: self.state_view(),
            backend: self.info.kind.as_str().to_string(),
            mode: self.info.mode_name(),
            entries: u32::try_from(h.len()).unwrap_or(u32::MAX),
            pinned: u32::try_from(h.entries().iter().filter(|e| e.pinned).count())
                .unwrap_or(u32::MAX),
            max_entries: self.caps.max_entries,
            max_pinned: self.caps.max_pinned,
            counters: self.counters,
            tray: "disabled".to_string(),
            picker_available: matches!(self.picker_plan(), LaunchPlan::Found { .. }),
            version: env!("CARGO_PKG_VERSION").to_string(),
            protocol: VERSION,
        }
    }

    fn picker_plan(&self) -> LaunchPlan {
        launch::plan(
            self.launch.picker_command.as_deref(),
            self.launch.terminal.as_deref(),
            &*self.launch.on_path,
        )
    }

    fn open_picker(&self) -> Result<Value, ProtoError> {
        let plan = self.picker_plan();
        let LaunchPlan::Found { argv, .. } = &plan else {
            return Err(ProtoError::new(
                ErrorCode::NoTerminal,
                "no terminal emulator found; set picker_command in the config",
            ));
        };
        let program = argv.first().cloned().unwrap_or_default();
        launch::spawn(&plan).map_err(|_| {
            ProtoError::new(
                ErrorCode::PickerCommandFailed,
                format!("could not start {program}"),
            )
        })?;
        Ok(json!({"argv": argv}))
    }
}

struct ShortIds(Vec<(EntryId, String)>);

impl ShortIds {
    fn of<B: Backend, F: Fs>(d: &Daemon<B, F>) -> Self {
        let ids: Vec<EntryId> = d.store.entries().iter().map(|e| e.id).collect();
        let shorts = ShortId::compute(&ids);
        Self(
            ids.into_iter()
                .zip(shorts.into_iter().map(|s| s.to_string()))
                .collect(),
        )
    }
}
