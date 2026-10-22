//! Snapshot harness and the component snapshots (T002, T006, T008).
//!
//! Every snapshot is the rendered buffer as text, followed, per row, by the
//! style runs of its cells. A distinction made by color alone therefore shows
//! up in review: it would be a style run with no matching glyph or word.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    text::Line,
    widgets::{Paragraph, Widget},
};
use unicode_width::UnicodeWidthStr;

use super::{
    glyphs::Glyphs,
    theme::{Background, Theme, Tier},
    widgets::{
        Banner, CheckLine, CheckStatus, Ctx, EntryLine, EntryRow, Footer, FooterKind, PreviewPane,
        QueryLine, RecState, StateBadge, preview_percent,
    },
};

/// One point of the render matrix.
#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub tier: Tier,
    pub width: u16,
    pub ascii: bool,
}

impl Case {
    /// Snapshot name: component, tier, width, charset.
    pub fn name(&self, component: &str) -> String {
        format!(
            "{component}_{}_{}_{}",
            self.tier.name(),
            self.width,
            if self.ascii { "ascii" } else { "utf8" }
        )
    }
}

/// Tier x width x ascii: 16 cases.
pub fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for tier in Tier::ALL {
        for width in [80, 40] {
            for ascii in [false, true] {
                out.push(Case { tier, width, ascii });
            }
        }
    }
    out
}

fn color_name(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02X}{g:02X}{b:02X}"),
        Color::Indexed(n) => format!("idx{n}"),
        other => format!("{other:?}"),
    }
}

fn cell_style(buf: &Buffer, x: u16, y: u16) -> String {
    let cell = &buf[(x, y)];
    let mut parts = Vec::new();
    if cell.fg != Color::Reset {
        parts.push(format!("fg={}", color_name(cell.fg)));
    }
    if cell.bg != Color::Reset {
        parts.push(format!("bg={}", color_name(cell.bg)));
    }
    for (m, n) in [
        (Modifier::REVERSED, "REV"),
        (Modifier::BOLD, "BOLD"),
        (Modifier::UNDERLINED, "UL"),
        (Modifier::DIM, "DIM"),
        (Modifier::ITALIC, "ITAL"),
        (Modifier::CROSSED_OUT, "STRIKE"),
        (Modifier::SLOW_BLINK, "BLINK"),
    ] {
        if cell.modifier.contains(m) {
            parts.push(n.to_string());
        }
    }
    parts.join(" ")
}

/// Text of each row, then its non-default style runs as `~ from..to style`.
pub fn dump(buf: &Buffer) -> String {
    let area = buf.area;
    let mut out = String::new();
    for y in 0..area.height {
        let mut row = String::new();
        let mut skip = 0usize;
        for x in 0..area.width {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let sym = buf[(x, y)].symbol();
            skip = sym.width().saturating_sub(1);
            row.push_str(sym);
        }
        out.push('|');
        out.push_str(row.trim_end());
        out.push('\n');
        let mut runs = Vec::new();
        let mut x = 0;
        while x < area.width {
            let s = cell_style(buf, x, y);
            let mut end = x + 1;
            while end < area.width && cell_style(buf, end, y) == s {
                end += 1;
            }
            if !s.is_empty() {
                runs.push(format!("{x}..{end} {s}"));
            }
            x = end;
        }
        if !runs.is_empty() {
            out.push_str("  ~ ");
            out.push_str(&runs.join(" ; "));
            out.push('\n');
        }
    }
    out
}

