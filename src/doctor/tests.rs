//! Doctor tests: foundation (T001), healthy baseline and JSON (T002), failure table (T005),
//! snapshots (T011) and probes (T010).

use std::path::PathBuf;

use time::macros::datetime;

use super::checks::{
    DaemonFact, Facts, GnomeFacts, SessionFacts, SessionKind, StoreFact, evaluate,
};
use super::*;
use crate::backend::{BackendKind, Mode};
use crate::cli::render::Term;
use crate::domain::entry::PauseState;
use crate::install::autostart::AutostartKind;
use crate::install::gnome::{ExtensionInstall, UUID};
use crate::ipc::protocol::{Counters, State, StateKind, Status as DaemonStatus};
use crate::launch::{LaunchPlan, Source};
use crate::session::{BackendSelection, Reason};
use crate::ui::theme::Tier;

mod foundation {
    use super::*;

    fn c(status: Status) -> Check {
        Check {
            id: "x",
            status,
            message: "m".into(),
            hint: None,
        }
    }

    #[test]
    fn statuses_have_json_names_and_an_order() {
        let names: Vec<_> = [Status::Ok, Status::Info, Status::Warn, Status::Fail]
            .map(Status::as_str)
            .to_vec();
        assert_eq!(names, ["ok", "info", "warn", "fail"]);
        assert!(Status::Ok < Status::Info && Status::Info < Status::Warn);
        assert!(Status::Warn < Status::Fail);
    }

    #[test]
    fn exit_scale_is_zero_for_ok_and_info_one_for_warn_two_for_fail() {
        assert_eq!(exit_code(&[]), 0);
        assert_eq!(exit_code(&[c(Status::Ok), c(Status::Info)]), 0);
        assert_eq!(exit_code(&[c(Status::Ok), c(Status::Warn)]), 1);
        assert_eq!(
            exit_code(&[c(Status::Fail), c(Status::Warn), c(Status::Ok)]),
            2
        );
    }

    #[test]
    fn counts_warnings_and_failures_only() {
        let all = [
            c(Status::Ok),
            c(Status::Info),
            c(Status::Warn),
            c(Status::Warn),
            c(Status::Fail),
        ];
        assert_eq!(counts(&all), (2, 1));
    }

    #[test]
    fn constructors_set_status_and_hint() {
        let ok = Check::ok("session", "Wayland");
        assert_eq!((ok.id, ok.status, ok.hint), ("session", Status::Ok, None));
        let w = Check::with_hint("tray", Status::Warn, "m", "h");
        assert_eq!((w.status, w.hint.as_deref()), (Status::Warn, Some("h")));
    }
}

const NOW: time::OffsetDateTime = datetime!(2026-10-21 14:00:00 UTC);
const STORE: &str = "/home/u/.local/state/rstory";

fn daemon_status(state: StateKind, until_secs: Option<i64>, tray: &str) -> DaemonStatus {
    DaemonStatus {
        state: State {
            state,
            paused_until: until_secs.map(|s| NOW + time::Duration::seconds(s)),
            reason: None,
        },
        backend: "x11".into(),
        mode: Some("xwayland_bridge".into()),
        entries: 8,
        pinned: 1,
        max_entries: 20,
        max_pinned: 10,
        counters: Counters::default(),
        tray: tray.into(),
        picker_available: true,
        version: "0.1.0".into(),
        protocol: 1,
    }
}

/// A healthy GNOME 48 Wayland session with the bridge, the way spec scenario 1 describes it.
fn healthy() -> Facts {
    Facts {
        now: NOW,
        session: SessionFacts {
            kind: SessionKind::Wayland,
            selection: BackendSelection {
                backend: BackendKind::X11,
                mode: Mode::XwaylandBridge,
                reason: Reason::NoDataControl,
            },
        },
        store: StoreFact::Private { entries: Some(8) },
        daemon: DaemonFact::Up(Box::new(daemon_status(
            StateKind::Recording,
            None,
            "no_host",
        ))),
        store_pause: None,
        gnome: Some(GnomeFacts {
            shell_version: Some("48.7".into()),
            extension: Some(ExtensionInstall {
                path: PathBuf::from("/home/u/.local/share/gnome-shell/extensions").join(UUID),
                installed_version: Some("0.1.0".into()),
                enabled: Some(true),
            }),
            shipped_version: Some("0.1.0".into()),
        }),
        picker: LaunchPlan::Found {
            source: Source::List,
            argv: vec![
                "ghostty".into(),
                "-e".into(),
                "rstory".into(),
                "pick".into(),
            ],
        },
        autostart: vec![(
            AutostartKind::Xdg,
            PathBuf::from("/home/u/.config/autostart/rstory.desktop"),
        )],
    }
}

