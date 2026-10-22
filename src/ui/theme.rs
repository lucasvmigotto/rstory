//! Color tiers and semantic roles (`specs/000-design-system/tokens.md`).

use std::str::FromStr;

use ratatui::style::{Color, Modifier, Style};

/// How much color the terminal gets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Mono,
    Ansi16,
    Ansi256,
    TrueColor,
}

impl Tier {
    pub const ALL: [Tier; 4] = [Tier::Ansi16, Tier::Ansi256, Tier::TrueColor, Tier::Mono];

    /// Stable name used in snapshot names.
    pub fn name(self) -> &'static str {
        match self {
            Tier::Mono => "mono",
            Tier::Ansi16 => "16",
            Tier::Ansi256 => "256",
            Tier::TrueColor => "truecolor",
        }
    }
}

/// The config `theme` key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemePref {
    #[default]
    Terminal,
    Rstory,
    Mono,
}

/// Returned by [`ThemePref::from_str`] for an unknown value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownTheme;

impl FromStr for ThemePref {
    type Err = UnknownTheme;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "terminal" => Ok(Self::Terminal),
            "rstory" => Ok(Self::Rstory),
            "mono" => Ok(Self::Mono),
            _ => Err(UnknownTheme),
        }
    }
}

/// Terminal background, which picks the truecolor variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Background {
    #[default]
    Dark,
    Light,
}

/// Semantic roles; widgets never name a color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Accent,
    Selected,
    Muted,
    Pinned,
    Warning,
    Danger,
}

impl Role {
    pub const ALL: [Role; 6] = [
        Role::Accent,
        Role::Selected,
        Role::Muted,
        Role::Pinned,
        Role::Warning,
        Role::Danger,
    ];
}

/// The environment variables theme detection reads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TermEnv {
    /// `NO_COLOR` is set and not empty.
    pub no_color: bool,
    pub term: Option<String>,
    pub colorterm: Option<String>,
}

impl TermEnv {
    pub fn from_process() -> Self {
        Self::from_vars(|k| std::env::var(k).ok())
    }

    pub fn from_vars(get: impl Fn(&str) -> Option<String>) -> Self {
        Self {
            no_color: get("NO_COLOR").is_some_and(|v| !v.is_empty()),
            term: get("TERM"),
            colorterm: get("COLORTERM"),
        }
    }
}

/// Picks the tier from the config preference and the environment.
///
/// `NO_COLOR`, `TERM=dumb` and `theme = "mono"` give [`Tier::Mono`]. The default
/// `terminal` theme is the 16-color tier. `rstory` opts in to truecolor when
/// `COLORTERM` says so, else 256 colors when `TERM` names them.
pub fn detect_tier(pref: ThemePref, env: &TermEnv) -> Tier {
    let term = env.term.as_deref().unwrap_or("");
    if env.no_color || term == "dumb" || pref == ThemePref::Mono {
        return Tier::Mono;
    }
    if pref == ThemePref::Terminal {
        return Tier::Ansi16;
    }
    let colorterm = env.colorterm.as_deref().unwrap_or("").to_ascii_lowercase();
    if colorterm == "truecolor" || colorterm == "24bit" {
        Tier::TrueColor
    } else if term.contains("256color") {
        Tier::Ansi256
    } else {
        Tier::Ansi16
    }
}

/// A tier plus a background: everything needed to style a role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    tier: Tier,
    background: Background,
}

impl Theme {
    pub fn new(tier: Tier, background: Background) -> Self {
        Self { tier, background }
    }

    pub fn detect(pref: ThemePref, env: &TermEnv, background: Background) -> Self {
        Self::new(detect_tier(pref, env), background)
    }

    pub fn tier(&self) -> Tier {
        self.tier
    }

