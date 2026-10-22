use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::Ctx;

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
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}
