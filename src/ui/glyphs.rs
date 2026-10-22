//! Glyphs with ASCII fallbacks, and locale detection.

use std::borrow::Cow;

use unicode_width::UnicodeWidthStr;

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
        let get = |k: &str| std::env::var(k).ok();
        Self {
            lc_all: get("LC_ALL"),
            lc_ctype: get("LC_CTYPE"),
            lang: get("LANG"),
        }
    }

    /// The first non-empty of `LC_ALL`, `LC_CTYPE`, `LANG` names a UTF-8 charset.
    pub fn is_utf8(&self) -> bool {
        let first = [&self.lc_all, &self.lc_ctype, &self.lang]
            .into_iter()
            .flatten()
            .find(|v| !v.is_empty());
        first.is_some_and(|v| {
            let v = v.to_ascii_lowercase();
            v.contains("utf-8") || v.contains("utf8")
        })
    }
}

/// ASCII mode: the explicit `--ascii` flag wins, else a non-UTF-8 locale.
pub fn ascii_mode(flag: bool, locale: &LocaleEnv) -> bool {
    flag || !locale.is_utf8()
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

    pub fn get(&self, glyph: Glyph) -> &'static str {
        let (unicode, ascii) = match glyph {
            Glyph::Pin => ("📌", "[p]"),
            Glyph::VBar => ("│", "|"),
            Glyph::HRule => ("─", "-"),
            Glyph::Ellipsis => ("…", "..."),
            Glyph::Newline => ("⏎", "\\n"),
            Glyph::Ok => ("✓", "ok"),
            Glyph::Info => ("i", "info"),
            Glyph::Warn => ("!", "warn"),
            Glyph::Fail => ("✗", "FAIL"),
            Glyph::Arrow => ("→", "->"),
            Glyph::Select => (">", ">"),
        };
        if self.ascii { ascii } else { unicode }
    }

    /// Display width of a glyph in the current mode.
    pub fn width(&self, glyph: Glyph) -> usize {
        UnicodeWidthStr::width(self.get(glyph))
    }

    /// Copy text with its non-ASCII punctuation replaced in ASCII mode.
    pub fn text<'a>(&self, s: &'a str) -> Cow<'a, str> {
        if !self.ascii || s.is_ascii() {
            return Cow::Borrowed(s);
        }
        Cow::Owned(
            s.replace('…', "...")
                .replace('—', "-")
                .replace('×', "x")
                .replace('→', "->"),
        )
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
