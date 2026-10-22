//! Glyphs with ASCII fallbacks, and locale detection.

use std::borrow::Cow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Pin,
    VBar,
    HRule,
    Ellipsis,
    Newline,
    Ok,
    Info,
    Warn,
    Fail,
    Arrow,
    Select,
}

impl Glyph {
    pub const ALL: [Glyph; 11] = [
        Glyph::Pin,
        Glyph::VBar,
        Glyph::HRule,
        Glyph::Ellipsis,
        Glyph::Newline,
        Glyph::Ok,
        Glyph::Info,
        Glyph::Warn,
        Glyph::Fail,
        Glyph::Arrow,
        Glyph::Select,
    ];
}

/// The locale variables, in POSIX precedence order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocaleEnv {
    pub lc_all: Option<String>,
    pub lc_ctype: Option<String>,
    pub lang: Option<String>,
}

impl LocaleEnv {
    pub fn from_process() -> Self {
        todo!()
    }

    /// The first non-empty of `LC_ALL`, `LC_CTYPE`, `LANG` names a UTF-8 charset.
    pub fn is_utf8(&self) -> bool {
        todo!()
    }
}

/// ASCII mode: the explicit `--ascii` flag wins, else a non-UTF-8 locale.
pub fn ascii_mode(_flag: bool, _locale: &LocaleEnv) -> bool {
    todo!()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glyphs {
    ascii: bool,
}

impl Glyphs {
    pub const fn new(ascii: bool) -> Self {
        Self { ascii }
    }

    pub fn is_ascii(&self) -> bool {
        self.ascii
    }

    pub fn get(&self, _glyph: Glyph) -> &'static str {
        todo!()
    }

    /// Display width of a glyph in the current mode.
    pub fn width(&self, _glyph: Glyph) -> usize {
        todo!()
    }

    /// Copy text with its non-ASCII punctuation replaced in ASCII mode.
    pub fn text<'a>(&self, _s: &'a str) -> Cow<'a, str> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale(all: &str, ctype: &str, lang: &str) -> LocaleEnv {
        let o = |s: &str| (!s.is_empty()).then(|| s.to_string());
        LocaleEnv {
            lc_all: o(all),
            lc_ctype: o(ctype),
            lang: o(lang),
        }
    }

    #[test]
    fn table_matches_tokens_md() {
        let u = Glyphs::new(false);
        let a = Glyphs::new(true);
        let table = [
            (Glyph::Pin, "📌", "[p]"),
            (Glyph::VBar, "│", "|"),
            (Glyph::HRule, "─", "-"),
            (Glyph::Ellipsis, "…", "..."),
            (Glyph::Newline, "⏎", "\\n"),
            (Glyph::Ok, "✓", "ok"),
            (Glyph::Warn, "!", "warn"),
            (Glyph::Fail, "✗", "FAIL"),
            (Glyph::Arrow, "→", "->"),
            (Glyph::Select, ">", ">"),
        ];
        for (g, uni, asc) in table {
            assert_eq!(u.get(g), uni, "{g:?}");
            assert_eq!(a.get(g), asc, "{g:?}");
        }
    }

    #[test]
    fn ascii_glyphs_are_ascii_and_widths_agree() {
        let a = Glyphs::new(true);
        let u = Glyphs::new(false);
        for g in Glyph::ALL {
            assert!(a.get(g).is_ascii(), "{g:?}");
            assert_eq!(a.width(g), a.get(g).len(), "{g:?}");
            assert_eq!(
                u.width(g),
                unicode_width::UnicodeWidthStr::width(u.get(g)),
                "{g:?}"
            );
        }
    }

    #[test]
    fn locale_uses_the_first_non_empty_variable() {
        assert!(locale("", "", "en_US.UTF-8").is_utf8());
        assert!(locale("", "", "C.utf8").is_utf8());
        assert!(!locale("", "", "C").is_utf8());
        assert!(!locale("", "", "en_US.ISO-8859-1").is_utf8());
        assert!(!locale("C", "en_US.UTF-8", "en_US.UTF-8").is_utf8());
        assert!(locale("en_US.UTF-8", "C", "C").is_utf8());
        assert!(!locale("", "C", "en_US.UTF-8").is_utf8());
        assert!(!locale("", "", "").is_utf8());
    }

    #[test]
    fn the_flag_overrides_the_locale() {
        let utf8 = locale("", "", "en_US.UTF-8");
        let c = locale("", "", "C");
        assert!(ascii_mode(true, &utf8));
        assert!(!ascii_mode(false, &utf8));
        assert!(ascii_mode(false, &c));
        assert!(ascii_mode(true, &c));
    }

    #[test]
    fn text_falls_back_only_in_ascii_mode() {
        let s = "Restoring… Paused — resumes need 20×5 →";
        assert_eq!(Glyphs::new(false).text(s), s);
        assert_eq!(
            Glyphs::new(true).text(s),
            "Restoring... Paused - resumes need 20x5 ->"
        );
        assert!(Glyphs::new(true).text("Delete this entry? y/n").is_ascii());
    }
}
