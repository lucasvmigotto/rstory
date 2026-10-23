//! Journeys over `update()` with a fake daemon, and snapshots of every picker state
//! (spec 007 US1–US3, SC-003). All content is synthetic.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
};
use unicode_width::UnicodeWidthStr;

use super::app::drive;
use super::data::Source;
use super::state::fixtures::{NOW, b_and_a, entry, id, snapshot};
use super::state::*;
use super::view;
use crate::ui::glyphs::Glyphs;
use crate::ui::theme::{Background, Theme, Tier};
use crate::ui::widgets::Ctx;

// ---------------------------------------------------------------- fake daemon

/// A daemon double: serves `snapshot`, records every action, answers with `next`.
pub(crate) struct Fake {
    pub snapshot: Snapshot,
    pub read_only: bool,
    pub calls: Vec<(Action, String)>,
    pub shows: Vec<String>,
    pub lists: usize,
    /// The answer to the next action (`Ok` when `None`).
    pub next: Option<Failure>,
}

impl Fake {
    pub fn new(snapshot: Snapshot) -> Self {
        Self {
            snapshot,
            read_only: false,
            calls: Vec::new(),
            shows: Vec::new(),
            lists: 0,
            next: None,
        }
    }
}

impl Source for Fake {
    fn read_only(&self) -> bool {
        self.read_only
    }
    fn list(&mut self) -> Result<Snapshot, Failure> {
        self.lists += 1;
        Ok(self.snapshot.clone())
    }
    fn show(&mut self, id: &str) -> Result<String, Failure> {
        self.shows.push(id.to_string());
        self.snapshot
            .entries
            .iter()
            .find(|e| e.id == id)
            .map(|e| format!("{} (full)", e.preview))
            .ok_or(Failure::Other("not found".into()))
    }
    fn act(&mut self, action: Action, id: &str) -> Result<(), Failure> {
        self.calls.push((action, id.to_string()));
        if self.read_only {
            return Err(Failure::DaemonDown);
        }
        if let Some(f) = self.next.take() {
            return Err(f);
        }
        // Like the daemon: the next `list` shows the change.
        let entries = &mut self.snapshot.entries;
        match action {
            Action::Pin | Action::Unpin => {
                if let Some(e) = entries.iter_mut().find(|e| e.id == id) {
                    e.pinned = action == Action::Pin;
                }
            }
            Action::Remove => entries.retain(|e| e.id != id),
            Action::Restore => {}
        }
        Ok(())
    }
}

pub(crate) fn typed(text: &str) -> Vec<Msg> {
    text.chars().map(|c| Msg::Key(Key::Char(c))).collect()
}

pub(crate) fn key(k: Key) -> Msg {
    Msg::Key(k)
}

fn open(snap: Snapshot, size: (u16, u16)) -> (PickerState, Fake) {
    let fake = Fake::new(snap.clone());
    (PickerState::new(snap, false, size), fake)
}

fn previews(s: &PickerState) -> Vec<&str> {
    s.rows()
        .iter()
        .map(|r| s.entries()[r.idx].preview.as_str())
        .collect()
}

// ---------------------------------------------------------------- US1 journeys (T003)

#[test]
fn journey_k2_type_reb_and_enter_restores_a_and_exits_0() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    let mut inputs = typed("reb");
    inputs.push(key(Key::Enter));
    let exit = drive(&mut s, &mut fake, inputs);
    assert_eq!(exit, Some(Exit::code(EXIT_RESTORED)));
    assert_eq!(fake.calls, vec![(Action::Restore, id(1))]);
}

#[test]
fn journey_filter_keeps_only_matches_and_marks_them() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    drive(&mut s, &mut fake, typed("reb"));
    assert_eq!(s.query(), "reb");
    assert_eq!(previews(&s), vec!["git rebase -i HEAD~3"]);
    assert_eq!(s.rows()[0].positions, vec![4, 5, 6]);
    assert!(s.rows()[0].score > 0);
    assert_eq!(s.selected_id(), Some(id(1).as_str()));
}

