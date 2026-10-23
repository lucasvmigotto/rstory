//! Human and JSON output for `ls` and `show`, built on the design system (`src/ui`).
//!
//! On a terminal `ls` is width-aware and styled; piped it is tab-separated
//! (`index  short_id  pinned  age_seconds  preview`) with no styling, for `cut` and `awk`.
//! Previews arrive escaped from the daemon; nothing here prints content except `show`.

use std::io::IsTerminal;

use serde_json::json;
use time::OffsetDateTime;

use crate::ipc::protocol::EntrySummary;
use crate::ui::ansi;
use crate::ui::glyphs::{Glyphs, LocaleEnv, ascii_mode};
use crate::ui::theme::{Background, TermEnv, Theme, ThemePref, Tier};
use crate::ui::widgets::{Ctx, EntryLine};

/// What the output goes to: a terminal or a pipe, how wide, how colorful.
#[derive(Debug, Clone, Copy)]
pub struct Term {
    /// Whether stdout is a terminal.
    pub tty: bool,
    /// Columns available (80 when unknown).
    pub width: usize,
    theme: Theme,
    glyphs: Glyphs,
}

impl Term {
    /// An explicit output description (tests).
    #[must_use]
    pub fn new(tty: bool, width: usize, tier: Tier, ascii: bool) -> Self {
        Self {
            tty,
            width,
            theme: Theme::new(tier, Background::Dark),
            glyphs: Glyphs::new(ascii),
        }
    }

    /// Looks at stdout, `NO_COLOR`, `TERM`, the locale and the terminal size.
    #[must_use]
    pub fn detect() -> Self {
        let out = std::io::stdout();
        let tty = out.is_terminal();
        let theme = Theme::detect(
            ThemePref::Terminal,
            &TermEnv::from_process(),
            Background::Dark,
        );
        let tier = if tty { theme.tier() } else { Tier::Mono };
        let ascii = ascii_mode(false, &LocaleEnv::from_process());
        let width = if tty { terminal_width(&out) } else { 80 };
        Self::new(tty, width, tier, ascii)
    }

    /// The design-system context for widgets.
    #[must_use]
    pub fn ctx(&self) -> Ctx<'_> {
        Ctx::new(&self.theme, &self.glyphs)
    }

    /// Copy text with the ASCII fallbacks applied.
    #[must_use]
    pub fn text<'a>(&self, s: &'a str) -> std::borrow::Cow<'a, str> {
        self.glyphs.text(s)
    }

    /// Whether output is limited to ASCII.
    #[must_use]
    pub fn is_ascii(&self) -> bool {
        self.glyphs.is_ascii()
    }

    /// Whether styling is on (a color tier above mono).
    #[must_use]
    pub fn styled(&self) -> bool {
        self.theme.tier() != Tier::Mono
    }
}

fn terminal_width(out: &std::io::Stdout) -> usize {
    if let Ok(ws) = rustix::termios::tcgetwinsize(out)
        && ws.ws_col > 0
    {
        return usize::from(ws.ws_col);
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse().ok())
        .filter(|w| *w > 0)
        .unwrap_or(80)
}

/// Seconds from `then` to `now`; a future time counts as now.
#[must_use]
pub fn age_secs(now: OffsetDateTime, then: OffsetDateTime) -> u64 {
    u64::try_from((now - then).whole_seconds()).unwrap_or(0)
}

/// A short age: `59s`, `3m`, `2h`, `5d`.
#[must_use]
pub fn age(now: OffsetDateTime, then: OffsetDateTime) -> String {
    let s = age_secs(now, then);
    match s {
        0..=59 => format!("{s}s"),
        60..=3599 => format!("{}m", s / 60),
        3600..=86_399 => format!("{}h", s / 3600),
        _ => format!("{}d", s / 86_400),
    }
}

