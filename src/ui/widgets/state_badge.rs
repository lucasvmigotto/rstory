use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use super::Ctx;
use crate::ui::{copy, text::one_line, theme::Role};

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
        match self {
            RecState::Recording => copy::STATE_RECORDING.to_string(),
            RecState::Paused => copy::STATE_PAUSED.to_string(),
            RecState::PausedUntil { secs } => copy::format(
                copy::STATE_PAUSED_UNTIL,
                &[
                    ("m", &(secs / 60).to_string()),
                    ("ss", &format!("{:02}", secs % 60)),
                ],
            ),
            RecState::Inactive { reason } => {
                copy::format(copy::STATE_INACTIVE, &[("reason", reason)])
            }
        }
    }

    /// The narrow form used below 60 columns.
    pub fn short_label(&self) -> String {
        match self {
            RecState::Recording => "Rec".to_string(),
            RecState::Paused | RecState::PausedUntil { .. } => copy::STATE_PAUSED.to_string(),
            RecState::Inactive { .. } => "Inactive".to_string(),
        }
    }

    fn style(&self, ctx: Ctx<'_>) -> Style {
        match self {
            RecState::Recording => Style::new(),
            RecState::Paused | RecState::PausedUntil { .. } => ctx.style(Role::Warning),
            RecState::Inactive { .. } => ctx.style(Role::Danger),
        }
    }
}

pub struct StateBadge<'a> {
    pub state: &'a RecState,
    pub short: bool,
    pub ctx: Ctx<'a>,
}

impl StateBadge<'_> {
    /// The text as drawn, after the ASCII fallback.
    pub(super) fn shown(&self) -> String {
        let label = if self.short {
            self.state.short_label()
        } else {
            self.state.label()
        };
        self.ctx.glyphs.text(&label).into_owned()
    }
}

impl Widget for StateBadge<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let text = one_line(&self.shown(), usize::from(area.width), self.ctx.glyphs);
        buf.set_string(area.x, area.y, text, self.state.style(self.ctx));
    }
}
