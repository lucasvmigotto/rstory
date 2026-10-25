//! Keyed copy constants; the keys are ux-vision's "Key-screen copy" table.

pub const PICKER_EMPTY: &str = "No entries yet. Copy something and it will appear here.";
pub const PICKER_READONLY_BANNER: &str = "Read-only: daemon not running";
pub const PICKER_READONLY_RESTORE: &str =
    "Daemon not running. Start it with `rstory daemon` or run `rstory install-autostart`.";
pub const PICKER_HINTS_FULL: &str =
    "Enter restore  Tab preview  Ctrl-T pin  Ctrl-D delete  Esc quit";
pub const PICKER_HINTS_SHORT: &str = "Enter restore  Tab preview  Esc quit";
pub const PICKER_CONFIRM_DELETE: &str = "Delete this entry? y/n";
pub const PICKER_PIN_LIMIT: &str = "Pin limit reached ({max}). Unpin an entry first.";
pub const PICKER_RESTORING: &str = "Restoring…";
pub const PICKER_TOO_SMALL: &str = "Terminal too small (need 20×5).";
pub const PICKER_COUNT: &str = "{shown}/{total}";
pub const STATE_RECORDING: &str = "Recording";
pub const STATE_PAUSED: &str = "Paused";
pub const STATE_PAUSED_UNTIL: &str = "Paused — resumes in {m}:{ss}";
pub const STATE_INACTIVE: &str = "Inactive — {reason}";
pub const MENU_EMPTY: &str = "No entries yet";
pub const MENU_PAUSE: &str = "Pause recording";
pub const MENU_RESUME: &str = "Resume recording";
pub const MENU_PAUSE_MINUTE: &str = "Pause for 1 minute";
pub const MENU_OPEN_PICKER: &str = "Open picker";
pub const MENU_CLEAR: &str = "Clear history…";
pub const MENU_CLEAR_CONFIRM: &str = "Delete {n} unpinned entries";
pub const MENU_CLEAR_CANCEL: &str = "Cancel";
pub const MENU_DISCONNECTED: &str = "rstory daemon not running";
pub const CLI_DAEMON_DOWN: &str =
    "Daemon not running. Start it with `rstory daemon` or run `rstory install-autostart`.";
pub const CLI_NOT_FOUND: &str = "No entry {ref}. Run `rstory ls` to see indexes and short ids.";
pub const CLI_AMBIGUOUS: &str = "Short id {ref} matches {n} entries: {ids}. Use more characters.";
pub const CLI_PIN_LIMIT: &str = "Pin limit reached ({max}). Unpin an entry first.";
pub const CLI_CLEAR_PROMPT: &str = "Delete {n} unpinned entries? This can't be undone. [y/N]";
pub const CLI_CLEAR_ALL_PROMPT: &str =
    "Delete ALL {n} entries, including {m} pinned? This can't be undone. [y/N]";
pub const CLI_CLEAR_DONE: &str = "Removed {n} entries.";
pub const CLI_NEEDS_YES: &str = "Refusing to clear without a terminal. Add `--yes`.";
pub const CLI_INACTIVE: &str =
    "No clipboard access in this session. Run `rstory doctor` for details.";
pub const CLI_READONLY: &str = "(daemon not running, read-only)";
pub const INSTALL_AUTOSTART_DONE: &str =
    "Autostart installed: {path}. rstory starts at your next login, or run `rstory daemon` now.";
pub const INSTALL_GNOME_DONE: &str = "GNOME extension installed. Log out and back in, then run: gnome-extensions enable rstory@lucasvmigotto.me";
pub const DOCTOR_ALL_OK: &str = "Everything looks good.";