#[test]
fn journey_rank_is_score_then_recency() {
    let snap = snapshot(vec![
        entry(4, "cargo bench --no-run", 1, false),
        entry(3, "a-b-c-d", 2, false),
        entry(2, "abcd", 3, false),
        entry(1, "abcd", 4, false),
    ]);
    let (mut s, mut fake) = open(snap, (80, 24));
    drive(&mut s, &mut fake, typed("abcd"));
    // The contiguous matches score higher; between equal scores the newer one wins.
    assert_eq!(previews(&s), vec!["abcd", "abcd", "a-b-c-d"]);
    let first = &s.entries()[s.rows()[0].idx];
    assert_eq!(first.id, id(2));
    assert!(s.rows()[0].score > s.rows()[2].score);
}

#[test]
fn journey_matching_is_case_insensitive_then_smart_case() {
    let snap = snapshot(vec![
        entry(2, "Makefile target", 1, false),
        entry(1, "make test", 2, false),
    ]);
    let (mut s, mut fake) = open(snap, (80, 24));
    drive(&mut s, &mut fake, typed("make"));
    assert_eq!(s.rows().len(), 2);
    drive(&mut s, &mut fake, vec![key(Key::Ctrl('u'))]);
    drive(&mut s, &mut fake, typed("Make"));
    assert_eq!(previews(&s), vec!["Makefile target"]);
}

#[test]
fn journey_no_match_shows_nothing_and_enter_does_nothing() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    let mut inputs = typed("zzzz");
    inputs.push(key(Key::Enter));
    assert_eq!(drive(&mut s, &mut fake, inputs), None);
    assert!(s.rows().is_empty());
    assert_eq!(s.selected_id(), None);
    assert!(fake.calls.is_empty());
}

#[test]
fn journey_query_editing_backspace_ctrl_u_ctrl_w() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    drive(&mut s, &mut fake, typed("git reb"));
    drive(&mut s, &mut fake, vec![key(Key::Backspace)]);
    assert_eq!(s.query(), "git re");
    drive(&mut s, &mut fake, vec![key(Key::Ctrl('w'))]);
    assert_eq!(s.query(), "git ");
    drive(&mut s, &mut fake, vec![key(Key::Ctrl('w'))]);
    assert_eq!(s.query(), "");
    drive(&mut s, &mut fake, typed("x y"));
    drive(&mut s, &mut fake, vec![key(Key::Ctrl('u'))]);
    assert_eq!(s.query(), "");
    assert_eq!(s.rows().len(), 2);
    // Backspace on an empty query is harmless.
    drive(&mut s, &mut fake, vec![key(Key::Backspace)]);
    assert_eq!(s.query(), "");
}

#[test]
fn journey_selection_moves_and_jumps() {
    let entries = (1..=30)
        .rev()
        .map(|n| entry(n, &format!("synthetic entry {n}"), i64::from(30 - n), false))
        .collect();
    let (mut s, mut fake) = open(snapshot(entries), (80, 24));
    let sel = |s: &PickerState| s.selected_pos().unwrap();
    assert_eq!(sel(&s), 0);
    drive(&mut s, &mut fake, vec![key(Key::Down), key(Key::Ctrl('n'))]);
    assert_eq!(sel(&s), 2);
    drive(&mut s, &mut fake, vec![key(Key::Up), key(Key::Ctrl('p'))]);
    assert_eq!(sel(&s), 0);
    drive(&mut s, &mut fake, vec![key(Key::Up)]);
    assert_eq!(sel(&s), 0, "clamped at the top");
    drive(&mut s, &mut fake, vec![key(Key::PageDown)]);
    assert_eq!(sel(&s), usize::from(s.list_rows()));
    drive(&mut s, &mut fake, vec![key(Key::End)]);
    assert_eq!(sel(&s), 29);
    drive(&mut s, &mut fake, vec![key(Key::Down), key(Key::PageDown)]);
    assert_eq!(sel(&s), 29, "clamped at the bottom");
    drive(&mut s, &mut fake, vec![key(Key::PageUp), key(Key::Home)]);
    assert_eq!(sel(&s), 0);
}

