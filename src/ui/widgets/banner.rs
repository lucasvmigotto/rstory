use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::{Ctx, draw_line};
use crate::ui::{copy, theme::Role};

/// The read-only notice above the list.
pub struct Banner<'a> {
    pub ctx: Ctx<'a>,
}

impl Widget for Banner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        draw_line(
            buf,
            area,
            copy::PICKER_READONLY_BANNER,
            self.ctx.style(Role::Warning),
            self.ctx,
        );
    }
}