/// Renders into a `width x height` buffer and returns the headed dump.
pub fn render(case: &Case, height: u16, draw: impl FnOnce(Ctx<'_>, Rect, &mut Buffer)) -> String {
    let theme = Theme::new(case.tier, Background::Dark);
    let glyphs = Glyphs::new(case.ascii);
    let area = Rect::new(0, 0, case.width, height);
    let mut buf = Buffer::empty(area);
    draw(Ctx::new(&theme, &glyphs), area, &mut buf);
    format!(
        "# tier={} width={} ascii={}\n{}",
        case.tier.name(),
        case.width,
        case.ascii,
        dump(&buf)
    )
}

fn row(area: Rect, y: u16) -> Rect {
    Rect::new(area.x, area.y + y, area.width, 1)
}

fn snap(component: &str, case: &Case, out: String) {
    insta::assert_snapshot!(case.name(component), out);
}

#[test]
fn matrix_has_sixteen_cases() {
    assert_eq!(cases().len(), 16);
}

#[test]
fn harness_makes_style_visible() {
    let case = Case {
        tier: Tier::Ansi16,
        width: 10,
        ascii: false,
    };
    let out = render(&case, 1, |_, area, buf| {
        buf.set_string(
            area.x,
            area.y,
            "ab",
            ratatui::style::Style::new()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    });
    assert!(out.contains("0..2 fg=Cyan BOLD"), "{out}");
}

#[test]
fn query_line_snapshots() {
    let long = "git push --force-with-lease origin feature/a-very-long-branch-name-indeed";
    let rows: [(&str, usize, usize, RecState); 5] = [
        ("", 14, 14, RecState::Recording),
        ("reb", 2, 14, RecState::Recording),
        ("zzz", 0, 14, RecState::PausedUntil { secs: 65 }),
        (long, 1, 14, RecState::Paused),
        (
            "x",
            3,
            3,
            RecState::Inactive {
                reason: "no clipboard access".into(),
            },
        ),
    ];
    for case in cases() {
        let out = render(&case, 5, |ctx, area, buf| {
            for (i, (q, shown, total, state)) in rows.iter().enumerate() {
                QueryLine {
                    query: q,
                    shown: *shown,
                    total: *total,
                    state,
                    ctx,
                }
                .render(row(area, i as u16), buf);
            }
        });
        snap("query_line", &case, out);
    }
}

#[test]
fn entry_row_snapshots() {
    let rows: [(&str, bool, bool, &str, &[usize]); 6] = [
        (
            "3m",
            false,
            false,
            "https://github.com/lucasvmigotto/rstory",
            &[],
        ),
        (
            "now",
            false,
            true,
            "cargo test -p rstory --test daemon_restore",
            &[],
        ),
        (
            "2h",
            true,
            false,
            "git reset --soft HEAD~1 && git push --force-with-lease",
            &[],
        ),
        ("3m", false, false, "git rebase -i HEAD~3", &[4, 5, 6]),
        ("5d", true, true, "git rebase -i HEAD~3", &[4, 5, 6]),
        (
            "1d",
            false,
            false,
            "日本語のテキスト\tline\nsecond e\u{301}nd",
            &[0],
        ),
    ];
    for case in cases() {
        let out = render(&case, 6, |ctx, area, buf| {
            for (i, (age, pinned, selected, preview, matches)) in rows.iter().enumerate() {
                EntryRow {
                    age,
                    pinned: *pinned,
                    selected: *selected,
                    preview,
                    matches,
                    ctx,
                }
                .render(row(area, i as u16), buf);
            }
        });
        snap("entry_row", &case, out);
    }
}

const PREVIEW: &str = "fn main() {\n\tprintln!(\"hello\");\n}\n日本語のテキストと、とても長い一行が折り返されます。\nA long line of plain words that has to wrap at the width of the pane.";

#[test]
fn preview_pane_snapshots() {
    for case in cases() {
        let out = render(&case, 9, |ctx, area, buf| {
            PreviewPane {
                content: Some(PREVIEW),
                ctx,
            }
            .render(area, buf);
        });
        snap("preview_content", &case, out);
        let out = render(&case, 4, |ctx, area, buf| {
            PreviewPane { content: None, ctx }.render(area, buf);
        });
        snap("preview_empty", &case, out);
    }
}

#[test]
fn preview_pane_shows_first_64_kib_then_the_remainder() {
    let big = "a".repeat(100 * 1024);
    let case = Case {
        tier: Tier::Mono,
        width: 40,
        ascii: false,
    };
    let out = render(&case, 4000, |ctx, area, buf| {
        PreviewPane {
            content: Some(&big),
            ctx,
        }
        .render(area, buf);
    });
    let a_count = out.matches('a').count();
    assert!((64 * 1024..65 * 1024).contains(&a_count), "{a_count}");
    assert!(out.contains(&format!("{} more bytes", 100 * 1024 - 64 * 1024)));
}

#[test]
fn preview_split_follows_the_tokens() {
    assert_eq!(preview_percent(120), Some(50));
    assert_eq!(preview_percent(100), Some(50));
    assert_eq!(preview_percent(99), Some(40));
    assert_eq!(preview_percent(60), Some(40));
    assert_eq!(preview_percent(59), None);
    assert_eq!(preview_percent(40), None);
}

#[test]
fn footer_snapshots() {
    for case in cases() {
        let out = render(&case, 5, |ctx, area, buf| {
            let kinds = [
                FooterKind::Hints { short: false },
                FooterKind::Hints { short: true },
                FooterKind::Status(super::copy::PICKER_PIN_LIMIT),
                FooterKind::Status(super::copy::PICKER_RESTORING),
                FooterKind::Confirm(super::copy::PICKER_CONFIRM_DELETE),
            ];
            for (i, kind) in kinds.into_iter().enumerate() {
                Footer { kind, ctx }.render(row(area, i as u16), buf);
            }
        });
        snap("footer", &case, out);
    }
}

#[test]
fn banner_snapshots() {
    for case in cases() {
        let out = render(&case, 1, |ctx, area, buf| {
            Banner { ctx }.render(area, buf);
        });
        snap("banner", &case, out);
    }
}

#[test]
fn state_badge_snapshots() {
    let states = [
        RecState::Recording,
        RecState::Paused,
        RecState::PausedUntil { secs: 65 },
        RecState::Inactive {
            reason: "no clipboard access".into(),
        },
    ];
    for case in cases() {
        let out = render(&case, 8, |ctx, area, buf| {
            for (i, state) in states.iter().enumerate() {
                for (j, short) in [false, true].into_iter().enumerate() {
                    StateBadge { state, short, ctx }.render(row(area, (i * 2 + j) as u16), buf);
                }
            }
        });
        snap("state_badge", &case, out);
    }
}

fn lines_dump(case: &Case, lines: Vec<Line<'static>>) -> String {
    let h = lines.len() as u16;
    render(case, h, |_, area, buf| {
        Paragraph::new(lines).render(area, buf);
    })
}

#[test]
fn entry_line_snapshots() {
    let long = "git reset --soft HEAD~1 && git push --force-with-lease origin main";
    let items = [
        (
            0,
            "a1b2c3",
            false,
            "3m",
            "https://github.com/lucasvmigotto/rstory",
        ),
        (1, "d4e5f6", true, "2h", long),
        (12, "0a0b0c", false, "5d", "日本語のテキスト\tand\nmore"),
    ];
    for case in cases() {
        let theme = Theme::new(case.tier, Background::Dark);
        let glyphs = Glyphs::new(case.ascii);
        let ctx = Ctx::new(&theme, &glyphs);
        let lines = items
            .iter()
            .map(|&(index, short_id, pinned, age, preview)| {
                EntryLine {
                    index,
                    short_id,
                    pinned,
                    age,
                    preview,
                }
                .tty(case.width as usize, ctx)
            })
            .collect();
        let mut out = lines_dump(&case, lines);
        out.push_str("# piped\n");
        for &(index, short_id, pinned, age, preview) in &items {
            let l = EntryLine {
                index,
                short_id,
                pinned,
                age,
                preview,
            };
            out.push_str(&l.tsv(ctx));
            out.push('\n');
        }
        snap("entry_line", &case, out);
    }
}

#[test]
fn check_line_snapshots() {
    let checks = [
        (CheckStatus::Ok, "store", "opened, 0700", None),
        (CheckStatus::Info, "backend", "x11 (XFIXES)", None),
        (
            CheckStatus::Warn,
            "autostart",
            "not installed",
            Some("run `rstory install-autostart`"),
        ),
        (
            CheckStatus::Fail,
            "daemon",
            "not running",
            Some("run `rstory daemon`"),
        ),
    ];
    for case in cases() {
        let theme = Theme::new(case.tier, Background::Dark);
        let glyphs = Glyphs::new(case.ascii);
        let ctx = Ctx::new(&theme, &glyphs);
        let lines: Vec<Line<'static>> = checks
            .iter()
            .flat_map(|&(status, label, message, hint)| {
                CheckLine {
                    status,
                    label,
                    message,
                    hint,
                }
                .lines(ctx)
            })
            .collect();
        snap("check_line", &case, lines_dump(&case, lines));
    }
}
