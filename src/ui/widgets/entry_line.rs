use ratatui::text::{Line, Span};

use super::Ctx;
use crate::ui::{
    glyphs::Glyph,
    text::{display_width, one_line},
    theme::Role,
};

/// One line of `rstory ls`.
pub struct EntryLine<'a> {
    pub index: usize,
    pub short_id: &'a str,
    pub pinned: bool,
    pub age: &'a str,
    pub preview: &'a str,
}

/// Longest preview in the piped form.
const PIPED_PREVIEW: usize = 256;

impl EntryLine<'_> {
    /// A width-aware, styled line for a terminal.
    pub fn tty(&self, width: usize, ctx: Ctx<'_>) -> Line<'static> {
        let idx = format!("{:<5}", format!("[{}]", self.index));
        let id = format!("{:<6}", self.short_id);
        let pw = ctx.glyphs.width(Glyph::Pin);
        let pin = if self.pinned {
            ctx.glyphs.get(Glyph::Pin).to_string()
        } else {
            " ".repeat(pw)
        };
        let age: String = self.age.chars().take(4).collect();
        let age = format!("{age:>4}");
        let used = display_width(&idx) + 1 + display_width(&id) + 1 + pw + 1 + 4 + 2;
        let preview = one_line(self.preview, width.saturating_sub(used), ctx.glyphs);
        Line::from(vec![
            Span::raw(idx),
            Span::raw(" "),
            Span::styled(id, ctx.style(Role::Muted)),
            Span::raw(" "),
            Span::styled(pin, ctx.style(Role::Pinned)),
            Span::raw(" "),
            Span::styled(age, ctx.style(Role::Muted)),
            Span::raw("  "),
            Span::raw(preview),
        ])
    }

    /// The piped form: tab-separated, one line, preview at most 256 characters, no styling.
    pub fn tsv(&self, ctx: Ctx<'_>) -> String {
        [
            self.index.to_string(),
            self.short_id.to_string(),
            if self.pinned { "pinned" } else { "" }.to_string(),
            self.age.to_string(),
            one_line(self.preview, PIPED_PREVIEW, ctx.glyphs),
        ]
        .join("\t")
    }
}
