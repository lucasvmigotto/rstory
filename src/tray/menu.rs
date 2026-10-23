//! The pure tray menu model (`specs/009-tray/contracts/menu.md`): a function of the history's
//! newest entries, the daemon state, `picker_available` and the clear-confirm step. No I/O.

use time::OffsetDateTime;

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
pub fn label_of(_preview: &str) -> String {
    todo!()
}

impl TrayMenu {
    /// Builds the model. `rows` is the history newest first (extra rows are ignored).
    #[must_use]
    pub fn build(
        _rows: &[EntryRow],
        _view: &View,
        _now: OffsetDateTime,
        _picker_available: bool,
        _clear: Clear,
    ) -> Self {
        todo!()
    }

    /// The items in contract order.
    #[must_use]
    pub fn items(&self) -> Vec<Item> {
        todo!()
    }
}
