use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::{Ctx, RecState};

/// `> query_` on the left, `{shown}/{total}` and the state badge on the right.
pub struct QueryLine<'a> {
    pub query: &'a str,
    pub shown: usize,
    pub total: usize,
    pub state: &'a RecState,
    pub ctx: Ctx<'a>,
}

impl Widget for QueryLine<'_> {
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}
