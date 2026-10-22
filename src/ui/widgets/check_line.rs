use ratatui::text::Line;

use super::Ctx;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckStatus {
    Ok,
    Info,
    Warn,
    Fail,
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
    pub fn lines(&self, _ctx: Ctx<'_>) -> Vec<Line<'static>> {
        todo!()
    }
}
