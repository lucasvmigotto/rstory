//! Keyed copy constants; the keys are ux-vision's "Key-screen copy" table.

pub const PICKER_EMPTY: &str = "";
pub const PICKER_READONLY_BANNER: &str = "";
pub const PICKER_READONLY_RESTORE: &str = "";
pub const PICKER_HINTS_FULL: &str = "";
pub const PICKER_HINTS_SHORT: &str = "";
pub const PICKER_CONFIRM_DELETE: &str = "";
pub const PICKER_PIN_LIMIT: &str = "";
pub const PICKER_RESTORING: &str = "";
pub const PICKER_TOO_SMALL: &str = "";
pub const PICKER_COUNT: &str = "";
pub const STATE_RECORDING: &str = "";
pub const STATE_PAUSED: &str = "";
pub const STATE_PAUSED_UNTIL: &str = "";
pub const STATE_INACTIVE: &str = "";
pub const MENU_EMPTY: &str = "";
pub const MENU_PAUSE: &str = "";
pub const MENU_RESUME: &str = "";
pub const MENU_PAUSE_MINUTE: &str = "";
pub const MENU_OPEN_PICKER: &str = "";
pub const MENU_CLEAR: &str = "";
pub const MENU_CLEAR_CONFIRM: &str = "";
pub const MENU_CLEAR_CANCEL: &str = "";
pub const MENU_DISCONNECTED: &str = "";
pub const CLI_DAEMON_DOWN: &str = "";
pub const CLI_NOT_FOUND: &str = "";
pub const CLI_AMBIGUOUS: &str = "";
pub const CLI_PIN_LIMIT: &str = "";
pub const CLI_CLEAR_PROMPT: &str = "";
pub const CLI_CLEAR_ALL_PROMPT: &str = "";
pub const CLI_CLEAR_DONE: &str = "";
pub const CLI_NEEDS_YES: &str = "";
pub const CLI_INACTIVE: &str = "";
pub const CLI_READONLY: &str = "";
pub const INSTALL_AUTOSTART_DONE: &str = "";
pub const INSTALL_GNOME_DONE: &str = "";
pub const DOCTOR_ALL_OK: &str = "";

/// Every key with its text, in ux-vision order.
pub const ALL: &[(&str, &str)] = &[];

/// Fills `{name}` slots; a slot with no argument is left as written.
pub fn format(_template: &str, _args: &[(&str, &str)]) -> String {
    todo!()
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
