//! The components of `specs/000-design-system/components.md`.

use ratatui::style::Style;

use super::glyphs::Glyphs;
use super::theme::{Role, Theme};

mod banner;
mod check_line;
mod entry_line;
mod entry_row;
mod footer;
mod preview;
mod query_line;
mod state_badge;

pub use banner::Banner;
pub use check_line::{CheckLine, CheckStatus};
pub use entry_line::EntryLine;
pub use entry_row::EntryRow;
pub use footer::{Footer, FooterKind};
pub use preview::{PreviewPane, preview_percent};
pub use query_line::QueryLine;
pub use state_badge::{RecState, StateBadge};

/// What every component needs to draw itself.
#[derive(Clone, Copy)]
pub struct Ctx<'a> {
    pub theme: &'a Theme,
    pub glyphs: &'a Glyphs,
}

impl<'a> Ctx<'a> {
    pub fn new(theme: &'a Theme, glyphs: &'a Glyphs) -> Self {
        Self { theme, glyphs }
    }

    pub fn style(&self, role: Role) -> Style {
        self.theme.style(role)
    }
}
