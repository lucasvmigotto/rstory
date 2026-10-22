use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::Ctx;

/// The recording state; the word is always shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecState {
    Recording,
    Paused,
    PausedUntil { secs: u64 },
    Inactive { reason: String },
}

impl RecState {
    /// The full word, e.g. `Paused — resumes in 1:05`.
    pub fn label(&self) -> String {
        todo!()
    }

    /// The narrow form used below 60 columns.
    pub fn short_label(&self) -> String {
        todo!()
    }
}

pub struct StateBadge<'a> {
    pub state: &'a RecState,
    pub short: bool,
    pub ctx: Ctx<'a>,
}

impl Widget for StateBadge<'_> {
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}
