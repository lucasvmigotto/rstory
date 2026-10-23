//! The daemon: event loop, single instance, timers.
//!
//! One current-thread task owns the store and the backend. Socket requests, backend events
//! and the one deadline timer are serialized in `Daemon::run`, so there are no lost updates.

#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

pub mod handlers;
pub mod log;
pub mod pipeline;
pub mod restore;
pub mod start;
pub mod state;
pub mod timers;

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use time::OffsetDateTime;
use tokio::net::UnixListener;
use tokio::sync::{mpsc, watch};

use crate::backend::{Backend, BackendEvent};
use crate::config::Caps;
use crate::domain::pause::RecordingState;
use crate::ipc::protocol::{Counters, Event};
use crate::ipc::server::{self, Call, Link};
use crate::store::Store;
use crate::store::fs::Fs;
use crate::store::{Mutation, StoreError};
use pipeline::{Filter, Pending};
use state::{BackendInfo, Backoff, Clock};

/// Capture queue bound (FR-004).
pub const QUEUE_BOUND: usize = 256;

/// Opens a fresh backend after a disconnect (bridge mode reconnect).
pub type Connector<B> = Box<dyn FnMut() -> Pin<Box<dyn Future<Output = Option<B>> + Send>> + Send>;

/// How `open_picker` and `status.picker_available` find a terminal.
#[derive(Clone)]
pub struct LaunchEnv {
    /// `picker_command` from the config.
    pub picker_command: Option<String>,
    /// `$TERMINAL`.
    pub terminal: Option<String>,
    /// Whether a program is runnable.
    pub on_path: Arc<dyn Fn(&str) -> bool + Send + Sync>,
}

impl LaunchEnv {
    /// The process environment: `$TERMINAL` and the real `PATH`.
    #[must_use]
    pub fn from_env(picker_command: Option<String>) -> Self {
        Self {
            picker_command,
            terminal: std::env::var("TERMINAL").ok(),
            on_path: Arc::new(crate::launch::on_path),
        }
    }
}

/// Counters the tests read to check that the loop only wakes for work.
#[derive(Debug, Clone, Default)]
pub struct Probe {
    /// Loop iterations since the daemon started.
    pub wakeups: Arc<AtomicU64>,
}

/// The daemon core.
pub struct Daemon<B: Backend, F: Fs> {
    pub(crate) store: Store<F>,
    pub(crate) backend: Option<B>,
    pub(crate) info: BackendInfo,
    pub(crate) rec: RecordingState,
    pub(crate) counters: Counters,
    pub(crate) clock: Clock,
    pub(crate) caps: Caps,
    pub(crate) launch: LaunchEnv,
    pub(crate) link: Link,
    pub(crate) queue: VecDeque<Pending>,
    pub(crate) filter: Filter,
    connector: Option<Connector<B>>,
    backoff: Backoff,
    reconnect_at: Option<OffsetDateTime>,
    saved_info: Option<BackendInfo>,
    calls: Option<mpsc::Receiver<Call>>,
    probe: Probe,
}

enum Woke {
    Shutdown,
    Call(Option<Call>),
    Event(Option<BackendEvent>),
    Timer,
}

impl<B: Backend, F: Fs> Daemon<B, F> {
    /// Builds a daemon around an exclusively locked `store`. `backend` is `None` for the
    /// inactive state, with `info.reason` saying why. A persisted pause is honoured; one whose
    /// end has passed resumes (and is persisted again).
    ///
    /// # Errors
    /// The store failure while persisting an expired pause.
    pub fn new(
        mut store: Store<F>,
        backend: Option<B>,
        info: BackendInfo,
        caps: Caps,
        clock: Clock,
        launch: LaunchEnv,
    ) -> Result<Self, StoreError> {
        let (rec, changed) = RecordingState::load(store.pause(), clock.now());
        if changed {
            store.apply(vec![Mutation::SetPause(rec.state())], clock.now())?;
        }
        let (link, calls) = Link::new();
        Ok(Self {
            store,
            backend,
            info,
            rec,
            counters: Counters::default(),
            clock,
            caps,
            launch,
            link,
            queue: VecDeque::new(),
            filter: pipeline::pass_through(),
            connector: None,
            backoff: Backoff::new(),
            reconnect_at: None,
            saved_info: None,
            calls: Some(calls),
            probe: Probe::default(),
        })
    }

    /// The link socket servers and the tray use to reach this daemon.
    #[must_use]
    pub fn link(&self) -> Link {
        self.link.clone()
    }

    /// Test probe (loop wake-up counter).
    #[must_use]
    pub fn probe(&self) -> Probe {
        self.probe.clone()
    }

