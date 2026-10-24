//! The components of `specs/000-design-system/components.md`.

use ratatui::style::Style;

use super::glyphs::Glyphs;
use super::theme::{Role, Theme};

mod banner;
mod check_line;
mod entry_line;
mod entry_row;
mod footer;
mod preview;
mod query_line;
mod state_badge;

pub use banner::Banner;
pub use check_line::{CheckLine, CheckStatus};
pub use entry_line::EntryLine;
pub use entry_row::EntryRow;
pub use footer::{Footer, FooterKind};
pub use preview::{PreviewPane, preview_percent};
pub use query_line::QueryLine;
pub use state_badge::{RecState, StateBadge};

/// What every component needs to draw itself.
#[derive(Clone, Copy)]
pub struct Ctx<'a> {
    pub theme: &'a Theme,
    pub glyphs: &'a Glyphs,
}

impl<'a> Ctx<'a> {
    pub fn new(theme: &'a Theme, glyphs: &'a Glyphs) -> Self {
        Self { theme, glyphs }
    }

    pub fn style(&self, role: Role) -> Style {
        self.theme.style(role)
    }
}

use ratatui::buffer::Buffer;

use super::text::Seg;

/// Draws segments from `(x, y)`, marking matched ones; returns the next free column.
fn draw_segs(buf: &mut Buffer, mut x: u16, y: u16, segs: &[Seg], base: Style, mark: Style) -> u16 {
    for seg in segs {
        let style = if seg.matched { base.patch(mark) } else { base };
        buf.set_string(x, y, &seg.text, style);
        x += seg.width as u16;
    }
    x
}

/// One-line text with a one-cell gutter, escaped and truncated to the area.
fn draw_line(
    buf: &mut Buffer,
    area: ratatui::layout::Rect,
    text: &str,
    style: Style,
    ctx: Ctx<'_>,
) {
    if area.width < 2 || area.height == 0 {
        return;
    }
    let shown = ctx.glyphs.text(text);
    let out = super::text::one_line(&shown, usize::from(area.width) - 1, ctx.glyphs);
    buf.set_string(area.x + 1, area.y, out, style);
}
