//! Shared terminal UI helpers: theme, glyphs, copy and the widgets built on them.
//!
//! Spec: `specs/000-design-system`. Rules that hold for everything here:
//! state is never conveyed by color alone (Constitution VII), and nothing in
//! this module logs entry content (Constitution III).

pub mod ansi;
pub mod copy;
pub mod glyphs;
pub mod text;
pub mod theme;
pub mod widgets;

#[cfg(test)]
mod tests;
