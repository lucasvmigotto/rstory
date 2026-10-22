use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::{Ctx, RecState, StateBadge, draw_segs};
use crate::ui::{
    copy,
    glyphs::Glyph,
    text::{display_width, segments},
    theme::Role,
};

/// `> query_` on the left, `{shown}/{total}` and the state badge on the right.
pub struct QueryLine<'a> {
    pub query: &'a str,
    pub shown: usize,
    pub total: usize,
    pub state: &'a RecState,
    pub ctx: Ctx<'a>,
}

/// Cells kept for the query before the right side may take the rest.
const MIN_QUERY: usize = 8;

impl Widget for QueryLine<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width < 4 {
            return;
        }
        let ctx = self.ctx;
        let w = usize::from(area.width);
        let y = area.y;
        let prefix_w = 3; // gutter, '>', space
        let count = copy::format(
            copy::PICKER_COUNT,
            &[
                ("shown", &self.shown.to_string()),
                ("total", &self.total.to_string()),
            ],
        );
        let cw = display_width(&count);
        let full = StateBadge {
            state: self.state,
            short: false,
            ctx,
        };
        let short = StateBadge {
            state: self.state,
            short: true,
            ctx,
        };
        let fits = |badge_w: usize| prefix_w + MIN_QUERY + 1 + cw + badge_w < w;
        let (badge, gap) = [(&full, 2), (&short, 2)]
            .into_iter()
            .map(|(b, g)| (b, g, display_width(&b.shown())))
            .find(|&(_, g, bw)| fits(g + bw))
            .map_or((None, 0), |(b, g, _)| (Some(b), g));
        let badge_w = badge.map_or(0, |b| display_width(&b.shown()));
        let right_w = if badge.is_some() {
            cw + gap + badge_w
        } else {
            cw
        };
        let right_fits = prefix_w + 2 + right_w < w;

        // Left side: the tail of the query that fits, then the cursor.
        let right_reserved = if right_fits { right_w + 2 } else { 1 };
        let avail = w.saturating_sub(prefix_w + 1 + right_reserved);
        let segs = segments(self.query, ctx.glyphs, &[]);
        let mut start = segs.len();
        let mut used = 0;
        while start > 0 && used + segs[start - 1].width <= avail {
            start -= 1;
            used += segs[start].width;
        }
        buf.set_string(
            area.x + 1,
            y,
            ctx.glyphs.get(Glyph::Select),
            ctx.style(Role::Accent),
        );
        let x = draw_segs(
            buf,
            area.x + prefix_w as u16,
            y,
            &segs[start..],
            Default::default(),
            Default::default(),
        );
        buf.set_string(x, y, "_", ctx.style(Role::Accent));

        if right_fits {
            let rx = area.x + (w - 1 - right_w) as u16;
            buf.set_string(rx, y, &count, ctx.style(Role::Muted));
            if let Some(b) = badge {
                let bx = rx + (cw + gap) as u16;
                let badge_area = Rect::new(bx, y, badge_w as u16, 1);
                StateBadge {
                    state: b.state,
                    short: b.short,
                    ctx,
                }
                .render(badge_area, buf);
            }
        }
    }
}
