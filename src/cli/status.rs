//! `rstory status` and `rstory config` output.

use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};
use time::OffsetDateTime;

use super::render::Term;
use crate::config::Caps;
use crate::ipc::protocol::{StateKind, Status};
use crate::ui::widgets::RecState;

/// The three lines of `rstory status`, each ending in a newline.
#[must_use]
pub fn status_text(status: &Status, now: OffsetDateTime, term: &Term) -> String {
    let sep = if term.is_ascii() { " - " } else { " · " };
    let state = match status.state.state {
        StateKind::Recording => RecState::Recording,
        StateKind::Paused => match status.state.paused_until {
            Some(until) => RecState::PausedUntil {
                secs: u64::try_from((until - now).whole_seconds()).unwrap_or(0),
            },
            None => RecState::Paused,
        },
        StateKind::Inactive => RecState::Inactive {
            reason: status
                .state
                .reason
                .clone()
                .unwrap_or_else(|| "unknown".into()),
        },
    };
    let backend = match (status.backend.as_str(), status.mode.as_deref()) {
        ("none", _) => "no capture backend".to_string(),
        (b, Some("xwayland_bridge")) => format!("backend {b} (XWayland bridge)"),
        (b, _) => format!("backend {b}"),
    };
    let noun = if status.entries == 1 {
        "entry"
    } else {
        "entries"
    };
    let c = &status.counters;
    let tray = match status.tray.as_str() {
        "no_host" => "no host",
        "no_bus" => "no bus",
        other => other,
    };
    let picker = if status.picker_available {
        "available"
    } else {
        "not available"
    };
    let lines = [
        format!(
            "{}{sep}{backend}{sep}{} {noun} ({} pinned) of {}",
            state.label(),
            status.entries,
            status.pinned,
            status.max_entries
        ),
        format!(
            "Skipped: {} sensitive{sep}{} oversize{sep}{} while paused",
            c.skipped_sensitive, c.skipped_oversize, c.skipped_paused
        ),
        format!("Tray: {tray}{sep}Picker: {picker}"),
    ];
    lines
        .iter()
        .map(|l| format!("{}\n", term.text(l)))
        .collect()
}

/// `{"schema":1, …Status}` and a newline.
#[must_use]
pub fn status_json(status: &Status) -> String {
    let mut value = serde_json::to_value(status).unwrap_or(Value::Null);
    if let Value::Object(map) = &mut value {
        map.insert("schema".into(), json!(1));
    }
    format!("{value}\n")
}

/// A time to live in the largest exact unit: `7d`, `36h`, `90m`.
#[must_use]
pub fn ttl_text(ttl: Duration) -> String {
    let s = ttl.as_secs();
    if s.is_multiple_of(86_400) {
        format!("{}d", s / 86_400)
    } else if s.is_multiple_of(3600) {
        format!("{}h", s / 3600)
    } else {
        format!("{}m", s / 60)
    }
}

/// The non-cap keys `rstory config` shows.
#[derive(Debug, Clone, Copy)]
pub struct ConfigView<'a> {
    /// The effective `theme`.
    pub theme: &'a str,
    /// `backend`, shown only when set.
    pub backend: Option<&'a str>,
    /// `picker_command`, shown only when set.
    pub picker_command: Option<&'a str>,
}

impl Default for ConfigView<'_> {
    fn default() -> Self {
        Self {
            theme: "terminal",
            backend: None,
            picker_command: None,
        }
    }
}

