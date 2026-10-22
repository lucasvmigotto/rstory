use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::Ctx;

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
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}
