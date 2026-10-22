//! Serializing styled lines for the CLI.

use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
};

/// The line's text with no styling.
pub fn plain(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

fn color_code(c: Color) -> Option<String> {
    Some(match c {
        Color::Reset => return None,
        Color::Black => "30".into(),
        Color::Red => "31".into(),
        Color::Green => "32".into(),
        Color::Yellow => "33".into(),
        Color::Blue => "34".into(),
        Color::Magenta => "35".into(),
        Color::Cyan => "36".into(),
        Color::Gray => "37".into(),
        Color::DarkGray => "90".into(),
        Color::LightRed => "91".into(),
        Color::LightGreen => "92".into(),
        Color::LightYellow => "93".into(),
        Color::LightBlue => "94".into(),
        Color::LightMagenta => "95".into(),
        Color::LightCyan => "96".into(),
        Color::White => "97".into(),
        Color::Indexed(n) => format!("38;5;{n}"),
        Color::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
    })
}

fn sgr(style: Style) -> String {
    let mut codes = Vec::new();
    codes.extend(style.fg.and_then(color_code));
    for (m, code) in [
        (Modifier::BOLD, "1"),
        (Modifier::UNDERLINED, "4"),
        (Modifier::REVERSED, "7"),
    ] {
        if style.add_modifier.contains(m) {
            codes.push(code.to_string());
        }
    }
    codes.join(";")
}

/// The line with SGR escapes. Mono-tier styles hold no colors, so none are emitted.
pub fn ansi(line: &Line<'_>) -> String {
    let mut out = String::new();
    for span in &line.spans {
        let codes = sgr(line.style.patch(span.style));
        if codes.is_empty() {
            out.push_str(&span.content);
        } else {
            out.push_str(&format!("\x1b[{codes}m{}\x1b[0m", span.content));
        }
    }
    out
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
