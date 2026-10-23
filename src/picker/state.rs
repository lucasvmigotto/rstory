//! The picker's pure core: [`PickerState`], the messages it reacts to ([`Msg`]) and the effects
//! it asks for ([`Cmd`]). [`update`] does no I/O, reads no clock and draws nothing, so every
//! journey is a test over a list of messages (Constitution II).
//!
//! Shape: `specs/007-picker/data-model.md`. The terminal, the socket and the store live in
//! `term`, `data` and `app`; they turn keys and answers into [`Msg`] values and carry out the
//! returned [`Cmd`] values. The core only knows entries by id (the daemon accepts a full id as
//! a ref), so a refresh never retargets an action to another entry.
//!
//! Privacy (Constitution III): entries are summaries with escaped previews; full content only
//! arrives in [`Msg::Content`] for the preview pane, and no status line ever quotes it.

use std::collections::HashMap;

use time::OffsetDateTime;

use crate::ipc::protocol::EntrySummary;
use crate::ui::widgets::RecState;

/// Exit status when an entry was restored.
pub const EXIT_RESTORED: u8 = 0;
/// Exit status when the picker was dismissed with Esc.
pub const EXIT_DISMISSED: u8 = 1;
/// Exit status when there is no terminal.
pub const EXIT_NO_TTY: u8 = 2;
/// Exit status when a restore needed the daemon and it was not running.
pub const EXIT_DAEMON_DOWN: u8 = 3;
/// Exit status when interrupted (Ctrl-C).
pub const EXIT_INTERRUPTED: u8 = 130;

/// A key press, independent of the terminal library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// A printable character (Shift already applied).
    Char(char),
    /// Ctrl plus a letter, lowercase (`Ctrl('t')`).
    Ctrl(char),
    Enter,
    Esc,
    Tab,
    Backspace,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    /// A function key, `F(1)` for F1.
    F(u8),
}

/// What the picker knows about the history at one moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Newest first, as `list` returns them.
    pub entries: Vec<EntrySummary>,
    /// The time ages are computed against.
    pub now: OffsetDateTime,
    /// The recording state; `None` when unknown (read-only).
    pub rec: Option<RecState>,
}

/// An action sent to the daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Restore,
    Pin,
    Unpin,
    Remove,
}

/// Why an action failed. Messages carry codes, ids and limits only, never content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// The daemon is gone (no socket, or it closed).
    DaemonDown,
    /// Pinning would exceed `max_pinned`.
    PinLimit,
    /// Anything else, with a one-line message for the status line.
    Other(String),
}

/// An input to [`update`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    /// A key press.
    Key(Key),
    /// The terminal size changed (also sent once before the first frame).
    Resize { width: u16, height: u16 },
    /// A fresh listing (initial load or after a change).
    Loaded(Snapshot),
    /// A `history_changed` event arrived.
    HistoryChanged,
    /// The full content of an entry, for the preview.
    Content { id: String, text: String },
    /// The content could not be read (the entry is gone, or the source failed).
    ContentFailed { id: String },
    /// An action finished.
    Done(Action, Result<(), Failure>),
}

/// How the process should end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exit {
    /// The exit status (`EXIT_*`).
    pub code: u8,
    /// A line for stderr, printed after the terminal is restored.
    pub message: Option<String>,
}

impl Exit {
    /// An exit without a message.
    #[must_use]
    pub fn code(code: u8) -> Self {
        Self {
            code,
            message: None,
        }
    }
}

/// An effect for the runtime to carry out. Ids are full entry ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cmd {
    /// `restore`; answer with `Msg::Done(Action::Restore, _)`.
    Restore(String),
    /// `pin`; answer with `Msg::Done(Action::Pin, _)`.
    Pin(String),
    /// `unpin`; answer with `Msg::Done(Action::Unpin, _)`.
    Unpin(String),
    /// `rm`; answer with `Msg::Done(Action::Remove, _)`.
    Remove(String),
    /// Fetch the content for the preview (the runtime may debounce); answer with
    /// `Msg::Content` or `Msg::ContentFailed`.
    Show(String),
    /// Re-`list`; answer with `Msg::Loaded`.
    Reload,
    /// Restore the terminal and exit.
    Quit(Exit),
}

