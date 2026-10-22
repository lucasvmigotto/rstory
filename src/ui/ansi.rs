//! Serializing styled lines for the CLI.

use ratatui::text::Line;

/// The line's text with no styling.
pub fn plain(_line: &Line<'_>) -> String {
    todo!()
}

/// The line with SGR escapes. Mono-tier styles hold no colors, so none are emitted.
pub fn ansi(_line: &Line<'_>) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use ratatui::{
        style::{Color, Modifier, Style},
        text::Span,
    };

    use super::*;

    fn line() -> Line<'static> {
        Line::from(vec![
            Span::raw("a "),
            Span::styled(
                "b",
                Style::new()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
            Span::styled("c", Style::new().fg(Color::Indexed(178))),
            Span::styled("d", Style::new().fg(Color::Rgb(1, 2, 3))),
            Span::styled("e", Style::new().add_modifier(Modifier::REVERSED)),
            Span::styled("f", Style::new().fg(Color::DarkGray)),
        ])
    }

    #[test]
    fn plain_drops_styles() {
        assert_eq!(plain(&line()), "a bcdef");
    }

    #[test]
    fn ansi_emits_sgr_per_styled_span() {
        let out = ansi(&line());
        assert_eq!(
            out,
            "a \x1b[36;1;4mb\x1b[0m\x1b[38;5;178mc\x1b[0m\x1b[38;2;1;2;3md\x1b[0m\x1b[7me\x1b[0m\x1b[90mf\x1b[0m"
        );
    }

    #[test]
    fn unstyled_text_has_no_escapes() {
        assert_eq!(ansi(&Line::from("plain")), "plain");
    }
}