/// The effective configuration as TOML, headed by the config file's path.
#[must_use]
pub fn config_text(path: &Path, exists: bool, caps: &Caps, view: &ConfigView<'_>) -> String {
    let note = if exists {
        ""
    } else {
        " (not found, showing the defaults)"
    };
    let mut out = format!(
        "# {}{note}\nmax_entries = {}\nmax_pinned = {}\nmax_entry_bytes = {}\nttl = \"{}\"\n",
        path.display(),
        caps.max_entries,
        caps.max_pinned,
        caps.max_entry_bytes,
        ttl_text(caps.ttl)
    );
    out.push_str(&format!("theme = \"{}\"\n", view.theme));
    if let Some(backend) = view.backend {
        out.push_str(&format!("backend = \"{backend}\"\n"));
    }
    if let Some(cmd) = view.picker_command {
        out.push_str(&format!(
            "picker_command = {}\n",
            toml::Value::String(cmd.into())
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::ipc::protocol::{Counters, State, StateKind, Status};
    use crate::ui::theme::Tier;

    const NOW: time::OffsetDateTime = datetime!(2026-10-21 14:00:00 UTC);

    fn status() -> Status {
        Status {
            state: State {
                state: StateKind::Recording,
                paused_until: None,
                reason: None,
            },
            backend: "x11".into(),
            mode: Some("xwayland_bridge".into()),
            entries: 8,
            pinned: 1,
            max_entries: 20,
            max_pinned: 10,
            counters: Counters {
                skipped_sensitive: 2,
                skipped_oversize: 0,
                skipped_paused: 5,
                dropped_burst: 0,
            },
            tray: "no_host".into(),
            picker_available: true,
            version: "0.1.0".into(),
            protocol: 1,
        }
    }

    fn utf8() -> Term {
        Term::new(true, 80, Tier::Mono, false)
    }

    #[test]
    fn recording_status_matches_the_design() {
        insta::assert_snapshot!(status_text(&status(), NOW, &utf8()), @r"
        Recording · backend x11 (XWayland bridge) · 8 entries (1 pinned) of 20
        Skipped: 2 sensitive · 0 oversize · 5 while paused
        Tray: no host · Picker: available
        ");
    }

    #[test]
    fn ascii_mode_has_no_special_characters() {
        let text = status_text(&status(), NOW, &Term::new(true, 80, Tier::Mono, true));
        assert!(text.is_ascii(), "{text}");
        assert!(text.starts_with("Recording - backend x11"), "{text}");
    }

    #[test]
    fn a_timed_pause_shows_the_time_left() {
        let mut s = status();
        s.state.state = StateKind::Paused;
        s.state.paused_until = Some(NOW + time::Duration::seconds(65));
        let text = status_text(&s, NOW, &utf8());
        assert!(
            text.starts_with("Paused — resumes in 1:05 · backend x11"),
            "{text}"
        );
        s.state.paused_until = Some(NOW - time::Duration::seconds(3));
        assert!(status_text(&s, NOW, &utf8()).starts_with("Paused — resumes in 0:00"));
    }

    #[test]
    fn an_untimed_pause_says_paused() {
        let mut s = status();
        s.state.state = StateKind::Paused;
        assert!(status_text(&s, NOW, &utf8()).starts_with("Paused · backend"));
    }

    #[test]
    fn inactive_names_the_reason_and_no_backend() {
        let mut s = status();
        s.state.state = StateKind::Inactive;
        s.state.reason = Some("no_session".into());
        s.backend = "none".into();
        s.mode = None;
        s.entries = 0;
        s.pinned = 0;
        s.picker_available = false;
        s.tray = "disabled".into();
        insta::assert_snapshot!(status_text(&s, NOW, &utf8()), @r"
        Inactive — no_session · no capture backend · 0 entries (0 pinned) of 20
        Skipped: 2 sensitive · 0 oversize · 5 while paused
        Tray: disabled · Picker: not available
        ");
    }

    #[test]
    fn one_entry_is_singular() {
        let mut s = status();
        s.entries = 1;
        s.pinned = 0;
        assert!(status_text(&s, NOW, &utf8()).contains("1 entry (0 pinned)"));
    }

    #[test]
    fn status_json_is_schema_1_plus_the_status() {
        let v: serde_json::Value = serde_json::from_str(&status_json(&status())).unwrap();
        assert_eq!(v["schema"], 1);
        assert_eq!(v["state"], "recording");
        assert_eq!(v["backend"], "x11");
        assert_eq!(v["counters"]["skipped_paused"], 5);
        assert_eq!(v["protocol"], 1);
    }

    #[test]
    fn config_text_shows_the_effective_values_and_where_the_file_is() {
        let caps = crate::config::Caps::from_toml("max_entries = 50\nttl = \"36h\"").unwrap();
        let path = std::path::Path::new("/home/u/.config/rstory/config.toml");
        insta::assert_snapshot!(config_text(
            path,
            true,
            &caps,
            &ConfigView {
                theme: "rstory",
                backend: Some("wayland"),
                picker_command: Some("ghostty -e"),
            },
        ), @r#"
        # /home/u/.config/rstory/config.toml
        max_entries = 50
        max_pinned = 10
        max_entry_bytes = 1048576
        ttl = "36h"
        theme = "rstory"
        backend = "wayland"
        picker_command = "ghostty -e"
        "#);
        let defaults = crate::config::Caps::default();
        insta::assert_snapshot!(config_text(path, false, &defaults, &ConfigView::default()), @r#"
        # /home/u/.config/rstory/config.toml (not found, showing the defaults)
        max_entries = 20
        max_pinned = 10
        max_entry_bytes = 1048576
        ttl = "7d"
        theme = "terminal"
        "#);
    }

    #[test]
    fn ttl_uses_the_largest_exact_unit() {
        let t = |s| ttl_text(std::time::Duration::from_secs(s));
        assert_eq!(t(3600), "1h");
        assert_eq!(t(90 * 60), "90m");
        assert_eq!(t(36 * 3600), "36h");
        assert_eq!(t(48 * 3600), "2d");
        assert_eq!(t(7 * 86_400), "7d");
    }
}
