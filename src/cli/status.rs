//! `rstory status` and `rstory config` output.

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
        insta::assert_snapshot!(config_text(path, true, &caps, Some("ghostty -e")), @r#"
        # /home/u/.config/rstory/config.toml
        max_entries = 50
        max_pinned = 10
        max_entry_bytes = 1048576
        ttl = "36h"
        picker_command = "ghostty -e"
        "#);
        let defaults = crate::config::Caps::default();
        insta::assert_snapshot!(config_text(path, false, &defaults, None), @r#"
        # /home/u/.config/rstory/config.toml (not found, showing the defaults)
        max_entries = 20
        max_pinned = 10
        max_entry_bytes = 1048576
        ttl = "7d"
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