#[test]
fn journey_editing_the_query_selects_the_best_match() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    drive(&mut s, &mut fake, vec![key(Key::Down)]);
    assert_eq!(s.selected_id(), Some(id(1).as_str()));
    drive(&mut s, &mut fake, typed("git"));
    assert_eq!(s.selected_id(), Some(id(2).as_str()));
}

#[test]
fn journey_enter_waits_for_the_restore_before_quitting() {
    let mut s = PickerState::new(b_and_a(), false, (80, 24));
    assert_eq!(update(&mut s, key(Key::Enter)), vec![Cmd::Restore(id(2))]);
    assert_eq!(s.mode(), &Mode::Restoring);
    assert!(s.status().is_some(), "shows Restoring…");
    // Keys wait while the restore is in flight.
    assert!(update(&mut s, key(Key::Char('x'))).is_empty());
    assert_eq!(s.query(), "");
    assert_eq!(
        update(&mut s, Msg::Done(Action::Restore, Ok(()))),
        vec![Cmd::Quit(Exit::code(EXIT_RESTORED))]
    );
}

#[test]
fn journey_esc_exits_1_and_ctrl_c_exits_130_without_a_restore() {
    for (k, code) in [
        (Key::Esc, EXIT_DISMISSED),
        (Key::Ctrl('c'), EXIT_INTERRUPTED),
    ] {
        let (mut s, mut fake) = open(b_and_a(), (80, 24));
        let mut inputs = typed("reb");
        inputs.push(key(k));
        assert_eq!(drive(&mut s, &mut fake, inputs), Some(Exit::code(code)));
        assert!(fake.calls.is_empty());
    }
}

#[test]
fn journey_ctrl_c_interrupts_a_restore_in_flight() {
    let mut s = PickerState::new(b_and_a(), false, (80, 24));
    update(&mut s, key(Key::Enter));
    assert_eq!(
        update(&mut s, key(Key::Ctrl('c'))),
        vec![Cmd::Quit(Exit::code(EXIT_INTERRUPTED))]
    );
}

#[test]
fn journey_daemon_gone_at_restore_exits_3_with_the_hint() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    fake.next = Some(Failure::DaemonDown);
    let exit = drive(&mut s, &mut fake, vec![key(Key::Enter)]).unwrap();
    assert_eq!(exit.code, EXIT_DAEMON_DOWN);
    assert_eq!(
        exit.message.as_deref(),
        Some(crate::ui::copy::CLI_DAEMON_DOWN)
    );
}

#[test]
fn journey_a_failed_restore_stays_open_with_a_status_line() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    fake.next = Some(Failure::Other("no clipboard access".into()));
    assert_eq!(drive(&mut s, &mut fake, vec![key(Key::Enter)]), None);
    assert_eq!(s.mode(), &Mode::Normal);
    assert_eq!(s.status(), Some("no clipboard access"));
    // The next key clears the status line.
    drive(&mut s, &mut fake, vec![key(Key::Down)]);
    assert_eq!(s.status(), None);
}

#[test]
fn journey_preview_content_is_fetched_for_the_selection() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    drive(&mut s, &mut fake, vec![key(Key::Down)]);
    assert_eq!(fake.shows, vec![id(1)]);
    assert_eq!(s.selected_content(), Some("git rebase -i HEAD~3 (full)"));
    // Cached: coming back does not fetch again.
    drive(&mut s, &mut fake, vec![key(Key::Up), key(Key::Down)]);
    assert_eq!(fake.shows, vec![id(1), id(2)]);
}

#[test]
fn journey_history_changed_relists_and_keeps_the_selection() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    drive(&mut s, &mut fake, vec![key(Key::Down)]);
    let mut entries = b_and_a().entries;
    entries.insert(0, entry(3, "echo synthetic", 0, false));
    fake.snapshot = snapshot(entries);
    drive(&mut s, &mut fake, vec![Msg::HistoryChanged]);
    assert_eq!(fake.lists, 1);
    assert_eq!(s.rows().len(), 3);
    assert_eq!(s.selected_id(), Some(id(1).as_str()));
}

// ---------------------------------------------------------------- snapshot harness

