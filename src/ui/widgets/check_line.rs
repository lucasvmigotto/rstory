use ratatui::text::{Line, Span};

use super::Ctx;
use crate::ui::{glyphs::Glyph, text::display_width, theme::Role};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckStatus {
    Ok,
    Info,
    Warn,
    Fail,
}

impl CheckStatus {
    const ALL: [CheckStatus; 4] = [Self::Ok, Self::Info, Self::Warn, Self::Fail];

    fn glyph(self) -> Glyph {
        match self {
            Self::Ok => Glyph::Ok,
            Self::Info => Glyph::Info,
            Self::Warn => Glyph::Warn,
            Self::Fail => Glyph::Fail,
        }
    }

    fn role(self) -> Option<Role> {
        match self {
            Self::Ok => None,
            Self::Info => Some(Role::Muted),
            Self::Warn => Some(Role::Warning),
            Self::Fail => Some(Role::Danger),
        }
    }
}

/// One doctor check result.
pub struct CheckLine<'a> {
    pub status: CheckStatus,
    pub label: &'a str,
    pub message: &'a str,
    pub hint: Option<&'a str>,
}

impl CheckLine<'_> {
    /// The result line, then the indented hint line if there is one.
    pub fn lines(&self, ctx: Ctx<'_>) -> Vec<Line<'static>> {
        let col = CheckStatus::ALL
            .iter()
            .map(|s| ctx.glyphs.width(s.glyph()))
            .max()
            .unwrap_or(1);
        let glyph = ctx.glyphs.get(self.status.glyph());
        let pad = " ".repeat(col - display_width(glyph));
        let style = self.status.role().map(|r| ctx.style(r)).unwrap_or_default();
        let mut out = vec![Line::from(vec![
            Span::styled(format!("{glyph}{pad}"), style),
            Span::raw(format!(" {}  {}", self.label, self.message)),
        ])];
        if let Some(hint) = self.hint {
            out.push(Line::from(Span::styled(
                format!("  {} {hint}", ctx.glyphs.get(Glyph::Arrow)),
                ctx.style(Role::Muted),
            )));
        }
        out
    }
}