/// Whether the preview pane is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preview {
    Visible,
    Hidden,
}

/// The input mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Typing edits the query.
    Normal,
    /// The footer asks to delete this entry; `y` confirms, `n` or Esc cancels.
    ConfirmDelete(String),
    /// A restore is in flight; keys other than Ctrl-C wait.
    Restoring,
}

/// One visible row: an index into the entries, its score and its matched graphemes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Index into [`PickerState::entries`].
    pub idx: usize,
    /// Match score (0 with an empty query).
    pub score: u32,
    /// Grapheme indexes of the preview that matched, ascending.
    pub positions: Vec<usize>,
}

/// Width from which the preview is visible by default (spec US1 scenario 3).
pub const PREVIEW_DEFAULT_WIDTH: u16 = 80;

/// The smallest usable terminal (spec edge cases).
pub const MIN_SIZE: (u16, u16) = (20, 5);

/// How many cached contents the preview keeps.
const CONTENT_CACHE: usize = 32;

/// Rows taken by everything but the list: query, rules, footer, and the banner and the
/// two-line keymap when shown.
#[must_use]
pub fn chrome_rows(read_only: bool, keymap: bool) -> u16 {
    4 + u16::from(read_only) + u16::from(keymap)
}

/// The picker state. Fields are private; the view reads it through accessors.
#[derive(Debug, Clone)]
pub struct PickerState {
    query: String,
    entries: Vec<EntrySummary>,
    rows: Vec<Row>,
    selected: Option<String>,
    preview: Option<Preview>,
    mode: Mode,
    read_only: bool,
    status: Option<String>,
    keymap: bool,
    size: (u16, u16),
    now: OffsetDateTime,
    rec: Option<RecState>,
    contents: HashMap<String, String>,
    requested: Option<String>,
}

impl PickerState {
    /// A state over `snapshot`, newest entry selected.
    #[must_use]
    pub fn new(snapshot: Snapshot, read_only: bool, size: (u16, u16)) -> Self {
        let mut s = Self {
            query: String::new(),
            entries: Vec::new(),
            rows: Vec::new(),
            selected: None,
            preview: None,
            mode: Mode::Normal,
            read_only,
            status: None,
            keymap: false,
            size,
            now: snapshot.now,
            rec: None,
            contents: HashMap::new(),
            requested: None,
        };
        s.load(snapshot);
        s
    }

    /// Replaces the entries, keeping the selection on the same id when it is still listed,
    /// else on the same position.
    fn load(&mut self, snapshot: Snapshot) {
        let old_pos = self.selected_pos();
        self.now = snapshot.now;
        self.rec = snapshot.rec;
        self.entries = snapshot.entries;
        self.contents
            .retain(|id, _| self.entries.iter().any(|e| &e.id == id));
        self.refilter();
        if self.selected_pos().is_none() {
            let pos = old_pos.unwrap_or(0).min(self.rows.len().saturating_sub(1));
            self.select_pos(pos);
        }
    }

    /// Recomputes the visible rows for the current query.
    fn refilter(&mut self) {
        self.rows = (0..self.entries.len())
            .map(|idx| Row {
                idx,
                score: 0,
                positions: Vec::new(),
            })
            .collect();
    }

    /// Selects the row at `pos`, or nothing when there are no rows.
    fn select_pos(&mut self, pos: usize) {
        self.selected = self.rows.get(pos).map(|r| self.entries[r.idx].id.clone());
    }

    /// Moves the selection by `delta` rows, clamped to the list.
    fn move_by(&mut self, delta: isize) {
        if self.rows.is_empty() {
            return;
        }
        let cur = self.selected_pos().unwrap_or(0);
        let last = self.rows.len() - 1;
        let to = cur.saturating_add_signed(delta).min(last);
        self.select_pos(to);
    }