fn extension(f: &mut Facts) -> &mut ExtensionInstall {
    f.gnome.as_mut().unwrap().extension.as_mut().unwrap()
}

type Mutate = fn(&mut Facts);

struct Row {
    name: &'static str,
    mutate: Mutate,
    id: &'static str,
    status: Status,
    message: &'static str,
    hint: &'static str,
}

/// One row per failure in spec scenarios 1 to 8 (4 has three variants).
fn rows() -> Vec<Row> {
    vec![
        Row {
            name: "1_no_backend",
            mutate: |f| {
                f.session.selection = BackendSelection {
                    backend: BackendKind::None,
                    mode: Mode::None,
                    reason: Reason::NoDataControlNoXwayland,
                };
            },
            id: "backend",
            status: Status::Fail,
            message: "This compositor offers no clipboard-monitoring protocol and no XWayland",
            hint: "enable XWayland",
        },
        Row {
            name: "2_daemon_down",
            mutate: |f| f.daemon = DaemonFact::Down,
            id: "daemon",
            status: Status::Fail,
            message: "not running",
            hint: "run `rstory daemon` or `rstory install-autostart`",
        },
        Row {
            name: "3_store_too_open",
            mutate: |f| {
                f.store = StoreFact::TooOpen {
                    dir: PathBuf::from(STORE),
                }
            },
            id: "store",
            status: Status::Fail,
            message: "too open",
            hint: "chmod 700 /home/u/.local/state/rstory && chmod 600 /home/u/.local/state/rstory/*",
        },
        Row {
            name: "4a_extension_missing",
            mutate: |f| extension(f).installed_version = None,
            id: "gnome_extension",
            status: Status::Warn,
            message: "not installed",
            hint: "rstory install-gnome-extension",
        },
        Row {
            name: "4b_extension_disabled",
            mutate: |f| extension(f).enabled = Some(false),
            id: "gnome_extension",
            status: Status::Warn,
            message: "installed 0.1.0, not enabled",
            hint: "gnome-extensions enable rstory@lucasvmigotto.github.io",
        },
        Row {
            name: "4c_extension_outdated",
            mutate: |f| extension(f).installed_version = Some("0.0.9".into()),
            id: "gnome_extension",
            status: Status::Warn,
            message: "installed 0.0.9, this rstory ships 0.1.0",
            hint: "rstory install-gnome-extension",
        },
        Row {
            name: "5_shell_unsupported",
            mutate: |f| f.gnome.as_mut().unwrap().shell_version = Some("52.1".into()),
            id: "gnome_shell",
            status: Status::Warn,
            message: "panel icon not supported on GNOME Shell 52; CLI and picker work",
            hint: "GNOME Shell 48 to 51",
        },
        Row {
            name: "6_no_tray_host",
            mutate: |f| {
                f.gnome = None;
                f.daemon = DaemonFact::Up(Box::new(daemon_status(
                    StateKind::Recording,
                    None,
                    "no_host",
                )));
            },
            id: "tray",
            status: Status::Warn,
            message: "no StatusNotifierItem host: the tray is unavailable; CLI and picker work",
            hint: "StatusNotifierItem",
        },
        Row {
            name: "7_no_terminal",
            mutate: |f| {
                f.picker = LaunchPlan::NotFound {
                    searched: vec!["xdg-terminal-exec".into(), "ghostty".into(), "kitty".into()],
                }
            },
            id: "picker",
            status: Status::Warn,
            message: "searched: xdg-terminal-exec, ghostty, kitty",
            hint: "set picker_command in config.toml",
        },
        Row {
            name: "8_paused",
            mutate: |f| {
                f.daemon = DaemonFact::Up(Box::new(daemon_status(
                    StateKind::Paused,
                    Some(252),
                    "no_host",
                )));
            },
            id: "pause",
            status: Status::Info,
            message: "recording paused (resumes in 4m 12s)",
            hint: "rstory resume",
        },
    ]
}

fn find<'a>(checks: &'a [Check], id: &str) -> &'a Check {
    checks
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("no `{id}` check in {checks:?}"))
}

mod checks {
    use super::*;

