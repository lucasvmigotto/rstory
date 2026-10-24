//! Status-notifier tray icon and menu (spec 009).
//!
//! The tray lives in the daemon process and talks to the daemon through the same [`Link`] the
//! control socket uses, so it is a client and never writes the store. Nothing here logs or
//! formats clipboard content (Constitution III).

pub mod client;
pub mod icons;
pub mod menu;

use std::sync::Arc;
use std::time::Duration;

use ksni::menu::{StandardItem, SubMenu};
use ksni::{Handle, Icon, MenuItem, ToolTip, TrayMethods};
use time::OffsetDateTime;
use tokio::sync::{broadcast, mpsc, watch};
use tokio::task::JoinHandle;

use crate::domain::ids::EntryRef;
use crate::ipc::protocol::{Command, EntrySummary, StateKind, Status};
use crate::ipc::server::Link;
use icons::IconState;
use ksni::OfflineReason;
use menu::{Action, Clear, EntryRow, Item, TrayMenu, View};

/// `status.tray`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayStatus {
    /// Registered with a watcher.
    Active,
    /// A session bus, but no `StatusNotifierWatcher` on it.
    NoHost,
    /// No session bus.
    NoBus,
    /// Turned off (`tray = false`) or not started yet.
    Disabled,
}

impl TrayStatus {
    /// The wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::NoHost => "no_host",
            Self::NoBus => "no_bus",
            Self::Disabled => "disabled",
        }
    }
}

/// The protocol request an activated item sends, `None` for the steps that stay in the tray.
#[must_use]
pub fn command_for(action: &Action) -> Option<Command> {
    match action {
        Action::Restore(id) => {
            // Validate, then build the id reference directly: an all-digit id must never be
            // read as a list position.
            id.parse::<EntryRef>().ok()?;
            Some(Command::Restore(EntryRef::ShortId(id.to_ascii_uppercase())))
        }
        Action::Pause => Some(Command::Pause { duration: None }),
        Action::PauseMinute => Some(Command::Pause {
            duration: Some(PAUSE_MINUTE),
        }),
        Action::Resume => Some(Command::Resume),
        Action::OpenPicker => Some(Command::OpenPicker),
        Action::ClearConfirm => Some(Command::Clear { all: false }),
        Action::ClearBegin | Action::ClearCancel => None,
    }
}

/// "Pause for 1 minute" is fixed at 60 s in the MVP.
const PAUSE_MINUTE: Duration = Duration::from_secs(60);

/// Item labels go through the D-Bus menu, where a single `_` marks an access key: double them.
fn dbus_label(label: &str) -> String {
    label.replace('_', "__")
}

/// What the tray shows, refreshed from the daemon.
struct Snapshot {
    rows: Vec<EntryRow>,
    view: View,
    picker_available: bool,
    unpinned: u32,
}

enum Msg {
    Do(Action),
    Refresh,
}

struct RstoryTray {
    snap: Snapshot,
    clear: Clear,
    msgs: mpsc::UnboundedSender<Msg>,
    status: Arc<watch::Sender<TrayStatus>>,
}

fn set_status(tx: &watch::Sender<TrayStatus>, to: TrayStatus) {
    let changed = tx.send_if_modified(|s| {
        let c = *s != to;
        *s = to;
        c
    });
    if changed {
        tracing::info!(state = to.as_str(), "tray.state");
    }
}

impl RstoryTray {
    fn model(&self) -> TrayMenu {
        TrayMenu::build(
            &self.snap.rows,
            &self.snap.view,
            OffsetDateTime::now_utc(),
            self.snap.picker_available,
            self.clear,
        )
    }

    fn convert(items: Vec<Item>) -> Vec<MenuItem<Self>> {
        items
            .into_iter()
            .map(|item| match item {
                Item::Separator => MenuItem::Separator,
                Item::Label(label) => StandardItem {
                    label: dbus_label(&label),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
                Item::Action { label, action } => StandardItem {
                    label: dbus_label(&label),
                    activate: Box::new(move |t: &mut Self| {
                        let _ = t.msgs.send(Msg::Do(action.clone()));
                    }),
                    ..Default::default()
                }
                .into(),
                Item::Submenu { label, children } => SubMenu {
                    label: dbus_label(&label),
                    submenu: Self::convert(children),
                    ..Default::default()
                }
                .into(),
            })
            .collect()
    }
}

impl ksni::Tray for RstoryTray {
    fn id(&self) -> String {
        "rstory".to_string()
    }

    fn title(&self) -> String {
        "rstory".to_string()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let state = match self.snap.view {
            View::Recording => IconState::Recording,
            View::Paused { .. } => IconState::Paused,
            View::Inactive { .. } => IconState::Inactive,
        };
        icons::pixmaps(state)
            .into_iter()
            .map(|p| Icon {
                width: p.size,
                height: p.size,
                data: p.argb,
            })
            .collect()
    }

