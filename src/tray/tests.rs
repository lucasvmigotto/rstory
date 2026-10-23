//! Menu-model tests: labels, truncation, order, every state (SC-002).

use time::OffsetDateTime;
use time::macros::datetime;

use super::menu::{
    Action, Clear, EntryRow, Item, LABEL_MAX, MAX_ENTRIES, PauseItem, TrayMenu, View, label_of,
};
use crate::ui::copy;

const T0: OffsetDateTime = datetime!(2026-10-21 14:00:00 UTC);

fn rows(n: usize) -> Vec<EntryRow> {
    (0..n)
        .map(|i| EntryRow {
            id: format!("ID{i:02}"),
            preview: format!("synthetic entry {i}"),
            pinned: false,
        })
        .collect()
}

fn build(n: usize, view: &View, picker: bool, clear: Clear) -> TrayMenu {
    TrayMenu::build(&rows(n), view, T0, picker, clear)
}

fn labels(items: &[Item]) -> Vec<String> {
    items
        .iter()
        .map(|i| match i {
            Item::Action { label, .. } | Item::Submenu { label, .. } => label.clone(),
            Item::Label(l) => format!("[{l}]"),
            Item::Separator => "---".to_string(),
        })
        .collect()
}

#[test]
fn twelve_entries_give_ten_items_in_order_and_the_third_restores_the_third() {
    let m = build(12, &View::Recording, true, Clear::Idle);
    assert_eq!(m.entries.len(), MAX_ENTRIES);
    let items = m.items();
    for (i, item) in items.iter().take(10).enumerate() {
        let Item::Action { label, action } = item else {
            panic!("item {i} is not an action")
        };
        assert_eq!(label, &format!("synthetic entry {i}"));
        assert_eq!(action, &Action::Restore(format!("ID{i:02}")));
    }
    assert_eq!(
        items[2],
        Item::Action {
            label: "synthetic entry 2".into(),
            action: Action::Restore("ID02".into())
        }
    );
}

#[test]
fn labels_escape_newlines_truncate_and_drop_control_characters() {
    assert_eq!(label_of("one\\ntwo"), "one⏎two");
    assert_eq!(label_of("a\nb\r\nc"), "a⏎b⏎c");
    assert_eq!(label_of("a\u{7}b\\x07c"), "ab\\x07c");
    let long = "x".repeat(100);
    let l = label_of(&long);
    assert_eq!(l.chars().count(), LABEL_MAX);
    assert!(l.ends_with('…'));
    assert_eq!(label_of(&"y".repeat(LABEL_MAX)), "y".repeat(LABEL_MAX));
    assert!(!label_of("é".repeat(60).as_str()).contains('\u{fffd}'));
}

#[test]
fn pinned_entries_carry_the_pin_glyph() {
    let mut r = rows(2);
    r[1].pinned = true;
    let m = TrayMenu::build(&r, &View::Recording, T0, false, Clear::Idle);
    assert_eq!(m.entries[0].1, "synthetic entry 0");
    assert_eq!(m.entries[1].1, "📌 synthetic entry 1");
    assert!(m.entries[1].2);
}

#[test]
fn recording_menu_has_the_contract_order() {
    let m = build(2, &View::Recording, true, Clear::Idle);
    assert_eq!(
        labels(&m.items()),
        [
            "synthetic entry 0",
            "synthetic entry 1",
            "---",
            "[Recording]",
            "Pause recording",
            "Pause for 1 minute",
            "Open picker",
            "---",
            "Clear history…",
        ]
    );
}

#[test]
fn paused_shows_resume_and_hides_the_timed_pause() {
    let m = build(1, &View::Paused { until: None }, true, Clear::Idle);
    assert_eq!(m.pause_item, PauseItem::Resume);
    assert!(!m.show_pause_minute);
    let l = labels(&m.items());
    assert!(l.contains(&"[Paused]".to_string()));
    assert!(l.contains(&"Resume recording".to_string()));
    assert!(!l.contains(&"Pause for 1 minute".to_string()));
    assert!(!l.contains(&"Pause recording".to_string()));
}

#[test]
fn a_timed_pause_shows_the_remaining_time() {
    let until = T0 + std::time::Duration::from_secs(42);
    let m = build(0, &View::Paused { until: Some(until) }, true, Clear::Idle);
    assert_eq!(m.status_label, "Paused — resumes in 0:42");
    let until = T0 + std::time::Duration::from_secs(60);
    let m = build(0, &View::Paused { until: Some(until) }, true, Clear::Idle);
    assert_eq!(m.status_label, "Paused — resumes in 1:00");
    let until = T0 + std::time::Duration::from_millis(400);
    let m = build(0, &View::Paused { until: Some(until) }, true, Clear::Idle);
    assert_eq!(m.status_label, "Paused — resumes in 0:01", "never 0:00 while paused");
    let past = T0 - std::time::Duration::from_secs(5);
    let m = build(0, &View::Paused { until: Some(past) }, true, Clear::Idle);
    assert_eq!(m.status_label, "Paused — resumes in 0:01");
}

#[test]
fn inactive_says_why() {
    let v = View::Inactive {
        reason: "no_session".into(),
    };
    let m = build(0, &v, false, Clear::Idle);
    assert_eq!(m.status_label, "Inactive — no_session");
}

#[test]
fn no_terminal_hides_open_picker() {
    let l = labels(&build(0, &View::Recording, false, Clear::Idle).items());
    assert!(!l.contains(&"Open picker".to_string()));
}

#[test]
fn an_empty_history_says_so() {
    let items = build(0, &View::Recording, true, Clear::Idle).items();
    assert_eq!(items[0], Item::Label(copy::MENU_EMPTY.to_string()));
}

#[test]
fn clear_goes_through_a_confirm_submenu() {
    let idle = build(1, &View::Recording, true, Clear::Idle).items();
    assert_eq!(
        idle.last(),
        Some(&Item::Action {
            label: "Clear history…".into(),
            action: Action::ClearBegin
        })
    );
    let confirming = build(1, &View::Recording, true, Clear::Confirming(7)).items();
    let Some(Item::Submenu { label, children }) = confirming.last() else {
        panic!("clear is not a submenu while confirming")
    };
    assert_eq!(label, "Clear history…");
    assert_eq!(
        children,
        &[
            Item::Action {
                label: "Delete 7 unpinned entries".into(),
                action: Action::ClearConfirm
            },
            Item::Action {
                label: "Cancel".into(),
                action: Action::ClearCancel
            },
        ]
    );
    let n_confirm = confirming
        .iter()
        .filter(|i| matches!(i, Item::Action { action: Action::ClearConfirm, .. }))
        .count();
    assert_eq!(n_confirm, 0, "only the submenu child sends clear");
}

#[test]
fn every_actionable_item_has_a_text_label() {
    for view in [
        View::Recording,
        View::Paused { until: None },
        View::Inactive { reason: "x".into() },
    ] {
        for clear in [Clear::Idle, Clear::Confirming(3)] {
            fn walk(items: &[Item]) {
                for i in items {
                    match i {
                        Item::Action { label, .. } => assert!(!label.trim().is_empty()),
                        Item::Submenu { label, children } => {
                            assert!(!label.trim().is_empty());
                            walk(children);
                        }
                        Item::Label(l) => assert!(!l.trim().is_empty()),
                        Item::Separator => {}
                    }
                }
            }
            walk(&build(3, &view, true, clear).items());
        }
    }
}