fn color_name(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02X}{g:02X}{b:02X}"),
        Color::Indexed(n) => format!("idx{n}"),
        other => format!("{other:?}"),
    }
}

fn cell_style(buf: &Buffer, x: u16, y: u16) -> String {
    let cell = &buf[(x, y)];
    let mut parts = Vec::new();
    if cell.fg != Color::Reset {
        parts.push(format!("fg={}", color_name(cell.fg)));
    }
    if cell.bg != Color::Reset {
        parts.push(format!("bg={}", color_name(cell.bg)));
    }
    for (m, n) in [
        (Modifier::REVERSED, "REV"),
        (Modifier::BOLD, "BOLD"),
        (Modifier::UNDERLINED, "UL"),
        (Modifier::DIM, "DIM"),
    ] {
        if cell.modifier.contains(m) {
            parts.push(n.to_string());
        }
    }
    parts.join(" ")
}

/// Text of each row, then its non-default style runs (the `src/ui` snapshot format).
pub(crate) fn dump(buf: &Buffer) -> String {
    let area = buf.area;
    let mut out = String::new();
    for y in 0..area.height {
        let mut row = String::new();
        let mut skip = 0usize;
        for x in 0..area.width {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let sym = buf[(x, y)].symbol();
            skip = sym.width().saturating_sub(1);
            row.push_str(sym);
        }
        out.push('|');
        out.push_str(row.trim_end());
        out.push('\n');
        let mut runs = Vec::new();
        let mut x = 0;
        while x < area.width {
            let st = cell_style(buf, x, y);
            let mut end = x + 1;
            while end < area.width && cell_style(buf, end, y) == st {
                end += 1;
            }
            if !st.is_empty() {
                runs.push(format!("{x}..{end} {st}"));
            }
            x = end;
        }
        if !runs.is_empty() {
            out.push_str("  ~ ");
            out.push_str(&runs.join(" ; "));
            out.push('\n');
        }
    }
    out
}

/// How a snapshot is drawn.
#[derive(Clone, Copy)]
pub(crate) struct Look {
    pub tier: Tier,
    pub ascii: bool,
}

pub(crate) const COLOR: Look = Look {
    tier: Tier::Ansi16,
    ascii: false,
};

/// Renders `state` full screen at its own size.
pub(crate) fn screen(state: &PickerState, look: Look) -> String {
    let (w, h) = state.size();
    let theme = Theme::new(look.tier, Background::Dark);
    let glyphs = Glyphs::new(look.ascii);
    let area = Rect::new(0, 0, w, h);
    let mut buf = Buffer::empty(area);
    view::render(state, area, &mut buf, Ctx::new(&theme, &glyphs));
    format!(
        "# {w}x{h} tier={} ascii={}\n{}",
        look.tier.name(),
        look.ascii,
        dump(&buf)
    )
}

pub(crate) const SIZES: [(u16, u16); 2] = [(80, 24), (40, 10)];

/// A history with pinned, long, wide and multi-line entries.
pub(crate) fn rich() -> Snapshot {
    snapshot(vec![
        entry(6, "git rebase -i HEAD~3", 3, false),
        entry(
            5,
            "git reset --soft HEAD~1 && git push --force-with-lease origin main",
            120,
            true,
        ),
        entry(
            4,
            "https://example.org/a/very/long/path/that/keeps/going/on",
            1440,
            false,
        ),
        entry(3, "日本語のテキスト", 2880, false),
        entry(2, "fn main() {⏎    println!(\"synthetic\");⏎}", 4320, false),
        entry(1, "cargo test -p rstory --test daemon_restore", 0, true),
    ])
}

/// A state over `snap` at `size`, with the selection's content loaded as the runtime would.
pub(crate) fn loaded(
    snap: Snapshot,
    read_only: bool,
    size: (u16, u16),
    msgs: Vec<Msg>,
) -> PickerState {
    let mut fake = Fake::new(snap.clone());
    fake.read_only = read_only;
    let mut s = PickerState::new(snap, read_only, size);
    // Feed the content for the first selection, then the scripted messages.
    let mut all = vec![Msg::Resize {
        width: size.0,
        height: size.1,
    }];
    all.extend(msgs);
    drive(&mut s, &mut fake, all);
    s
}