    #[test]
    fn a_healthy_setup_is_all_ok_with_exit_zero() {
        let checks = evaluate(&healthy());
        let ids: Vec<_> = checks.iter().map(|c| c.id).collect();
        assert_eq!(
            ids,
            [
                "session",
                "backend",
                "store",
                "daemon",
                "gnome_extension",
                "picker",
                "autostart"
            ]
        );
        assert!(checks.iter().all(|c| c.status == Status::Ok), "{checks:?}");
        assert!(checks.iter().all(|c| c.hint.is_none()));
        assert_eq!(exit_code(&checks), 0);
        let msg = |id| find(&checks, id).message.as_str();
        assert_eq!(msg("session"), "Wayland (GNOME Shell 48.7)");
        assert_eq!(
            msg("backend"),
            "x11 via XWayland bridge (no data-control on this compositor)"
        );
        assert_eq!(msg("store"), "private (0700/0600), 8 entries");
        assert_eq!(msg("daemon"), "running, recording");
        assert_eq!(msg("gnome_extension"), "installed 0.1.0, enabled");
        assert_eq!(msg("picker"), "ghostty");
        assert_eq!(msg("autostart"), "/home/u/.config/autostart/rstory.desktop");
    }

    #[test]
    fn every_failure_row_has_its_status_message_and_hint() {
        for row in rows() {
            let mut f = healthy();
            (row.mutate)(&mut f);
            let checks = evaluate(&f);
            let c = find(&checks, row.id);
            assert_eq!(c.status, row.status, "{}: {c:?}", row.name);
            assert!(
                c.message.contains(row.message),
                "{}: message {:?} lacks {:?}",
                row.name,
                c.message,
                row.message
            );
            let hint = c.hint.as_deref().unwrap_or("");
            assert!(
                hint.contains(row.hint),
                "{}: hint {hint:?} lacks {:?}",
                row.name,
                row.hint
            );
        }
    }

    #[test]
    fn every_result_that_is_not_ok_has_a_hint() {
        for row in rows() {
            let mut f = healthy();
            (row.mutate)(&mut f);
            for c in evaluate(&f) {
                assert!(
                    c.status == Status::Ok || c.hint.is_some(),
                    "{}: {c:?}",
                    row.name
                );
            }
        }
    }

    #[test]
    fn the_exit_code_follows_the_worst_row() {
        for row in rows() {
            let mut f = healthy();
            (row.mutate)(&mut f);
            let want = match row.status {
                Status::Fail => 2,
                Status::Warn => 1,
                _ => 0,
            };
            assert_eq!(exit_code(&evaluate(&f)), want, "{}", row.name);
        }
    }

    #[test]
    fn no_session_is_reported_in_words() {
        let mut f = healthy();
        f.session = SessionFacts {
            kind: SessionKind::None,
            selection: BackendSelection {
                backend: BackendKind::None,
                mode: Mode::None,
                reason: Reason::NoSession,
            },
        };
        f.gnome = None;
        let checks = evaluate(&f);
        let s = find(&checks, "session");
        assert_eq!(s.status, Status::Fail);
        assert!(s.message.contains("no graphical session"), "{s:?}");
        assert_eq!(find(&checks, "backend").status, Status::Fail);
    }

    #[test]
    fn a_native_backend_and_an_override_are_described() {
        let mut f = healthy();
        f.session.selection = BackendSelection {
            backend: BackendKind::Wayland,
            mode: Mode::Native,
            reason: Reason::DataControl,
        };
        assert_eq!(
            find(&evaluate(&f), "backend").message,
            "wayland (data-control)"
        );
        f.session.selection = BackendSelection {
            backend: BackendKind::X11,
            mode: Mode::Native,
            reason: Reason::X11Session,
        };
        assert_eq!(find(&evaluate(&f), "backend").message, "x11");
    }

    #[test]
    fn a_daemon_that_does_not_answer_is_a_failure_with_a_hint() {
        let mut f = healthy();
        f.daemon = DaemonFact::NotAnswering;
        let checks = evaluate(&f);
        let d = find(&checks, "daemon");
        assert_eq!(d.status, Status::Fail);
        assert!(d.message.contains("not answering"), "{d:?}");
        assert!(d.hint.is_some());
        assert!(checks.iter().all(|c| c.id != "tray"), "tray unknown");
    }