    pub fn style(&self, role: Role) -> Style {
        match role {
            Role::Selected => return Style::new().add_modifier(Modifier::REVERSED),
            Role::Accent if self.tier == Tier::Mono => {
                return Style::new().add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
            }
            _ if self.tier == Tier::Mono => return Style::new(),
            _ => {}
        }
        let dark = self.background == Background::Dark;
        let rgb = |d: (u8, u8, u8), l: (u8, u8, u8)| {
            let (r, g, b) = if dark { d } else { l };
            Color::Rgb(r, g, b)
        };
        let color = match (role, self.tier) {
            (Role::Accent, Tier::Ansi16) => Color::Cyan,
            (Role::Accent, Tier::Ansi256) => Color::Indexed(37),
            (Role::Accent, _) => rgb((0x2A, 0xA1, 0x98), (0x1F, 0x7A, 0x73)),
            (Role::Muted, Tier::Ansi16) => Color::DarkGray,
            (Role::Muted, Tier::Ansi256) => Color::Indexed(245),
            (Role::Muted, _) => rgb((0x8A, 0x8A, 0x8A), (0x6C, 0x6C, 0x6C)),
            (Role::Pinned | Role::Warning, Tier::Ansi16) => Color::Yellow,
            (Role::Pinned | Role::Warning, Tier::Ansi256) => Color::Indexed(178),
            (Role::Pinned | Role::Warning, _) => rgb((0xD7, 0xAF, 0x00), (0x8A, 0x6D, 0x00)),
            (Role::Danger, Tier::Ansi16) => Color::Red,
            (Role::Danger, Tier::Ansi256) => Color::Indexed(160),
            (Role::Danger, _) => rgb((0xD7, 0x00, 0x00), (0xAF, 0x00, 0x00)),
            (Role::Selected, _) => unreachable!("handled above"),
        };
        Style::new().fg(color)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Modifier};

    use super::*;

    fn env(no_color: bool, term: &str, colorterm: &str) -> TermEnv {
        TermEnv {
            no_color,
            term: (!term.is_empty()).then(|| term.to_string()),
            colorterm: (!colorterm.is_empty()).then(|| colorterm.to_string()),
        }
    }

    #[test]
    fn no_color_and_dumb_terminals_are_mono() {
        for pref in [ThemePref::Terminal, ThemePref::Rstory, ThemePref::Mono] {
            assert_eq!(
                detect_tier(pref, &env(true, "xterm-256color", "truecolor")),
                Tier::Mono
            );
            assert_eq!(
                detect_tier(pref, &env(false, "dumb", "truecolor")),
                Tier::Mono
            );
        }
    }

    #[test]
    fn mono_theme_is_mono_everywhere() {
        assert_eq!(
            detect_tier(ThemePref::Mono, &env(false, "xterm-256color", "truecolor")),
            Tier::Mono
        );
    }

    #[test]
    fn terminal_theme_is_the_16_color_tier_even_with_truecolor() {
        assert_eq!(
            detect_tier(
                ThemePref::Terminal,
                &env(false, "xterm-256color", "truecolor")
            ),
            Tier::Ansi16
        );
    }

    #[test]
    fn rstory_theme_needs_colorterm_for_truecolor() {
        for ct in ["truecolor", "24bit", "TrueColor"] {
            assert_eq!(
                detect_tier(ThemePref::Rstory, &env(false, "xterm", ct)),
                Tier::TrueColor
            );
        }
        assert_eq!(
            detect_tier(ThemePref::Rstory, &env(false, "xterm-256color", "")),
            Tier::Ansi256
        );
        assert_eq!(
            detect_tier(ThemePref::Rstory, &env(false, "xterm", "")),
            Tier::Ansi16
        );
        assert_eq!(
            detect_tier(ThemePref::Rstory, &env(false, "", "")),
            Tier::Ansi16
        );
    }

