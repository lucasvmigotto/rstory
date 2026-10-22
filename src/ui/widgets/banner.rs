use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::Ctx;

/// The read-only notice above the list.
pub struct Banner<'a> {
    pub ctx: Ctx<'a>,
}

impl Widget for Banner<'_> {
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}