    #[test]
    fn an_inactive_daemon_is_a_warning_naming_the_reason() {
        let mut f = healthy();
        let mut st = daemon_status(StateKind::Inactive, None, "no_host");
        st.state.reason = Some("no_data_control_no_xwayland".into());
        f.daemon = DaemonFact::Up(Box::new(st));
        let checks = evaluate(&f);
        let d = find(&checks, "daemon");
        assert_eq!(d.status, Status::Warn);
        assert!(d.message.contains("no_data_control_no_xwayland"), "{d:?}");
    }

    #[test]
    fn nonzero_counters_are_shown_zero_ones_are_not() {
        let mut f = healthy();
        let mut st = daemon_status(StateKind::Recording, None, "no_host");
        st.counters.skipped_sensitive = 3;
        f.daemon = DaemonFact::Up(Box::new(st));
        let msg = find(&evaluate(&f), "daemon").message.clone();
        assert_eq!(msg, "running, recording (skipped: 3 sensitive)");
    }

    #[test]
    fn the_pause_comes_from_the_store_when_there_is_no_daemon() {
        let mut f = healthy();
        f.daemon = DaemonFact::Down;
        f.store_pause = Some(PauseState::Paused { until: None });
        let checks = evaluate(&f);
        let p = find(&checks, "pause");
        assert_eq!(p.status, Status::Info);
        assert!(p.message.contains("until you resume"), "{p:?}");
        f.store_pause = Some(PauseState::Paused {
            until: Some(NOW - time::Duration::seconds(5)),
        });
        assert!(
            evaluate(&f).iter().all(|c| c.id != "pause"),
            "an ended pause is not reported"
        );
    }

    #[test]
    fn the_tray_follows_the_daemon_outside_gnome_only() {
        let mut f = healthy();
        assert!(evaluate(&f).iter().all(|c| c.id != "tray"), "GNOME: panel");
        f.gnome = None;
        for (tray, status) in [
            ("active", Status::Ok),
            ("no_host", Status::Warn),
            ("no_bus", Status::Warn),
            ("disabled", Status::Info),
        ] {
            f.daemon = DaemonFact::Up(Box::new(daemon_status(StateKind::Recording, None, tray)));
            let checks = evaluate(&f);
            assert_eq!(find(&checks, "tray").status, status, "{tray}");
        }
    }

    #[test]
    fn unknown_gnome_details_are_never_failures() {
        let mut f = healthy();
        extension(&mut f).enabled = None;
        f.gnome.as_mut().unwrap().shell_version = None;
        let checks = evaluate(&f);
        assert_eq!(find(&checks, "gnome_extension").status, Status::Info);
        assert_eq!(find(&checks, "session").message, "Wayland");
        assert!(checks.iter().all(|c| c.id != "gnome_shell"));
        f.gnome.as_mut().unwrap().extension = None;
        assert_eq!(find(&evaluate(&f), "gnome_extension").status, Status::Info);
    }

    #[test]
    fn autostart_missing_is_information_and_both_kinds_are_listed() {
        let mut f = healthy();
        f.autostart.clear();
        let checks = evaluate(&f);
        let a = find(&checks, "autostart");
        assert_eq!(a.status, Status::Info);
        assert!(a.hint.as_deref().unwrap().contains("install-autostart"));
        assert_eq!(exit_code(&checks), 0);
        f.autostart = vec![
            (
                AutostartKind::Xdg,
                PathBuf::from("/c/autostart/rstory.desktop"),
            ),
            (
                AutostartKind::Systemd,
                PathBuf::from("/c/systemd/user/rstory.service"),
            ),
        ];
        let m = find(&evaluate(&f), "autostart").message.clone();
        assert!(
            m.contains("rstory.desktop") && m.contains("rstory.service"),
            "{m}"
        );
    }

    #[test]
    fn store_states_other_than_too_open() {
        let mut f = healthy();
        for (store, status) in [
            (StoreFact::Unknown, Status::Fail),
            (StoreFact::Missing(PathBuf::from(STORE)), Status::Ok),
            (
                StoreFact::Unreadable {
                    dir: PathBuf::from(STORE),
                },
                Status::Fail,
            ),
            (StoreFact::Private { entries: None }, Status::Ok),
        ] {
            f.store = store.clone();
            let checks = evaluate(&f);
            assert_eq!(find(&checks, "store").status, status, "{store:?}");
            assert!(
                status == Status::Ok || find(&checks, "store").hint.is_some(),
                "{store:?}"
            );
        }
    }

    #[test]
    fn a_configured_picker_command_is_named() {
        let mut f = healthy();
        f.picker = LaunchPlan::Found {
            source: Source::Config,
            argv: vec!["/opt/term/mine".into(), "rstory".into(), "pick".into()],
        };
        let m = find(&evaluate(&f), "picker").message.clone();
        assert_eq!(m, "mine (picker_command in config.toml)");
    }
}

