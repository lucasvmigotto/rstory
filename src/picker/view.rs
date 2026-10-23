//! Layout and rendering of a [`PickerState`] (tui.md "Screens"), built from the design system
//! components: `QueryLine`, a list of `EntryRow`, `PreviewPane`, `Banner` and `Footer`.
//!
//! Full screen, top to bottom: the query line, the read-only banner (when read-only), a rule,
//! the list with the preview on the right, a rule, the footer (two lines with F1's keymap).
//! Nothing here reads a clock or the terminal: the state carries the size and "now".

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use time::OffsetDateTime;

use crate::ui::copy;
use crate::ui::glyphs::Glyph;
use crate::ui::text::{display_width, one_line};
use crate::ui::theme::Role;
use crate::ui::widgets::{
    Banner, Ctx, EntryRow, Footer, FooterKind, PreviewPane, QueryLine, preview_percent,
};

use super::state::{Mode, PickerState, Preview};

/// A short age: `59s`, `3m`, `2h`, `5d` (the `ls` format).
#[must_use]
pub fn age(now: OffsetDateTime, then: OffsetDateTime) -> String {
    let s = u64::try_from((now - then).whole_seconds()).unwrap_or(0);
    match s {
        0..=59 => format!("{s}s"),
        60..=3599 => format!("{}m", s / 60),
        3600..=86_399 => format!("{}h", s / 3600),
        _ => format!("{}d", s / 86_400),
    }
}

fn row(area: Rect, y: u16) -> Rect {
    Rect::new(area.x, area.y + y, area.width, 1)
}

/// One line of copy text with a one-cell gutter, ASCII fallbacks applied.
fn text_line(buf: &mut Buffer, area: Rect, text: &str, role: Option<Role>, ctx: Ctx<'_>) {
    if area.width < 2 || area.height == 0 {
        return;
    }
    let shown = ctx.glyphs.text(text);
    let out = one_line(&shown, usize::from(area.width) - 1, ctx.glyphs);
    let style = role.map(|r| ctx.style(r)).unwrap_or_default();
    buf.set_string(area.x + 1, area.y, out, style);
}

fn rule(buf: &mut Buffer, area: Rect, ctx: Ctx<'_>) {
    let g = ctx.glyphs.get(Glyph::HRule);
    let n = usize::from(area.width) / display_width(g).max(1);
    buf.set_string(area.x, area.y, g.repeat(n), ctx.style(Role::Muted));
}

/// The query line without a state badge (the state is unknown while read-only).
fn query_without_badge(state: &PickerState, area: Rect, buf: &mut Buffer, ctx: Ctx<'_>) {
    let count = copy::format(
        copy::PICKER_COUNT,
        &[
            ("shown", &state.rows().len().to_string()),
            ("total", &state.entries().len().to_string()),
        ],
    );
    let w = usize::from(area.width);
    let cw = display_width(&count);
    let right = w >= cw + 12;
    let room = w.saturating_sub(4 + if right { cw + 2 } else { 0 });
    // The tail of the query that fits, then the cursor.
    let q = state.query();
    let mut tail = q;
    while display_width(tail) > room {
        let mut chars = tail.chars();
        chars.next();
        tail = chars.as_str();
    }
    let shown = one_line(tail, room, ctx.glyphs);
    buf.set_string(
        area.x + 1,
        area.y,
        ctx.glyphs.get(Glyph::Select),
        ctx.style(Role::Accent),
    );
    buf.set_string(area.x + 3, area.y, &shown, ratatui::style::Style::new());
    let cx = area.x + 3 + u16::try_from(display_width(&shown)).unwrap_or(0);
    buf.set_string(cx, area.y, "_", ctx.style(Role::Accent));
    if right {
        let rx = area.x + u16::try_from(w - 1 - cw).unwrap_or(0);
        buf.set_string(rx, area.y, &count, ctx.style(Role::Muted));
    }
}

