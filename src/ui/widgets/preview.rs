use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::Ctx;
use crate::ui::{glyphs::Glyph, text::wrap, theme::Role};

/// Content shown before the "N more bytes" line.
const SHOWN_BYTES: usize = 64 * 1024;

/// Share of the width the preview takes, `None` when it is hidden.
pub fn preview_percent(width: u16) -> Option<u16> {
    match width {
        100.. => Some(50),
        60..=99 => Some(40),
        _ => None,
    }
}

/// The full content of the selected entry, wrapped, with a size line.
pub struct PreviewPane<'a> {
    /// `None` when nothing is selected.
    pub content: Option<&'a str>,
    pub ctx: Ctx<'a>,
}

fn human_size(n: usize) -> String {
    const KIB: f64 = 1024.0;
    let f = n as f64;
    if n < 1024 {
        format!("{n} B")
    } else if f < KIB * KIB {
        format!("{:.1} KiB", f / KIB)
    } else {
        format!("{:.1} MiB", f / (KIB * KIB))
    }
}

impl Widget for PreviewPane<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let ctx = self.ctx;
        let muted = ctx.style(Role::Muted);
        for y in area.y..area.y + area.height {
            buf.set_string(area.x, y, ctx.glyphs.get(Glyph::VBar), muted);
        }
        let Some(content) = self.content else { return };
        if area.width < 3 {
            return;
        }
        let width = usize::from(area.width) - 2;
        let mut end = content.len().min(SHOWN_BYTES);
        while !content.is_char_boundary(end) {
            end -= 1;
        }
        let mut lines = wrap(&content[..end], width, ctx.glyphs);
        if end < content.len() {
            let more = format!(
                "{} {} more bytes",
                ctx.glyphs.get(Glyph::Ellipsis),
                content.len() - end
            );
            lines.extend(wrap(&more, width, ctx.glyphs));
        }
        let text_rows = usize::from(area.height - 1);
        for (i, line) in lines.iter().take(text_rows).enumerate() {
            buf.set_string(
                area.x + 2,
                area.y + i as u16,
                line,
                ratatui::style::Style::new(),
            );
        }
        let size = crate::ui::text::one_line(&human_size(content.len()), width, ctx.glyphs);
        buf.set_string(area.x + 2, area.y + area.height - 1, size, muted);
    }
}