mod render {
    use super::*;
    use crate::doctor::render;

    fn term(ascii: bool) -> Term {
        Term::new(false, 80, Tier::Mono, ascii)
    }

    #[test]
    fn json_has_one_object_per_check_in_the_contract_shape() {
        let mut f = healthy();
        (rows()[3].mutate)(&mut f);
        let checks = evaluate(&f);
        let out = render::json(&checks);
        assert!(out.ends_with('\n') && out.matches('\n').count() == 1);
        let doc: serde_json::Value = serde_json::from_str(&out).unwrap();
        let schema = serde_json::json!({
            "type": "object",
            "required": ["schema", "checks"],
            "additionalProperties": false,
            "properties": {
                "schema": {"const": 1},
                "checks": {"type": "array", "items": {
                    "type": "object",
                    "required": ["id", "status", "message", "hint"],
                    "additionalProperties": false,
                    "properties": {
                        "id": {"type": "string", "minLength": 1},
                        "status": {"enum": ["ok", "info", "warn", "fail"]},
                        "message": {"type": "string", "minLength": 1},
                        "hint": {"type": ["string", "null"]}
                    }
                }}
            }
        });
        let v = jsonschema::validator_for(&schema).unwrap();
        let errors: Vec<String> = v.iter_errors(&doc).map(|e| e.to_string()).collect();
        assert!(errors.is_empty(), "{errors:?} in {doc}");
        assert_eq!(doc["checks"].as_array().unwrap().len(), checks.len());
        let ext = doc["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == "gnome_extension")
            .unwrap();
        assert_eq!(ext["status"], "warn");
        assert!(ext["hint"].is_string());
        assert!(doc["checks"][0]["hint"].is_null());
    }

    #[test]
    fn text_ends_with_the_summary() {
        let healthy_text = render::text(&evaluate(&healthy()), &term(false));
        assert!(
            healthy_text.ends_with("\nEverything looks good.\n"),
            "{healthy_text}"
        );
        let mut f = healthy();
        (rows()[3].mutate)(&mut f);
        assert!(render::text(&evaluate(&f), &term(false)).ends_with("\n1 warning.\n"));
        (rows()[1].mutate)(&mut f);
        (rows()[6].mutate)(&mut f);
        let t = render::text(&evaluate(&f), &term(false));
        assert!(t.ends_with("\n2 warnings, 1 failure.\n"), "{t}");
        let mut only_fail = healthy();
        (rows()[1].mutate)(&mut only_fail);
        assert!(render::text(&evaluate(&only_fail), &term(false)).ends_with("\n1 failure.\n"));
    }

    #[test]
    fn text_has_words_as_well_as_glyphs_and_ascii_has_no_unicode() {
        let mut f = healthy();
        (rows()[1].mutate)(&mut f);
        (rows()[3].mutate)(&mut f);
        let ascii = render::text(&evaluate(&f), &term(true));
        assert!(ascii.is_ascii(), "{ascii}");
        assert!(ascii.contains("FAIL") && ascii.contains("warn") && ascii.contains("->"));
        let utf8 = render::text(&evaluate(&f), &term(false));
        assert!(utf8.contains('✗') && utf8.contains('!') && utf8.contains('→'));
        assert!(!utf8.contains('\x1b'), "mono output has no escapes");
    }

    #[test]
    fn every_line_names_what_it_is_about() {
        let t = render::text(&evaluate(&healthy()), &term(false));
        for label in [
            "Session",
            "Backend",
            "Store",
            "Daemon",
            "Extension",
            "Picker",
            "Autostart",
        ] {
            assert!(t.contains(label), "{label} in {t}");
        }
    }

    #[test]
    fn healthy_snapshot() {
        for ascii in [false, true] {
            let name = format!("healthy_{}", if ascii { "ascii" } else { "utf8" });
            insta::assert_snapshot!(name, render::text(&evaluate(&healthy()), &term(ascii)));
        }
    }

    #[test]
    fn failure_row_snapshots() {
        for row in rows() {
            let mut f = healthy();
            (row.mutate)(&mut f);
            for ascii in [false, true] {
                let name = format!("row_{}_{}", row.name, if ascii { "ascii" } else { "utf8" });
                insta::assert_snapshot!(name, render::text(&evaluate(&f), &term(ascii)));
            }
        }
    }
}
