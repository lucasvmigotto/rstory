use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::Ctx;

/// Share of the width the preview takes, `None` when it is hidden.
pub fn preview_percent(_width: u16) -> Option<u16> {
    todo!()
}

/// The full content of the selected entry, wrapped, with a size line.
pub struct PreviewPane<'a> {
    /// `None` when nothing is selected.
    pub content: Option<&'a str>,
    pub ctx: Ctx<'a>,
}

impl Widget for PreviewPane<'_> {
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}