    #[test]
    fn env_reads_no_color_as_set_and_not_empty() {
        let get = |v: Option<&'static str>| {
            move |k: &str| match k {
                "NO_COLOR" => v.map(str::to_string),
                "TERM" => Some("xterm".to_string()),
                _ => None,
            }
        };
        assert!(TermEnv::from_vars(get(Some("1"))).no_color);
        assert!(!TermEnv::from_vars(get(Some(""))).no_color);
        assert!(!TermEnv::from_vars(get(None)).no_color);
        assert_eq!(TermEnv::from_vars(get(None)).term.as_deref(), Some("xterm"));
    }

    #[test]
    fn theme_pref_parses_the_config_values() {
        assert_eq!("terminal".parse(), Ok(ThemePref::Terminal));
        assert_eq!("rstory".parse(), Ok(ThemePref::Rstory));
        assert_eq!("mono".parse(), Ok(ThemePref::Mono));
        assert_eq!("neon".parse::<ThemePref>(), Err(UnknownTheme));
        assert_eq!(ThemePref::default(), ThemePref::Terminal);
    }

    fn fg(tier: Tier, bg: Background, role: Role) -> Option<Color> {
        Theme::new(tier, bg).style(role).fg
    }

    #[test]
    fn roles_by_tier_follow_tokens_md() {
        let d = Background::Dark;
        let l = Background::Light;
        let rows: [(Role, Color, Color, Color, Color); 5] = [
            (
                Role::Accent,
                Color::Cyan,
                Color::Indexed(37),
                Color::Rgb(0x2A, 0xA1, 0x98),
                Color::Rgb(0x1F, 0x7A, 0x73),
            ),
            (
                Role::Muted,
                Color::DarkGray,
                Color::Indexed(245),
                Color::Rgb(0x8A, 0x8A, 0x8A),
                Color::Rgb(0x6C, 0x6C, 0x6C),
            ),
            (
                Role::Pinned,
                Color::Yellow,
                Color::Indexed(178),
                Color::Rgb(0xD7, 0xAF, 0x00),
                Color::Rgb(0x8A, 0x6D, 0x00),
            ),
            (
                Role::Warning,
                Color::Yellow,
                Color::Indexed(178),
                Color::Rgb(0xD7, 0xAF, 0x00),
                Color::Rgb(0x8A, 0x6D, 0x00),
            ),
            (
                Role::Danger,
                Color::Red,
                Color::Indexed(160),
                Color::Rgb(0xD7, 0x00, 0x00),
                Color::Rgb(0xAF, 0x00, 0x00),
            ),
        ];
        for (role, c16, c256, tc_dark, tc_light) in rows {
            assert_eq!(fg(Tier::Ansi16, d, role), Some(c16), "{role:?}");
            assert_eq!(fg(Tier::Ansi256, d, role), Some(c256), "{role:?}");
            assert_eq!(fg(Tier::TrueColor, d, role), Some(tc_dark), "{role:?}");
            assert_eq!(fg(Tier::TrueColor, l, role), Some(tc_light), "{role:?}");
        }
    }

    #[test]
    fn selected_is_reverse_in_every_tier_and_carries_no_color() {
        for tier in Tier::ALL {
            let s = Theme::new(tier, Background::Dark).style(Role::Selected);
            assert!(s.add_modifier.contains(Modifier::REVERSED), "{tier:?}");
            assert_eq!(s.fg, None);
            assert_eq!(s.bg, None);
        }
    }

    #[test]
    fn mono_tier_has_no_colors_and_marks_accent_with_bold_underline() {
        let theme = Theme::new(Tier::Mono, Background::Dark);
        for role in Role::ALL {
            let s = theme.style(role);
            assert_eq!(s.fg, None, "{role:?}");
            assert_eq!(s.bg, None, "{role:?}");
        }
        let accent = theme.style(Role::Accent);
        assert!(
            accent
                .add_modifier
                .contains(Modifier::BOLD | Modifier::UNDERLINED)
        );
        for role in [Role::Muted, Role::Pinned, Role::Warning, Role::Danger] {
            assert_eq!(theme.style(role), Style::default(), "{role:?}");
        }
    }

    #[test]
    fn muted_never_uses_dim() {
        for tier in Tier::ALL {
            let s = Theme::new(tier, Background::Dark).style(Role::Muted);
            assert!(!s.add_modifier.contains(Modifier::DIM));
        }
    }

    #[test]
    fn detect_builds_a_theme_with_the_detected_tier() {
        let t = Theme::detect(
            ThemePref::Rstory,
            &env(false, "xterm", "truecolor"),
            Background::Light,
        );
        assert_eq!(t.tier(), Tier::TrueColor);
    }
}