    /// Rows a page jump moves.
    fn page(&self) -> isize {
        isize::try_from(self.list_rows().max(1)).unwrap_or(1)
    }

    /// The id whose content the preview needs and nobody has asked for yet.
    fn wanted_content(&self) -> Option<String> {
        if self.preview() != Preview::Visible {
            return None;
        }
        let id = self.selected.as_ref()?;
        if self.contents.contains_key(id) || self.requested.as_ref() == Some(id) {
            return None;
        }
        Some(id.clone())
    }

    /// The query as typed.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Every entry, newest first.
    #[must_use]
    pub fn entries(&self) -> &[EntrySummary] {
        &self.entries
    }

    /// The visible rows, best match first.
    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The selected entry's id.
    #[must_use]
    pub fn selected_id(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Position of the selection in [`PickerState::rows`].
    #[must_use]
    pub fn selected_pos(&self) -> Option<usize> {
        let id = self.selected.as_deref()?;
        self.rows.iter().position(|r| self.entries[r.idx].id == id)
    }

    /// The selected entry.
    #[must_use]
    pub fn selected_entry(&self) -> Option<&EntrySummary> {
        self.selected_pos().map(|p| &self.entries[self.rows[p].idx])
    }

    /// Whether the preview is shown: Tab's choice, else visible from 80 columns.
    #[must_use]
    pub fn preview(&self) -> Preview {
        self.preview
            .unwrap_or(if self.size.0 >= PREVIEW_DEFAULT_WIDTH {
                Preview::Visible
            } else {
                Preview::Hidden
            })
    }

    /// The input mode.
    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// True when the daemon is down and the list came from the store.
    #[must_use]
    pub fn read_only(&self) -> bool {
        self.read_only
    }

    /// A status line replacing the footer hints.
    #[must_use]
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Whether F1's full keymap is shown.
    #[must_use]
    pub fn keymap(&self) -> bool {
        self.keymap
    }

    /// The terminal size, `(width, height)`.
    #[must_use]
    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    /// The time ages are computed against.
    #[must_use]
    pub fn now(&self) -> OffsetDateTime {
        self.now
    }

    /// The recording state, when known.
    #[must_use]
    pub fn rec(&self) -> Option<&RecState> {
        self.rec.as_ref()
    }

    /// The cached content of the selected entry.
    #[must_use]
    pub fn selected_content(&self) -> Option<&str> {
        self.contents
            .get(self.selected.as_deref()?)
            .map(String::as_str)
    }

    /// Whether the terminal is below [`MIN_SIZE`].
    #[must_use]
    pub fn too_small(&self) -> bool {
        self.size.0 < MIN_SIZE.0 || self.size.1 < MIN_SIZE.1
    }

    /// Rows the list gets at the current size.
    #[must_use]
    pub fn list_rows(&self) -> u16 {
        self.size
            .1
            .saturating_sub(chrome_rows(self.read_only, self.keymap))
    }
}

/// Applies `msg` to `state` and returns the effects to carry out, in order.
pub fn update(state: &mut PickerState, msg: Msg) -> Vec<Cmd> {
    let mut cmds = match msg {
        Msg::Key(key) => on_key(state, key),
        Msg::Resize { width, height } => {
            state.size = (width, height);
            Vec::new()
        }
        Msg::Loaded(snapshot) => {
            state.load(snapshot);
            Vec::new()
        }
        Msg::HistoryChanged => vec![Cmd::Reload],
        Msg::Content { id, text } => {
            if state.requested.as_ref() == Some(&id) {
                state.requested = None;
            }
            if state.contents.len() >= CONTENT_CACHE {
                let keep = state.selected.clone();
                state.contents.retain(|k, _| Some(k) == keep.as_ref());
            }
            state.contents.insert(id, text);
            Vec::new()
        }
        Msg::ContentFailed { id } => {
            if state.requested.as_ref() == Some(&id) {
                state.requested = None;
            }
            Vec::new()
        }
        Msg::Done(action, result) => on_done(state, action, result),
    };
    let quitting = cmds.iter().any(|c| matches!(c, Cmd::Quit(_)));
    if !quitting && let Some(id) = state.wanted_content() {
        state.requested = Some(id.clone());
        cmds.push(Cmd::Show(id));
    }
    cmds
}

fn on_key(state: &mut PickerState, key: Key) -> Vec<Cmd> {
    // Ctrl-C always interrupts, whatever the mode.
    if key == Key::Ctrl('c') {
        return vec![Cmd::Quit(Exit::code(EXIT_INTERRUPTED))];
    }
    match key {
        Key::Esc => return vec![Cmd::Quit(Exit::code(EXIT_DISMISSED))],
        Key::Tab => {
            state.preview = Some(match state.preview() {
                Preview::Visible => Preview::Hidden,
                Preview::Hidden => Preview::Visible,
            });
        }
        Key::Up | Key::Ctrl('p') => state.move_by(-1),
        Key::Down | Key::Ctrl('n') => state.move_by(1),
        Key::PageUp => state.move_by(-state.page()),
        Key::PageDown => state.move_by(state.page()),
        Key::Home => state.select_pos(0),
        Key::End => state.select_pos(state.rows.len().saturating_sub(1)),
        _ => {}
    }
    Vec::new()
}

fn on_done(state: &mut PickerState, action: Action, result: Result<(), Failure>) -> Vec<Cmd> {
    let _ = (state, action, result);
    Vec::new()
}

/// Synthetic fixtures shared by the picker's tests. No real clipboard content.
#[cfg(test)]
pub(crate) mod fixtures {
    use time::macros::datetime;
    use time::{Duration, OffsetDateTime};