// ---------------------------------------------------------------- US1 snapshots (T004)

#[test]
fn snapshot_normal() {
    for size in SIZES {
        let s = loaded(rich(), false, size, vec![]);
        insta::assert_snapshot!(format!("normal_{}x{}", size.0, size.1), screen(&s, COLOR));
    }
}

#[test]
fn snapshot_filtered() {
    for size in SIZES {
        let s = loaded(rich(), false, size, typed("git"));
        insta::assert_snapshot!(format!("filtered_{}x{}", size.0, size.1), screen(&s, COLOR));
    }
}

#[test]
fn snapshot_preview_toggled() {
    // Tab flips the default: off at 80 columns, on at 40.
    for size in SIZES {
        let s = loaded(rich(), false, size, vec![key(Key::Tab)]);
        insta::assert_snapshot!(
            format!("preview_toggled_{}x{}", size.0, size.1),
            screen(&s, COLOR)
        );
    }
}

#[test]
fn snapshot_a_120_by_40_screen_splits_the_preview_in_half() {
    let s = loaded(rich(), false, (120, 40), vec![key(Key::Down)]);
    insta::assert_snapshot!("normal_120x40", screen(&s, COLOR));
}

#[test]
fn the_selected_row_has_the_marker_and_reverse_video() {
    let s = loaded(b_and_a(), false, (80, 24), vec![]);
    let out = screen(&s, COLOR);
    let line = out
        .lines()
        .find(|l| l.contains("git reset"))
        .expect("row for B");
    assert!(line.starts_with("| >"), "{line}");
    assert!(out.contains("REV"), "{out}");
    let _ = NOW;
}

// ---------------------------------------------------------------- US2 journeys (T006)

#[test]
fn pin_toggle_pins_then_unpins_and_relists() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    assert_eq!(
        update(&mut s.clone(), key(Key::Ctrl('t')))[0],
        Cmd::Pin(id(2))
    );
    drive(&mut s, &mut fake, vec![key(Key::Ctrl('t'))]);
    assert_eq!(fake.calls, vec![(Action::Pin, id(2))]);
    assert_eq!(fake.lists, 1, "relisted after the pin");
    assert!(s.selected_entry().unwrap().pinned);
    assert_eq!(s.selected_id(), Some(id(2).as_str()), "selection kept");
    drive(&mut s, &mut fake, vec![key(Key::Ctrl('t'))]);
    assert_eq!(fake.calls[1], (Action::Unpin, id(2)));
    assert!(!s.selected_entry().unwrap().pinned);
}

#[test]
fn pin_limit_shows_the_status_line_and_keeps_the_picker_open() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    fake.next = Some(Failure::PinLimit);
    assert_eq!(drive(&mut s, &mut fake, vec![key(Key::Ctrl('t'))]), None);
    assert_eq!(s.status(), Some(crate::ui::copy::PICKER_PIN_LIMIT));
    assert!(!s.selected_entry().unwrap().pinned);
    let out = screen(&s, COLOR);
    assert!(out.contains("Pin limit reached (10)"), "{out}");
}

#[test]
fn delete_asks_and_y_removes() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    assert!(drive(&mut s, &mut fake, vec![key(Key::Ctrl('d'))]).is_none());
    assert_eq!(s.mode(), &Mode::ConfirmDelete(id(2)));
    assert!(fake.calls.is_empty(), "nothing removed before the answer");
    // Typing does not edit the query while asking.
    drive(&mut s, &mut fake, typed("x"));
    assert_eq!(s.query(), "");
    assert_eq!(s.mode(), &Mode::ConfirmDelete(id(2)));
    drive(&mut s, &mut fake, typed("y"));
    assert_eq!(fake.calls, vec![(Action::Remove, id(2))]);
    assert_eq!(s.mode(), &Mode::Normal);
    assert_eq!(s.rows().len(), 1, "relisted after the removal");
    assert_eq!(
        s.selected_id(),
        Some(id(1).as_str()),
        "the next entry takes its place"
    );
}

