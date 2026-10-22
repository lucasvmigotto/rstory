//! One-line text helpers: escaping, display width and truncation.

use super::glyphs::Glyphs;

/// A display unit: one grapheme, or the escape sequence of a control character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seg {
    pub text: String,
    pub width: usize,
    pub matched: bool,
}

pub fn display_width(_s: &str) -> usize {
    todo!()
}

/// Splits `text` into display units. Control characters are escaped; `matches`
/// are grapheme indexes into `text` to mark.
pub fn segments(_text: &str, _glyphs: &Glyphs, _matches: &[usize]) -> Vec<Seg> {
    todo!()
}

/// Total display width of the segments.
pub fn segs_width(_segs: &[Seg]) -> usize {
    todo!()
}

/// Cuts to `max` cells at a unit boundary, ending in the ellipsis glyph when cut.
pub fn truncate(_segs: Vec<Seg>, _max: usize, _glyphs: &Glyphs) -> Vec<Seg> {
    todo!()
}

/// Escapes control characters and truncates to `max` cells.
pub fn one_line(_text: &str, _max: usize, _glyphs: &Glyphs) -> String {
    todo!()
}

/// Hard-wraps to `width` cells, keeping real newlines as line breaks.
pub fn wrap(_text: &str, _width: usize, _glyphs: &Glyphs) -> Vec<String> {
    todo!()
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
