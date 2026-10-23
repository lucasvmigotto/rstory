//! The pure tray menu model (`specs/009-tray/contracts/menu.md`): a function of the history's
//! newest entries, the daemon state, `picker_available` and the clear-confirm step. No I/O.

use time::OffsetDateTime;

use crate::ui::copy;
use crate::ui::glyphs::{Glyph, Glyphs};

/// Most entry items in the menu.
pub const MAX_ENTRIES: usize = 10;
/// Longest entry label, in characters, counting the closing `…`.
pub const LABEL_MAX: usize = 48;

/// One history row as the menu needs it.
#[derive(Clone, PartialEq, Eq)]
pub struct EntryRow {
    /// Short id the `restore` request takes.
    pub id: String,
    /// The daemon's escaped preview (content-derived: never logged).
    pub preview: String,
    /// Pinned entries get a pin glyph.
    pub pinned: bool,
}

// A preview is content-derived: never show it through `Debug`.
impl std::fmt::Debug for EntryRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryRow")
            .field("id", &self.id)
            .field("pinned", &self.pinned)
            .finish_non_exhaustive()
    }
}

/// The daemon state, reduced to what the menu shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    /// Capturing.
    Recording,
    /// Paused; `until` is the end of a timed pause.
    Paused {
        /// End of a timed pause.
        until: Option<OffsetDateTime>,
    },
    /// No capture backend.
    Inactive {
        /// Why.
        reason: String,
    },
}

/// The clear-history step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clear {
    /// Not asked yet.
    Idle,
    /// Waiting for the confirmation of deleting this many unpinned entries.
    Confirming(u32),
}

/// What activating an item does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Restore the entry with this short id.
    Restore(String),
    /// Pause recording.
    Pause,
    /// Resume recording.
    Resume,
    /// Pause for one minute.
    PauseMinute,
    /// Open the picker in a terminal.
    OpenPicker,
    /// Show the confirm step.
    ClearBegin,
    /// Send `clear`.
    ClearConfirm,
    /// Back out of the confirm step.
    ClearCancel,
}

/// Pause or Resume, whichever applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseItem {
    /// "Pause recording".
    Pause,
    /// "Resume recording".
    Resume,
}

/// A rendered item, in menu order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// An activatable item.
    Action {
        /// Text label.
        label: String,
        /// What it does.
        action: Action,
    },
    /// An insensitive text item.
    Label(String),
    /// A separator.
    Separator,
    /// A submenu.
    Submenu {
        /// Text label.
        label: String,
        /// Its items.
        children: Vec<Item>,
    },
}

/// The menu, derived purely from the daemon's snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayMenu {
    /// The newest entries, at most [`MAX_ENTRIES`], as `(short id, label, pinned)`.
    pub entries: Vec<(String, String, bool)>,
    /// The insensitive status line.
    pub status_label: String,
    /// Pause or Resume.
    pub pause_item: PauseItem,
    /// "Pause for 1 minute" is hidden while paused.
    pub show_pause_minute: bool,
    /// "Open picker" needs a terminal.
    pub show_open_picker: bool,
    /// The clear step.
    pub clear: Clear,
}

/// One-line label of a preview: newlines become `⏎`, other control characters go, at most
/// [`LABEL_MAX`] characters with a closing `…`.
#[must_use]
pub fn label_of(preview: &str) -> String {
    // The daemon already escaped newlines to the two characters `\n`; real ones are handled too.
    let mut out = String::new();
    let mut chars = preview.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'n') => {
                chars.next();
                out.push_str(NEWLINE);
            }
            '\r' if chars.peek() == Some(&'\n') => {}
            '\n' | '\r' => out.push_str(NEWLINE),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    if out.chars().count() <= LABEL_MAX {
        return out;
    }
    let mut cut: String = out.chars().take(LABEL_MAX - 1).collect();
    cut.push('…');
    cut
}

const NEWLINE: &str = "⏎";

fn remaining_label(until: OffsetDateTime, now: OffsetDateTime) -> String {
    let ms = (until - now).whole_milliseconds().max(0);
    // Round up: a pause that has not ended never reads 0:00.
    let secs = u64::try_from(ms).unwrap_or(0).div_ceil(1000).max(1);
    copy::format(
        copy::STATE_PAUSED_UNTIL,
        &[
            ("m", &(secs / 60).to_string()),
            ("ss", &format!("{:02}", secs % 60)),
        ],
    )
}

impl TrayMenu {
    /// Builds the model. `rows` is the history newest first (extra rows are ignored).
    #[must_use]
    pub fn build(
        rows: &[EntryRow],
        view: &View,
        now: OffsetDateTime,
        picker_available: bool,
        clear: Clear,
    ) -> Self {
        let entries = rows
            .iter()
            .take(MAX_ENTRIES)
            .map(|r| {
                let label = label_of(&r.preview);
                let label = if r.pinned {
                    format!("{} {label}", Glyphs::new(false).get(Glyph::Pin))
                } else {
                    label
                };
                (r.id.clone(), label, r.pinned)
            })
            .collect();
        let (status_label, pause_item) = match view {
            View::Recording => (copy::STATE_RECORDING.to_string(), PauseItem::Pause),
            View::Paused { until: None } => (copy::STATE_PAUSED.to_string(), PauseItem::Resume),
            View::Paused { until: Some(t) } => (remaining_label(*t, now), PauseItem::Resume),
            View::Inactive { reason } => (
                copy::format(copy::STATE_INACTIVE, &[("reason", reason)]),
                PauseItem::Pause,
            ),
        };
        Self {
            entries,
            status_label,
            pause_item,
            show_pause_minute: pause_item == PauseItem::Pause,
            show_open_picker: picker_available,
            clear,
        }
    }

    /// The items in contract order.
    #[must_use]
    pub fn items(&self) -> Vec<Item> {
        let mut out: Vec<Item> = Vec::new();
        if self.entries.is_empty() {
            out.push(Item::Label(copy::MENU_EMPTY.to_string()));
        }
        for (id, label, _) in &self.entries {
            out.push(Item::Action {
                label: label.clone(),
                action: Action::Restore(id.clone()),
            });
        }
        out.push(Item::Separator);
        out.push(Item::Label(self.status_label.clone()));
        out.push(match self.pause_item {
            PauseItem::Pause => action(copy::MENU_PAUSE, Action::Pause),
            PauseItem::Resume => action(copy::MENU_RESUME, Action::Resume),
        });
        if self.show_pause_minute {
            out.push(action(copy::MENU_PAUSE_MINUTE, Action::PauseMinute));
        }
        if self.show_open_picker {
            out.push(action(copy::MENU_OPEN_PICKER, Action::OpenPicker));
        }
        out.push(Item::Separator);
        out.push(match self.clear {
            Clear::Idle => action(copy::MENU_CLEAR, Action::ClearBegin),
            Clear::Confirming(n) => Item::Submenu {
                label: copy::MENU_CLEAR.to_string(),
                children: vec![
                    action(
                        &copy::format(copy::MENU_CLEAR_CONFIRM, &[("n", &n.to_string())]),
                        Action::ClearConfirm,
                    ),
                    action(copy::MENU_CLEAR_CANCEL, Action::ClearCancel),
                ],
            },
        });
        out
    }
}

fn action(label: &str, action: Action) -> Item {
    Item::Action {
        label: label.to_string(),
        action,
    }
}