    use super::Snapshot;
    use crate::ipc::protocol::EntrySummary;
    use crate::ui::widgets::RecState;

    /// The fixed "now" of every fixture.
    pub const NOW: OffsetDateTime = datetime!(2026-10-21 14:00:00 UTC);

    /// A full 26-character id ending in `n`.
    pub fn id(n: u32) -> String {
        format!("01JB{n:022}")
    }

    /// An entry `n`, copied `mins` minutes before [`NOW`].
    pub fn entry(n: u32, preview: &str, mins: i64, pinned: bool) -> EntrySummary {
        let at = NOW - Duration::minutes(mins);
        EntrySummary {
            id: id(n),
            short_id: format!("{:04}", n),
            index: 0,
            size: u32::try_from(preview.len()).unwrap_or(u32::MAX),
            created: at,
            last_copied: at,
            pinned,
            preview: preview.to_string(),
        }
    }

    /// A snapshot over `entries`, re-indexed newest first, recording.
    pub fn snapshot(mut entries: Vec<EntrySummary>) -> Snapshot {
        for (i, e) in entries.iter_mut().enumerate() {
            e.index = u32::try_from(i).unwrap_or(u32::MAX);
        }
        Snapshot {
            entries,
            now: NOW,
            rec: Some(RecState::Recording),
        }
    }

