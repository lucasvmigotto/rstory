use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use super::{Ctx, draw_segs};
use crate::ui::{
    glyphs::Glyph,
    text::{segments, truncate},
    theme::Role,
};

/// One list row: marker, age, pin, preview with match marks.
pub struct EntryRow<'a> {
    pub age: &'a str,
    pub pinned: bool,
    pub selected: bool,
    pub preview: &'a str,
    /// Grapheme indexes into `preview` to mark as matched.
    pub matches: &'a [usize],
    pub ctx: Ctx<'a>,
}

impl Widget for EntryRow<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let ctx = self.ctx;
        let y = area.y;
        let base = if self.selected {
            ctx.style(Role::Selected)
        } else {
            Style::new()
        };
        buf.set_style(Rect::new(area.x, y, area.width, 1), base);
        let end = area.x + area.width;
        let put = |buf: &mut Buffer, x: u16, s: &str, style: Style| {
            if x < end {
                buf.set_stringn(x, y, s, usize::from(end - x), style);
            }
        };
        let marker = if self.selected {
            ctx.glyphs.get(Glyph::Select)
        } else {
            " "
        };
        put(buf, area.x + 1, marker, base);
        let age: String = self.age.chars().take(4).collect();
        put(
            buf,
            area.x + 3,
            &format!("{age:>4}"),
            ctx.style(Role::Muted).patch(base),
        );
        let pw = ctx.glyphs.width(Glyph::Pin);
        if self.pinned {
            put(
                buf,
                area.x + 8,
                ctx.glyphs.get(Glyph::Pin),
                ctx.style(Role::Pinned).patch(base),
            );
        }
        let px = area.x + 8 + pw as u16 + 1;
        if px < end {
            let room = usize::from(end - px);
            let segs = truncate(
                segments(self.preview, ctx.glyphs, self.matches),
                room,
                ctx.glyphs,
            );
            let mark = ctx
                .style(Role::Accent)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
            draw_segs(buf, px, y, &segs, base, mark);
        }
    }
}