    /// Replaces the capture filter (spec 005 hook point).
    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }

    /// Reconnect policy: after a `Disconnected` the daemon is inactive and calls `connector` at
    /// 100 ms, 200 ms, ... up to 5 s until it yields a backend.
    pub fn set_connector(&mut self, connector: Connector<B>) {
        self.connector = Some(connector);
    }

    /// The instant the single timer is armed for.
    pub(crate) fn deadline(&self) -> Option<OffsetDateTime> {
        timers::next_deadline(
            self.store.history().next_deadline(),
            self.rec.until(),
            self.reconnect_at,
        )
    }

    /// Runs until `shutdown` turns true. Returns the store so the caller releases the lock last.
    pub async fn run(mut self, mut shutdown: watch::Receiver<bool>) -> Store<F> {
        todo!("daemon loop")
    }

    /// Ends a due timed pause and expires due entries, in one store commit.
    pub(crate) fn housekeeping(&mut self) {
        let now = self.clock.now();
        let mut batch = Vec::new();
        let mut rec = self.rec;
        let resumed = rec.tick(now);
        if resumed {
            batch.push(Mutation::SetPause(rec.state()));
        }
        let expiring = self
            .store
            .history()
            .next_deadline()
            .is_some_and(|t| t <= now);
        if expiring {
            batch.push(Mutation::Expire);
        }
        if batch.is_empty() {
            return;
        }
        match self.store.apply(batch, now) {
            Ok(outcomes) => {
                self.rec = rec;
                if resumed {
                    self.publish_pause();
                }
                let removed = outcomes
                    .iter()
                    .any(|o| matches!(o, crate::store::Outcome::Expired(ids) if !ids.is_empty()));
                if removed {
                    self.publish_history();
                }
            }
            Err(e) => tracing::error!(kind = %error_kind(&e), "housekeeping failed"),
        }
    }

    pub(crate) fn publish_history(&self) {
        let h = self.store.history();
        let top = h.entries().first().map(|e| e.id.to_string());
        self.link.publish(Event::history_changed(h.len(), top));
    }

    pub(crate) fn publish_pause(&self) {
        self.link.publish(Event::pause_changed(&self.state_view()));
    }

    pub(crate) fn state_view(&self) -> crate::ipc::protocol::State {
        state::view(
            &self.rec,
            self.backend.is_some(),
            self.info.reason.as_deref(),
        )
    }

    /// Switches to inactive after the display connection was lost.
    pub(crate) fn lose_backend(&mut self) {
        if self.backend.take().is_none() {
            return;
        }
        self.saved_info = Some(std::mem::replace(
            &mut self.info,
            BackendInfo::none("disconnected"),
        ));
        self.queue.clear();
        tracing::warn!("daemon.state{{state=inactive, reason=disconnected}}");
        if self.connector.is_some() {
            let delay = self.backoff.next_delay();
            self.reconnect_at = Some(self.clock.now() + delay);
        }
        self.publish_pause();
    }

    async fn maybe_reconnect(&mut self) {
        let due = self.reconnect_at.is_some_and(|t| t <= self.clock.now());
        if !due {
            return;
        }
        self.reconnect_at = None;
        let Some(connector) = self.connector.as_mut() else {
            return;
        };
        if let Some(b) = connector().await {
            self.backend = Some(b);
            if let Some(info) = self.saved_info.take() {
                self.info = info;
            }
            self.backoff.reset();
            tracing::info!("daemon.state{{state=recording}}");
            self.publish_pause();
        } else {
            let delay = self.backoff.next_delay();
            self.reconnect_at = Some(self.clock.now() + delay);
        }
    }
}

/// Serves `daemon` on `listener` until `shutdown`; returns the store for the caller to drop.
pub async fn run_with_listener<B: Backend, F: Fs>(
    daemon: Daemon<B, F>,
    listener: UnixListener,
    shutdown: watch::Receiver<bool>,
) -> Store<F> {
    let uid = rustix::process::getuid().as_raw();
    let server = tokio::spawn(server::serve(
        listener,
        daemon.link(),
        uid,
        shutdown.clone(),
    ));
    let store = daemon.run(shutdown).await;
    let _ = server.await;
    store
}

pub(crate) fn error_kind(e: &StoreError) -> &'static str {
    match e {
        StoreError::StoreTooOpen { .. } => "store_too_open",
        StoreError::UnknownVersion(_) => "unknown_version",
        StoreError::Locked => "locked",
        StoreError::DaemonRunning => "daemon_running",
        StoreError::TooLarge { .. } => "too_large",
        StoreError::Corrupt { .. } => "corrupt",
        StoreError::History(_) => "history",
        StoreError::Io(_) => "io",
    }
}