    fn tool_tip(&self) -> ToolTip {
        let word = match self.snap.view {
            View::Recording => "recording",
            View::Paused { .. } => "paused",
            View::Inactive { .. } => "inactive",
        };
        ToolTip {
            title: format!("rstory — {word}"),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        Self::convert(self.model().items())
    }

    fn menu_about_to_show(&mut self) {
        let _ = self.msgs.send(Msg::Refresh);
    }

    fn watcher_online(&self) {
        set_status(&self.status, TrayStatus::Active);
    }

    fn watcher_offline(&self, _reason: OfflineReason) -> bool {
        set_status(&self.status, TrayStatus::NoHost);
        true
    }
}

/// Reads the daemon's state and newest entries; `None` when the daemon is gone.
async fn snapshot(link: &Link) -> Option<Snapshot> {
    let status: Status = serde_json::from_value(link.request(Command::Status).await.ok()?).ok()?;
    let listed = link
        .request(Command::List {
            limit: Some(u32::try_from(menu::MAX_ENTRIES).unwrap_or(10)),
        })
        .await
        .ok()?;
    let entries: Vec<EntrySummary> = serde_json::from_value(listed.get("entries")?.clone()).ok()?;
    let view = match status.state.state {
        StateKind::Recording => View::Recording,
        StateKind::Paused => View::Paused {
            until: status.state.paused_until,
        },
        StateKind::Inactive => View::Inactive {
            reason: status.state.reason.clone().unwrap_or_default(),
        },
    };
    Some(Snapshot {
        rows: entries
            .into_iter()
            .map(|e| EntryRow {
                id: e.id,
                preview: e.preview,
                pinned: e.pinned,
            })
            .collect(),
        view,
        picker_available: status.picker_available,
        unpinned: status.entries.saturating_sub(status.pinned),
    })
}

async fn refresh(link: &Link, handle: &Handle<RstoryTray>, reset_clear: bool) -> bool {
    let Some(snap) = snapshot(link).await else {
        return false;
    };
    handle
        .update(|t| {
            let n = snap.unpinned;
            t.snap = snap;
            t.clear = match (t.clear, reset_clear) {
                (Clear::Confirming(_), false) => Clear::Confirming(n),
                _ => Clear::Idle,
            };
        })
        .await
        .is_some()
}

async fn perform(link: &Link, handle: &Handle<RstoryTray>, action: Action) {
    match action {
        Action::ClearBegin => {
            let _ = handle
                .update(|t| t.clear = Clear::Confirming(t.snap.unpinned))
                .await;
        }
        Action::ClearCancel => {
            let _ = handle.update(|t| t.clear = Clear::Idle).await;
        }
        other => {
            if let Some(command) = command_for(&other) {
                let op = command.op();
                if let Err(e) = link.request(command).await {
                    tracing::warn!(op, code = e.code.as_str(), "tray.action_failed");
                }
            }
            let _ = handle.update(|t| t.clear = Clear::Idle).await;
            let _ = refresh(link, handle, true).await;
        }
    }
}

async fn run(
    link: Link,
    mut shutdown: watch::Receiver<bool>,
    status: Arc<watch::Sender<TrayStatus>>,
) {
    let mut events = link.subscribe();
    let Some(snap) = snapshot(&link).await else {
        return;
    };
    let (msgs, mut inbox) = mpsc::unbounded_channel();
    let tray = RstoryTray {
        snap,
        clear: Clear::Idle,
        msgs,
        status: status.clone(),
    };
    // `assume_sni_available`: a missing watcher is not an error here; it is reported through
    // `watcher_offline`, and the item registers when the watcher appears.
    let spawned = tray.assume_sni_available(true).spawn().await;
    let handle = match spawned {
        Ok(h) => h,
        Err(ksni::Error::Dbus(_)) => {
            set_status(&status, TrayStatus::NoBus);
            return;
        }
        Err(_) => {
            set_status(&status, TrayStatus::NoHost);
            return;
        }
    };
    // Unless the watcher turned out to be absent while starting.
    if *status.borrow() == TrayStatus::Disabled {
        set_status(&status, TrayStatus::Active);
    }
    loop {
        tokio::select! {
            r = shutdown.changed() => {
                if r.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            msg = inbox.recv() => match msg {
                Some(Msg::Refresh) => {
                    if !refresh(&link, &handle, false).await { break; }
                }
                Some(Msg::Do(action)) => perform(&link, &handle, action).await,
                None => break,
            },
            ev = events.recv() => match ev {
                Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {
                    if !refresh(&link, &handle, true).await { break; }
                }
                Err(broadcast::error::RecvError::Closed) => break,
            },
        }
    }
    handle.shutdown().await;
}

/// Starts the tray on the current runtime. The receiver follows `status.tray` (`disabled` until
/// the first attempt settles); the task ends when `shutdown` turns true.
#[must_use]
pub fn spawn(
    link: Link,
    shutdown: watch::Receiver<bool>,
) -> (watch::Receiver<TrayStatus>, JoinHandle<()>) {
    let (tx, rx) = watch::channel(TrayStatus::Disabled);
    (rx, tokio::spawn(run(link, shutdown, Arc::new(tx))))
}

#[cfg(test)]
mod tests;