#[test]
fn delete_n_or_esc_cancels_without_quitting() {
    for cancel in [Key::Char('n'), Key::Esc] {
        let (mut s, mut fake) = open(b_and_a(), (80, 24));
        let exit = drive(&mut s, &mut fake, vec![key(Key::Ctrl('d')), key(cancel)]);
        assert_eq!(exit, None, "{cancel:?}");
        assert_eq!(s.mode(), &Mode::Normal);
        assert!(fake.calls.is_empty());
        assert_eq!(s.rows().len(), 2);
    }
}

#[test]
fn ctrl_c_still_interrupts_while_asking() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    let exit = drive(
        &mut s,
        &mut fake,
        vec![key(Key::Ctrl('d')), key(Key::Ctrl('c'))],
    );
    assert_eq!(exit, Some(Exit::code(EXIT_INTERRUPTED)));
    assert!(fake.calls.is_empty());
}

#[test]
fn pin_and_delete_need_a_selection() {
    let (mut s, mut fake) = open(b_and_a(), (80, 24));
    drive(&mut s, &mut fake, typed("zzzz"));
    drive(
        &mut s,
        &mut fake,
        vec![key(Key::Ctrl('t')), key(Key::Ctrl('d'))],
    );
    assert_eq!(s.mode(), &Mode::Normal);
    assert!(fake.calls.is_empty());
}

#[test]
fn snapshot_confirm_delete() {
    for size in SIZES {
        let s = loaded(rich(), false, size, vec![key(Key::Ctrl('d'))]);
        insta::assert_snapshot!(format!("confirm_{}x{}", size.0, size.1), screen(&s, COLOR));
    }
}

#[test]
fn snapshot_pinned_after_ctrl_t() {
    let s = loaded(rich(), false, (80, 24), vec![key(Key::Ctrl('t'))]);
    let out = screen(&s, COLOR);
    let line = out.lines().find(|l| l.contains("git rebase")).unwrap();
    assert!(line.contains('📌'), "{line}");
}

// ---------------------------------------------------------------- US3 (T008)

pub(crate) const MONO: Look = Look {
    tier: Tier::Mono,
    ascii: false,
};

pub(crate) const ASCII_MONO: Look = Look {
    tier: Tier::Mono,
    ascii: true,
};

/// Renders `state` with the `--plain` renderer at its own size.
pub(crate) fn plain(state: &PickerState) -> String {
    let (w, h) = state.size();
    let theme = Theme::new(Tier::Mono, Background::Dark);
    let glyphs = Glyphs::new(true);
    let area = Rect::new(0, 0, w, view::plain_height(h));
    let mut buf = Buffer::empty(area);
    view::render_plain(state, area, &mut buf, Ctx::new(&theme, &glyphs));
    format!("# plain {w}x{h}\n{}", dump(&buf))
}

#[test]
fn snapshot_plain() {
    for size in SIZES {
        let s = loaded(rich(), false, size, vec![]);
        insta::assert_snapshot!(format!("plain_{}x{}", size.0, size.1), plain(&s));
        let s = loaded(rich(), false, size, typed("git"));
        insta::assert_snapshot!(format!("plain_filtered_{}x{}", size.0, size.1), plain(&s));
    }
}

#[test]
fn plain_marks_selection_and_pins_in_words_without_box_drawing() {
    let s = loaded(rich(), false, (80, 24), vec![key(Key::Down)]);
    let out = plain(&s);
    let lines: Vec<&str> = out.lines().filter(|l| l.starts_with('|')).collect();
    assert_eq!(lines[0], "|query: _");
    assert!(lines[1].starts_with("|  [0] 3m  git rebase"), "{}", lines[1]);
    assert!(lines[2].starts_with("|> [1] 2h  [pinned] git reset"), "{}", lines[2]);
    assert!(out.contains("6 of 6 shown. Enter restores, Esc quits."), "{out}");
    for bad in ['│', '─', '┬', '📌', '…'] {
        assert!(!out.contains(bad), "{bad} in {out}");
    }
}

