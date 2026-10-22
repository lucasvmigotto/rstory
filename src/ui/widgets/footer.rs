use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::{Ctx, draw_line};
use crate::ui::{copy, theme::Role};

pub enum FooterKind<'a> {
    /// Key hints; `short` below 40 columns or 8 rows.
    Hints { short: bool },
    /// A status message (pin limit, restoring).
    Status(&'a str),
    /// A confirm question.
    Confirm(&'a str),
}

pub struct Footer<'a> {
    pub kind: FooterKind<'a>,
    pub ctx: Ctx<'a>,
}

impl Widget for Footer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (text, role) = match self.kind {
            FooterKind::Hints { short: false } => (copy::PICKER_HINTS_FULL, Role::Muted),
            FooterKind::Hints { short: true } => (copy::PICKER_HINTS_SHORT, Role::Muted),
            FooterKind::Status(t) => (t, Role::Warning),
            FooterKind::Confirm(t) => (t, Role::Danger),
        };
        draw_line(buf, area, text, self.ctx.style(role), self.ctx);
    }
}
