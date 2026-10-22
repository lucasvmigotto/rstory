use ratatui::text::Line;

use super::Ctx;

/// One line of `rstory ls`.
pub struct EntryLine<'a> {
    pub index: usize,
    pub short_id: &'a str,
    pub pinned: bool,
    pub age: &'a str,
    pub preview: &'a str,
}

impl EntryLine<'_> {
    /// A width-aware, styled line for a terminal.
    pub fn tty(&self, _width: usize, _ctx: Ctx<'_>) -> Line<'static> {
        todo!()
    }

    /// The piped form: tab-separated, one line, preview at most 256 characters, no styling.
    pub fn tsv(&self, _ctx: Ctx<'_>) -> String {
        todo!()
    }
}