/// Every key with its text, in ux-vision order.
pub const ALL: &[(&str, &str)] = &[
    ("picker.empty", PICKER_EMPTY),
    ("picker.readonly.banner", PICKER_READONLY_BANNER),
    ("picker.readonly.restore", PICKER_READONLY_RESTORE),
    ("picker.hints.full", PICKER_HINTS_FULL),
    ("picker.hints.short", PICKER_HINTS_SHORT),
    ("picker.confirm.delete", PICKER_CONFIRM_DELETE),
    ("picker.pin.limit", PICKER_PIN_LIMIT),
    ("picker.restoring", PICKER_RESTORING),
    ("picker.too_small", PICKER_TOO_SMALL),
    ("picker.count", PICKER_COUNT),
    ("state.recording", STATE_RECORDING),
    ("state.paused", STATE_PAUSED),
    ("state.paused_until", STATE_PAUSED_UNTIL),
    ("state.inactive", STATE_INACTIVE),
    ("menu.empty", MENU_EMPTY),
    ("menu.pause", MENU_PAUSE),
    ("menu.resume", MENU_RESUME),
    ("menu.pause_minute", MENU_PAUSE_MINUTE),
    ("menu.open_picker", MENU_OPEN_PICKER),
    ("menu.clear", MENU_CLEAR),
    ("menu.clear.confirm", MENU_CLEAR_CONFIRM),
    ("menu.clear.cancel", MENU_CLEAR_CANCEL),
    ("menu.disconnected", MENU_DISCONNECTED),
    ("cli.daemon_down", CLI_DAEMON_DOWN),
    ("cli.not_found", CLI_NOT_FOUND),
    ("cli.ambiguous", CLI_AMBIGUOUS),
    ("cli.pin_limit", CLI_PIN_LIMIT),
    ("cli.clear.prompt", CLI_CLEAR_PROMPT),
    ("cli.clear_all.prompt", CLI_CLEAR_ALL_PROMPT),
    ("cli.clear.done", CLI_CLEAR_DONE),
    ("cli.needs_yes", CLI_NEEDS_YES),
    ("cli.inactive", CLI_INACTIVE),
    ("cli.readonly", CLI_READONLY),
    ("install.autostart.done", INSTALL_AUTOSTART_DONE),
    ("install.gnome.done", INSTALL_GNOME_DONE),
    ("doctor.all_ok", DOCTOR_ALL_OK),
];

/// Fills `{name}` slots; a slot with no argument is left as written.
pub fn format(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (name, value) in args {
        out = out.replace(&["{", name, "}"].concat(), value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rows of the "Key-screen copy" table in ux-vision: (keys, text).
    fn doc_rows() -> Vec<(Vec<String>, Vec<String>)> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/product/ux-vision.md");
        let doc = std::fs::read_to_string(path).expect("ux-vision.md");
        let section = doc
            .split("## Key-screen copy")
            .nth(1)
            .expect("section")
            .split("\n## ")
            .next()
            .expect("section end");
        let mut rows = Vec::new();
        for line in section.lines().filter(|l| l.starts_with('|')) {
            let cells: Vec<&str> = line.trim_matches('|').splitn(2, " | ").collect();
            if cells.len() != 2 {
                continue;
            }
            let (k, t) = (
                cells[0].trim(),
                cells[1].trim().trim_end_matches('|').trim(),
            );
            if k == "Key" || k.starts_with("---") {
                continue;
            }
            let keys: Vec<String> = k
                .split(" / ")
                .map(|s| s.trim().trim_matches('`').to_string())
                .collect();
            let texts: Vec<String> = if keys.len() > 1 {
                t.split(" / ").map(str::to_string).collect()
            } else {
                vec![t.to_string()]
            };
            rows.push((keys, texts));
        }
        rows
    }

    #[test]
    fn every_ux_vision_key_exists_with_identical_text() {
        let rows = doc_rows();
        assert!(rows.len() >= 30, "parsed only {} rows", rows.len());
        let mut doc_keys = Vec::new();
        for (keys, texts) in rows {
            assert_eq!(keys.len(), texts.len(), "{keys:?}");
            for (k, t) in keys.iter().zip(&texts) {
                let found = ALL.iter().find(|(key, _)| key == k);
                let (_, text) = found.unwrap_or_else(|| panic!("missing copy key {k}"));
                assert_eq!(text, t, "text of {k}");
                doc_keys.push(k.clone());
            }
        }
        for (key, _) in ALL {
            assert!(
                doc_keys.iter().any(|k| k == key),
                "{key} is not in ux-vision"
            );
        }
    }

    #[test]
    fn keys_are_unique_and_texts_non_empty() {
        for (i, (k, t)) in ALL.iter().enumerate() {
            assert!(!t.is_empty(), "{k}");
            assert!(ALL[i + 1..].iter().all(|(o, _)| o != k), "duplicate {k}");
        }
    }

    #[test]
    fn constants_are_the_table_entries() {
        let get = |k: &str| ALL.iter().find(|(key, _)| *key == k).map(|(_, t)| *t);
        assert_eq!(get("picker.hints.full"), Some(PICKER_HINTS_FULL));
        assert_eq!(get("picker.readonly.banner"), Some(PICKER_READONLY_BANNER));
        assert_eq!(get("state.paused_until"), Some(STATE_PAUSED_UNTIL));
        assert_eq!(get("menu.resume"), Some(MENU_RESUME));
        assert_eq!(get("doctor.all_ok"), Some(DOCTOR_ALL_OK));
    }

    #[test]
    fn format_fills_slots() {
        assert_eq!(
            format(PICKER_COUNT, &[("shown", "2"), ("total", "14")]),
            "2/14"
        );
        assert_eq!(
            format(CLI_NOT_FOUND, &[("ref", "a1b2")]),
            "No entry a1b2. Run `rstory ls` to see indexes and short ids."
        );
        assert_eq!(
            format("{n} and {n} and {x}", &[("n", "1")]),
            "1 and 1 and {x}"
        );
    }
}