#[test]
fn plain_shows_the_confirm_and_status_lines() {
    let s = loaded(rich(), false, (80, 24), vec![key(Key::Ctrl('d'))]);
    assert!(plain(&s).contains("Delete this entry? y/n"));
    let s = loaded(rich(), true, (80, 24), vec![]);
    assert!(plain(&s).contains("Read-only: daemon not running"));
}

#[test]
fn snapshot_no_color() {
    for size in SIZES {
        let s = loaded(rich(), false, size, typed("git"));
        let out = screen(&s, MONO);
        assert!(!out.contains("fg="), "NO_COLOR draws no colors:\n{out}");
        insta::assert_snapshot!(format!("no_color_{}x{}", size.0, size.1), out);
    }
}

#[test]
fn snapshot_ascii() {
    for size in SIZES {
        let s = loaded(rich(), false, size, vec![]);
        let out = screen(&s, ASCII_MONO);
        assert!(out.lines().all(|l| l.is_ascii()), "{out}");
        insta::assert_snapshot!(format!("ascii_{}x{}", size.0, size.1), out);
    }
}

#[test]
fn snapshot_read_only() {
    for size in SIZES {
        let mut snap = rich();
        snap.rec = None;
        let s = loaded(snap, true, size, vec![]);
        let out = screen(&s, COLOR);
        assert!(out.contains("Read-only: daemon not running"), "{out}");
        assert!(!out.contains("Ctrl-T"), "read-only hints drop Ctrl-T/Ctrl-D:\n{out}");
        insta::assert_snapshot!(format!("read_only_{}x{}", size.0, size.1), out);
    }
}

#[test]
fn snapshot_empty() {
    for size in SIZES {
        let s = loaded(snapshot(vec![]), false, size, vec![]);
        let out = screen(&s, COLOR);
        assert!(out.contains("0/0"), "{out}");
        insta::assert_snapshot!(format!("empty_{}x{}", size.0, size.1), out);
        assert!(plain(&s).contains("No entries yet"));
    }
}

#[test]
fn snapshot_too_small() {
    for size in [(19, 10), (40, 4)] {
        let s = loaded(rich(), false, size, vec![]);
        let out = screen(&s, COLOR);
        assert!(out.contains("Terminal too small (need 20×5)."), "{out}");
        insta::assert_snapshot!(format!("too_small_{}x{}", size.0, size.1), out);
    }
}

#[test]
fn resizing_never_panics() {
    let mut s = loaded(rich(), false, (80, 24), vec![key(Key::Down)]);
    for w in [0u16, 1, 5, 19, 20, 21, 39, 40, 59, 60, 79, 80, 99, 100, 200] {
        for h in [0u16, 1, 4, 5, 6, 7, 8, 24, 60] {
            update(&mut s, Msg::Resize { width: w, height: h });
            let _ = screen(&s, COLOR);
            let _ = plain(&s);
        }
    }
}

#[test]
fn read_only_enter_shows_the_hint_and_exits_3_without_a_restore() {
    let mut snap = b_and_a();
    snap.rec = None;
    let mut fake = Fake::new(snap.clone());
    fake.read_only = true;
    let mut s = PickerState::new(snap, true, (80, 24));
    let exit = drive(&mut s, &mut fake, vec![key(Key::Enter)]).unwrap();
    assert_eq!(exit.code, EXIT_DAEMON_DOWN);
    assert_eq!(exit.message.as_deref(), Some(crate::ui::copy::PICKER_READONLY_RESTORE));
    assert!(fake.calls.is_empty(), "the clipboard is not touched");
}

#[test]
fn read_only_ignores_pin_and_delete() {
    let mut snap = b_and_a();
    snap.rec = None;
    let mut fake = Fake::new(snap.clone());
    fake.read_only = true;
    let mut s = PickerState::new(snap, true, (80, 24));
    drive(&mut s, &mut fake, vec![key(Key::Ctrl('t')), key(Key::Ctrl('d'))]);
    assert_eq!(s.mode(), &Mode::Normal);
    assert!(fake.calls.is_empty());
}