    /// Entries B (newest) and A, the spec's independent test.
    pub fn b_and_a() -> Snapshot {
        snapshot(vec![
            entry(2, "git reset --soft HEAD~1 && git push", 3, false),
            entry(1, "git rebase -i HEAD~3", 120, false),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::{b_and_a, entry, id, snapshot};
    use super::*;

    fn keys(state: &mut PickerState, ks: &[Key]) -> Vec<Cmd> {
        ks.iter()
            .flat_map(|k| update(state, Msg::Key(*k)))
            .collect()
    }

    #[test]
    fn opens_with_the_newest_entry_selected_and_everything_listed() {
        let s = PickerState::new(b_and_a(), false, (80, 24));
        assert_eq!(s.selected_id(), Some(id(2).as_str()));
        assert_eq!(s.rows().len(), 2);
        assert_eq!(s.rows()[0].idx, 0);
        assert_eq!(s.query(), "");
        assert_eq!(s.mode(), &Mode::Normal);
        assert!(!s.read_only());
    }

    #[test]
    fn an_empty_history_selects_nothing() {
        let mut s = PickerState::new(snapshot(vec![]), false, (80, 24));
        assert_eq!(s.selected_id(), None);
        assert!(keys(&mut s, &[Key::Enter]).is_empty());
    }

    #[test]
    fn esc_dismisses_with_1_and_ctrl_c_interrupts_with_130() {
        let mut s = PickerState::new(b_and_a(), false, (80, 24));
        assert_eq!(
            keys(&mut s, &[Key::Esc]),
            vec![Cmd::Quit(Exit::code(EXIT_DISMISSED))]
        );
        let mut s = PickerState::new(b_and_a(), false, (80, 24));
        assert_eq!(
            keys(&mut s, &[Key::Ctrl('c')]),
            vec![Cmd::Quit(Exit::code(EXIT_INTERRUPTED))]
        );
    }

    #[test]
    fn the_preview_is_visible_from_80_columns_and_tab_toggles_it() {
        let mut s = PickerState::new(b_and_a(), false, (80, 24));
        assert_eq!(s.preview(), Preview::Visible);
        update(
            &mut s,
            Msg::Resize {
                width: 40,
                height: 10,
            },
        );
        assert_eq!(s.size(), (40, 10));
        assert_eq!(s.preview(), Preview::Hidden);
        keys(&mut s, &[Key::Tab]);
        assert_eq!(s.preview(), Preview::Visible);
        keys(&mut s, &[Key::Tab]);
        assert_eq!(s.preview(), Preview::Hidden);
    }

    #[test]
    fn showing_the_preview_asks_for_the_content_once() {
        let mut s = PickerState::new(b_and_a(), false, (40, 10));
        let cmds = keys(&mut s, &[Key::Tab]);
        assert_eq!(cmds, vec![Cmd::Show(id(2))]);
        // Hiding and showing again while the fetch is pending does not ask twice.
        assert!(keys(&mut s, &[Key::Tab, Key::Tab]).is_empty());
        update(
            &mut s,
            Msg::Content {
                id: id(2),
                text: "synthetic content".into(),
            },
        );
        assert_eq!(s.selected_content(), Some("synthetic content"));
    }

    #[test]
    fn a_reload_keeps_the_selection_on_the_same_id() {
        let mut s = PickerState::new(b_and_a(), false, (40, 10));
        keys(&mut s, &[Key::Down]);
        assert_eq!(s.selected_id(), Some(id(1).as_str()));
        let mut fresh = b_and_a().entries;
        fresh.insert(0, entry(3, "echo new", 0, false));
        update(&mut s, Msg::Loaded(snapshot(fresh)));
        assert_eq!(s.rows().len(), 3);
        assert_eq!(s.selected_id(), Some(id(1).as_str()));
    }

    #[test]
    fn a_history_change_asks_for_a_reload() {
        let mut s = PickerState::new(b_and_a(), false, (40, 10));
        assert_eq!(update(&mut s, Msg::HistoryChanged), vec![Cmd::Reload]);
    }

    #[test]
    fn too_small_below_20_by_5() {
        let mut s = PickerState::new(b_and_a(), false, (20, 5));
        assert!(!s.too_small());
        update(
            &mut s,
            Msg::Resize {
                width: 19,
                height: 5,
            },
        );
        assert!(s.too_small());
        update(
            &mut s,
            Msg::Resize {
                width: 20,
                height: 4,
            },
        );
        assert!(s.too_small());
    }

    #[test]
    fn update_is_deterministic() {
        let script = [
            Msg::Key(Key::Down),
            Msg::Key(Key::Tab),
            Msg::Resize {
                width: 100,
                height: 30,
            },
            Msg::HistoryChanged,
            Msg::Key(Key::Esc),
        ];
        let run = || {
            let mut s = PickerState::new(b_and_a(), false, (40, 10));
            let cmds: Vec<Cmd> = script
                .iter()
                .cloned()
                .flat_map(|m| update(&mut s, m))
                .collect();
            (cmds, s.selected_id().map(str::to_string), s.preview())
        };
        assert_eq!(run(), run());
    }
}