fn entry_line<'a>(e: &'a EntrySummary, age: &'a str) -> EntryLine<'a> {
    EntryLine {
        index: e.index as usize,
        short_id: &e.short_id,
        pinned: e.pinned,
        age,
        preview: &e.preview,
    }
}

/// The `ls` lines, each ending in a newline; empty for an empty history.
#[must_use]
pub fn ls_text(entries: &[EntrySummary], now: OffsetDateTime, term: &Term) -> String {
    let ctx = term.ctx();
    let mut out = String::new();
    for e in entries {
        if term.tty {
            let styled = entry_line(e, &age(now, e.last_copied)).tty(term.width, ctx);
            out.push_str(&if term.styled() {
                ansi::ansi(&styled)
            } else {
                ansi::plain(&styled)
            });
        } else {
            out.push_str(&entry_line(e, &age_secs(now, e.last_copied).to_string()).tsv(ctx));
        }
        out.push('\n');
    }
    out
}

/// `{"schema":1,"entries":[…],"read_only":bool}` and a newline.
#[must_use]
pub fn ls_json(entries: &[EntrySummary], read_only: bool) -> String {
    let mut text = json!({"schema": 1, "entries": entries, "read_only": read_only}).to_string();
    text.push('\n');
    text
}

/// `{"schema":1,"entry":…,"content":…}` and a newline. Content that is not UTF-8 is lossy.
#[must_use]
pub fn show_json(entry: &EntrySummary, content: &[u8]) -> String {
    let mut text = json!({
        "schema": 1,
        "entry": entry,
        "content": String::from_utf8_lossy(content),
    })
    .to_string();
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use time::macros::datetime;

    use super::*;
    use crate::ipc::protocol::EntrySummary;
    use crate::ui::theme::Tier;

    const NOW: time::OffsetDateTime = datetime!(2026-10-21 14:05:00 UTC);

    fn entry(index: u32, short: &str, pinned: bool, ago_secs: i64, preview: &str) -> EntrySummary {
        let at = NOW - time::Duration::seconds(ago_secs);
        EntrySummary {
            id: format!("01JBD8Z0R5S7K3M2N1P0Q9T8V{index}"),
            short_id: short.to_string(),
            index,
            size: u32::try_from(preview.len()).unwrap(),
            created: at,
            last_copied: at,
            pinned,
            preview: preview.to_string(),
        }
    }

    fn sample() -> Vec<EntrySummary> {
        vec![
            entry(0, "01JBD8Z", false, 180, "https://example.org/rstory"),
            entry(1, "01JBD8X", true, 2 * 3600 + 5, "git rebase -i HEAD~3"),
            entry(
                2,
                "01JBC2K",
                false,
                86_400 + 60,
                "cargo test -p rstory --test daemon_restore -- --nocapture --test-threads=1 and a very long tail",
            ),
        ]
    }

    #[test]
    fn ages_are_short() {
        let a = |s: i64| age(NOW, NOW - time::Duration::seconds(s));
        assert_eq!(a(0), "0s");
        assert_eq!(a(59), "59s");
        assert_eq!(a(60), "1m");
        assert_eq!(a(3599), "59m");
        assert_eq!(a(3600), "1h");
        assert_eq!(a(86_399), "23h");
        assert_eq!(a(86_400), "1d");
        assert_eq!(a(40 * 86_400), "40d");
        assert_eq!(a(-30), "0s", "a future time counts as now");
    }

    #[test]
    fn tty_at_80_columns() {
        let term = Term::new(true, 80, Tier::Mono, false);
        insta::assert_snapshot!(ls_text(&sample(), NOW, &term), @r"
        [0]   01JBD8Z      3m  https://example.org/rstory
        [1]   01JBD8X 📌   2h  git rebase -i HEAD~3
        [2]   01JBC2K      1d  cargo test -p rstory --test daemon_restore -- --nocaptur…
        ");
    }

    #[test]
    fn tty_at_120_columns_shows_more_preview() {
        let term = Term::new(true, 120, Tier::Mono, false);
        let out = ls_text(&sample(), NOW, &term);
        assert!(
            out.contains("--test-threads=1 and a very long tail"),
            "{out}"
        );
        assert!(
            out.lines()
                .all(|l| unicode_width::UnicodeWidthStr::width(l) <= 120)
        );
    }

    #[test]
    fn tty_at_40_columns_ascii_keeps_index_and_id() {
        let term = Term::new(true, 40, Tier::Mono, true);
        let out = ls_text(&sample(), NOW, &term);
        insta::assert_snapshot!(out, @r"
        [0]   01JBD8Z       3m  https://examp...
        [1]   01JBD8X [p]   2h  git rebase -i...
        [2]   01JBC2K       1d  cargo test -p...
        ");
    }

    #[test]
    fn a_colored_tty_emits_escapes_and_mono_does_not() {
        let colored = ls_text(&sample(), NOW, &Term::new(true, 80, Tier::Ansi16, false));
        assert!(colored.contains("\x1b["), "{colored:?}");
        let mono = ls_text(&sample(), NOW, &Term::new(true, 80, Tier::Mono, false));
        assert!(!mono.contains('\x1b'));
    }

    #[test]
    fn piped_is_tab_separated_with_age_in_seconds() {
        let term = Term::new(false, 80, Tier::Mono, false);
        insta::assert_snapshot!(ls_text(&sample(), NOW, &term), @r"
        0	01JBD8Z		180	https://example.org/rstory
        1	01JBD8X	pinned	7205	git rebase -i HEAD~3
        2	01JBC2K		86460	cargo test -p rstory --test daemon_restore -- --nocapture --test-threads=1 and a very long tail
        ");
    }

    #[test]
    fn piped_never_holds_more_than_256_characters_of_preview() {
        let long = "x".repeat(400);
        let e = [entry(0, "01JBD8Z", false, 1, &long)];
        let out = ls_text(&e, NOW, &Term::new(false, 80, Tier::Mono, false));
        let preview = out.trim_end().split('\t').next_back().unwrap();
        assert_eq!(preview.chars().count(), 256);
    }

    #[test]
    fn control_characters_in_a_preview_stay_escaped() {
        let e = [entry(0, "01JBD8Z", false, 1, "a\\tb\\n\x1b[31m")];
        for tty in [true, false] {
            let out = ls_text(&e, NOW, &Term::new(tty, 80, Tier::Mono, false));
            assert!(!out.contains('\x1b'), "{out:?}");
            assert_eq!(out.lines().count(), 1);
        }
    }

    #[test]
    fn no_entries_print_nothing() {
        assert_eq!(
            ls_text(&[], NOW, &Term::new(true, 80, Tier::Mono, false)),
            ""
        );
    }

    #[test]
    fn ls_json_is_schema_1_with_the_read_only_flag() {
        let text = ls_json(&sample(), true);
        assert!(text.ends_with('\n'));
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["schema"], 1);
        assert_eq!(v["read_only"], true);
        assert_eq!(v["entries"].as_array().unwrap().len(), 3);
        assert_eq!(v["entries"][1]["pinned"], true);
        assert_eq!(v["entries"][0]["created"], "2026-10-21T14:02:00Z");
    }

    #[test]
    fn show_json_carries_the_entry_and_its_content() {
        let e = sample().remove(1);
        let v: Value = serde_json::from_str(&show_json(&e, b"git rebase -i HEAD~3")).unwrap();
        assert_eq!(v["schema"], 1);
        assert_eq!(v["entry"]["short_id"], "01JBD8X");
        assert_eq!(v["content"], "git rebase -i HEAD~3");
    }

    #[test]
    fn show_json_of_invalid_utf8_is_lossy() {
        let e = sample().remove(0);
        let v: Value = serde_json::from_str(&show_json(&e, b"a\xffb")).unwrap();
        assert_eq!(v["content"], "a\u{fffd}b");
    }
}