/// Draws `state` full screen into `area`.
pub fn render(state: &PickerState, area: Rect, buf: &mut Buffer, ctx: Ctx<'_>) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    if state.too_small() || area.width < 20 || area.height < 5 {
        text_line(buf, row(area, 0), copy::PICKER_TOO_SMALL, None, ctx);
        return;
    }
    let mut y = 0;
    match state.rec() {
        Some(rec) => QueryLine {
            query: state.query(),
            shown: state.rows().len(),
            total: state.entries().len(),
            state: rec,
            ctx,
        }
        .render(row(area, y), buf),
        None => query_without_badge(state, row(area, y), buf, ctx),
    }
    y += 1;
    if state.read_only() {
        Banner { ctx }.render(row(area, y), buf);
        y += 1;
    }
    rule(buf, row(area, y), ctx);
    y += 1;

    let footer_rows = if state.keymap() { 2 } else { 1 };
    let body_h = area.height.saturating_sub(y + 1 + footer_rows);
    let body = Rect::new(area.x, area.y + y, area.width, body_h);
    list_and_preview(state, body, buf, ctx);
    y += body_h;
    rule(buf, row(area, y), ctx);
    y += 1;
    footer(
        state,
        Rect::new(area.x, area.y + y, area.width, footer_rows),
        buf,
        ctx,
    );
}

fn list_and_preview(state: &PickerState, body: Rect, buf: &mut Buffer, ctx: Ctx<'_>) {
    if body.height == 0 {
        return;
    }
    let preview_w = match state.preview() {
        Preview::Visible => {
            let pct = preview_percent(body.width).unwrap_or(100);
            body.width * pct / 100
        }
        Preview::Hidden => 0,
    };
    let list = Rect::new(body.x, body.y, body.width - preview_w, body.height);
    if state.entries().is_empty() {
        text_line(buf, row(list, 0), copy::PICKER_EMPTY, None, ctx);
    } else if list.width > 0 {
        let h = usize::from(list.height);
        let sel = state.selected_pos().unwrap_or(0);
        let offset = sel.saturating_sub(h.saturating_sub(1));
        for (i, r) in state.rows().iter().skip(offset).take(h).enumerate() {
            let e = &state.entries()[r.idx];
            let a = age(state.now(), e.last_copied);
            EntryRow {
                age: &a,
                pinned: e.pinned,
                selected: offset + i == sel && state.selected_id().is_some(),
                preview: &e.preview,
                matches: &r.positions,
                ctx,
            }
            .render(row(list, u16::try_from(i).unwrap_or(0)), buf);
        }
    }
    if preview_w > 0 {
        let pane = Rect::new(body.x + list.width, body.y, preview_w, body.height);
        // Until the content arrives, the escaped one-line preview stands in.
        let content = state
            .selected_content()
            .or_else(|| state.selected_entry().map(|e| e.preview.as_str()));
        PreviewPane { content, ctx }.render(pane, buf);
    }
}

fn footer(state: &PickerState, area: Rect, buf: &mut Buffer, ctx: Ctx<'_>) {
    let (w, h) = state.size();
    let kind = match (state.mode(), state.status()) {
        (Mode::ConfirmDelete(_), _) => FooterKind::Confirm(copy::PICKER_CONFIRM_DELETE),
        (_, Some(status)) => FooterKind::Status(status),
        // Short hints when the full line does not fit, on short screens, and read-only
        // (Ctrl-T and Ctrl-D need the daemon).
        _ => FooterKind::Hints {
            short: usize::from(w) < display_width(copy::PICKER_HINTS_FULL) + 2
                || h < 8
                || state.read_only(),
        },
    };
    Footer { kind, ctx }.render(row(area, 0), buf);
}

/// Rows the inline (`--plain`) viewport takes on a terminal `height` rows tall.
#[must_use]
pub fn plain_height(height: u16) -> u16 {
    height.min(12)
}

/// Draws `state` inline for `--plain`.
pub fn render_plain(state: &PickerState, area: Rect, buf: &mut Buffer, ctx: Ctx<'_>) {
    render(state, area, buf, ctx);
}
