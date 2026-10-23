//! Layout and rendering of a [`PickerState`] (tui.md "Screens"), built from the design system
//! components: `QueryLine`, a list of `EntryRow`, `PreviewPane`, `Banner` and `Footer`.
//!
//! Full screen, top to bottom: the query line, the read-only banner (when read-only), a rule,
//! the list with the preview on the right, a rule, the footer (two lines with F1's keymap).

use ratatui::{buffer::Buffer, layout::Rect};

use crate::ui::widgets::Ctx;

use super::state::PickerState;

/// Draws `state` full screen into `area`.
pub fn render(state: &PickerState, area: Rect, buf: &mut Buffer, ctx: Ctx<'_>) {
    let _ = (state, area, buf, ctx);
    todo!()
}
