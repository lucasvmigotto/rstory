//! One-line text helpers: escaping, display width and truncation.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::glyphs::{Glyph, Glyphs};

/// A display unit: one grapheme, or the escape sequence of a control character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seg {
    pub text: String,
    pub width: usize,
    pub matched: bool,
}

pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

fn escape_char(c: char, glyphs: &Glyphs, out: &mut String) {
    match c {
        '\n' => out.push_str(glyphs.get(Glyph::Newline)),
        '\t' => out.push_str("\\t"),
        '\r' => out.push_str("\\r"),
        c if (c as u32) < 0x100 => out.push_str(&format!("\\x{:02x}", c as u32)),
        c => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
    }
}

/// Splits `text` into display units. Control characters are escaped; `matches`
/// are grapheme indexes into `text` to mark.
pub fn segments(text: &str, glyphs: &Glyphs, matches: &[usize]) -> Vec<Seg> {
    text.graphemes(true)
        .enumerate()
        .map(|(i, g)| {
            let shown = if g.chars().any(char::is_control) {
                let mut s = String::new();
                for c in g.chars() {
                    if c.is_control() {
                        escape_char(c, glyphs, &mut s);
                    } else {
                        s.push(c);
                    }
                }
                s
            } else {
                g.to_string()
            };
            Seg {
                width: display_width(&shown),
                text: shown,
                matched: matches.contains(&i),
            }
        })
        .collect()
}

/// Total display width of the segments.
pub fn segs_width(segs: &[Seg]) -> usize {
    segs.iter().map(|s| s.width).sum()
}

/// Cuts to `max` cells at a unit boundary, ending in the ellipsis glyph when cut.
pub fn truncate(segs: Vec<Seg>, max: usize, glyphs: &Glyphs) -> Vec<Seg> {
    if segs_width(&segs) <= max {
        return segs;
    }
    let ellipsis = glyphs.get(Glyph::Ellipsis);
    let ew = display_width(ellipsis);
    let (budget, tail) = if ew <= max {
        (max - ew, ellipsis.to_string())
    } else {
        let mut t = String::new();
        for c in ellipsis.chars() {
            if display_width(&t) + display_width(c.encode_utf8(&mut [0; 4])) <= max {
                t.push(c);
            }
        }
        (0, t)
    };
    let mut used = 0;
    let mut out = Vec::new();
    if ew <= max {
        for s in segs {
            if used + s.width > budget {
                break;
            }
            used += s.width;
            out.push(s);
        }
    }
    if !tail.is_empty() {
        out.push(Seg {
            width: display_width(&tail),
            text: tail,
            matched: false,
        });
    }
    out
}

/// Escapes control characters and truncates to `max` cells.
pub fn one_line(text: &str, max: usize, glyphs: &Glyphs) -> String {
    let segs = truncate(segments(text, glyphs, &[]), max, glyphs);
    segs.iter().map(|s| s.text.as_str()).collect()
}

/// Hard-wraps to `width` cells, keeping real newlines as line breaks.
pub fn wrap(text: &str, width: usize, glyphs: &Glyphs) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    let text = text.replace("\r\n", "\n");
    let mut out = Vec::new();
    for line in text.split('\n') {
        let mut cur = String::new();
        let mut w = 0;
        for seg in segments(line, glyphs, &[]) {
            if w + seg.width > width && w > 0 {
                out.push(std::mem::take(&mut cur));
                w = 0;
            }
            cur.push_str(&seg.text);
            w += seg.width;
        }
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn join(segs: &[Seg]) -> String {
        segs.iter().map(|s| s.text.as_str()).collect()
    }

    const U: Glyphs = Glyphs::new(false);
    const A: Glyphs = Glyphs::new(true);

    #[test]
    fn control_characters_are_escaped() {
        let segs = segments("a\tb\nc\r\nd\u{1b}e\u{7f}", &U, &[]);
        assert_eq!(join(&segs), "a\\tb⏎c\\r⏎d\\x1be\\x7f");
        let segs = segments("a\nb", &A, &[]);
        assert_eq!(join(&segs), "a\\nb");
        assert!(!one_line("x\u{0}y", 20, &U).contains('\u{0}'));
    }

    #[test]
    fn widths_follow_unicode_width() {
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("日本"), 4);
        let segs = segments("日a", &U, &[]);
        assert_eq!(segs_width(&segs), 3);
    }

    #[test]
    fn graphemes_stay_whole() {
        // e + combining acute is one unit of width 1.
        let segs = segments("e\u{301}x", &U, &[]);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].width, 1);
        // a family emoji is one unit.
        let fam = "👨\u{200d}👩\u{200d}👧";
        assert_eq!(segments(fam, &U, &[]).len(), 1);
    }

    #[test]
    fn matches_mark_graphemes_not_bytes() {
        let segs = segments("日本語", &U, &[1]);
        assert_eq!(
            segs.iter().map(|s| s.matched).collect::<Vec<_>>(),
            vec![false, true, false]
        );
    }

    #[test]
    fn truncation_cuts_at_unit_boundaries_by_width() {
        let t = |s: &str, max, g: &Glyphs| one_line(s, max, g);
        assert_eq!(t("hello", 5, &U), "hello");
        assert_eq!(t("hello world", 6, &U), "hello…");
        assert_eq!(t("日本語テキスト", 7, &U), "日本語…");
        // A wide char that would straddle the boundary is dropped whole.
        assert_eq!(t("日本語テキスト", 6, &U), "日本…");
        assert_eq!(t("e\u{301}e\u{301}e\u{301}", 2, &U), "e\u{301}…");
        assert_eq!(t("hello world", 8, &A), "hello...");
        assert_eq!(t("hello", 0, &U), "");
        assert_eq!(t("hello", 1, &U), "…");
        assert_eq!(t("hello", 2, &A), "..");
    }

    #[test]
    fn truncated_width_never_exceeds_max() {
        for max in 0..30 {
            for g in [&U, &A] {
                let out = one_line("日本語 mixed ascii and 😀 emoji e\u{301}", max, g);
                assert!(display_width(&out) <= max, "{max}: {out:?}");
            }
        }
    }

    #[test]
    fn wrap_keeps_newlines_and_splits_by_width() {
        assert_eq!(wrap("ab\ncd", 10, &U), vec!["ab", "cd"]);
        assert_eq!(wrap("abcdef", 4, &U), vec!["abcd", "ef"]);
        assert_eq!(wrap("日本語", 5, &U), vec!["日本", "語"]);
        assert_eq!(wrap("a\r\nb\n", 10, &U), vec!["a", "b", ""]);
        assert_eq!(wrap("a\tb", 10, &U), vec!["a\\tb"]);
        assert!(wrap("abc", 0, &U).is_empty());
    }
}
