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

mod probes {
    use std::collections::HashMap;
    use std::fs;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;
    use std::path::Path;
    use std::time::{Duration, Instant};

    use tempfile::TempDir;

    use super::*;
    use crate::doctor::probes::{self, ProbeEnv};
    use crate::session::{DataControl, Probe as SessionProbe, SessionEnv};

    const T: Duration = Duration::from_millis(300);

    /// A session probe that answers from fixed values.
    struct FakeProbe {
        data_control: DataControl,
        x11: bool,
    }

    impl SessionProbe for FakeProbe {
        fn data_control(&mut self) -> DataControl {
            self.data_control
        }
        fn x11_reachable(&mut self) -> bool {
            self.x11
        }
    }

    fn bridge() -> FakeProbe {
        FakeProbe {
            data_control: DataControl::None,
            x11: true,
        }
    }

    /// An executable script in `dir`.
    fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    /// Everything under one temp dir; nothing of the real host is reachable.
    struct Rig {
        tmp: TempDir,
        env: ProbeEnv,
    }

    impl Rig {
        fn new() -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let r = tmp.path();
            for d in ["bin", "run/rstory", "state", "config/rstory", "data"] {
                fs::create_dir_all(r.join(d)).unwrap();
            }
            let env = ProbeEnv {
                now: NOW,
                session: SessionEnv {
                    wayland_display: Some("wayland-0".into()),
                    display: Some(":0".into()),
                    session_type: Some("wayland".into()),
                    xdg_runtime_dir: Some(r.join("run").display().to_string()),
                },
                xauthority: None,
                current_desktop: Some("GNOME".into()),
                terminal: None,
                path: r.join("bin").display().to_string(),
                socket: Some(r.join("run/rstory/control.sock")),
                store_dir: Some(r.join("state/rstory")),
                config_path: Some(r.join("config/rstory/config.toml")),
                config_home: Some(r.join("config")),
                data_home: Some(r.join("data")),
                gnome_shell: r.join("bin/gnome-shell"),
                gnome_extensions: r.join("bin/gnome-extensions"),
                timeout: T,
            };
            Self { tmp, env }
        }

        fn bin(&self, name: &str, body: &str) -> PathBuf {
            script(&self.tmp.path().join("bin"), name, body)
        }

        fn store(&self) -> PathBuf {
            let d = self.env.store_dir.clone().unwrap();
            fs::create_dir_all(&d).unwrap();
            fs::set_permissions(&d, fs::Permissions::from_mode(0o700)).unwrap();
            d
        }

        fn extension_dir(&self) -> PathBuf {
            crate::install::gnome::extension_dir(self.env.data_home.as_ref().unwrap())
        }

        /// A stub daemon that answers `status` with `status` and then stops.
        fn stub_daemon(&self, status: &DaemonStatus) {
            let listener = UnixListener::bind(self.env.socket.as_ref().unwrap()).unwrap();
            let result = serde_json::to_value(status).unwrap();
            std::thread::spawn(move || {
                let Ok((stream, _)) = listener.accept() else {
                    return;
                };
                let mut line = String::new();
                BufReader::new(&stream).read_line(&mut line).unwrap();
                let req: serde_json::Value = serde_json::from_str(&line).unwrap();
                assert_eq!(req["op"], "status");
                let reply =
                    serde_json::json!({"v": 1, "id": req["id"], "ok": true, "result": result});
                writeln!(&stream, "{reply}").unwrap();
            });
        }
    }

    /// Every path under `dir` with its mode and size.
    fn tree(dir: &Path) -> Vec<(PathBuf, u32, u64)> {
        let mut out = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            for e in fs::read_dir(&d).unwrap().flatten() {
                let m = e.metadata().unwrap();
                out.push((e.path(), m.permissions().mode(), m.len()));
                if m.is_dir() {
                    stack.push(e.path());
                }
            }
        }
        out.sort();
        out
    }

    #[test]
    fn the_environment_gives_paths_the_way_the_cli_finds_them() {
        let vars: HashMap<&str, &str> = HashMap::from([
            ("XDG_RUNTIME_DIR", "/r"),
            ("XDG_STATE_HOME", "/s"),
            ("XDG_CONFIG_HOME", "/c"),
            ("XDG_DATA_HOME", "/d"),
            ("HOME", "/h"),
            ("WAYLAND_DISPLAY", "wayland-1"),
            ("DISPLAY", ""),
            ("XDG_CURRENT_DESKTOP", "ubuntu:GNOME"),
            ("TERMINAL", "foot"),
            ("PATH", "/x:/y"),
        ]);
        let get = |k: &str| vars.get(k).map(|v| (*v).to_string());
        let e = ProbeEnv::from_lookup(&get, 1000, Path::new("/nonexistent"), NOW);
        assert_eq!(
            e.socket.as_deref(),
            Some(Path::new("/r/rstory/control.sock"))
        );
        assert_eq!(e.store_dir.as_deref(), Some(Path::new("/s/rstory")));
        assert_eq!(
            e.config_path.as_deref(),
            Some(Path::new("/c/rstory/config.toml"))
        );
        assert_eq!(e.config_home.as_deref(), Some(Path::new("/c")));
        assert_eq!(e.data_home.as_deref(), Some(Path::new("/d")));
        assert_eq!(e.session.wayland_display.as_deref(), Some("wayland-1"));
        assert_eq!(e.session.display, None, "empty counts as unset");
        assert_eq!(e.terminal.as_deref(), Some("foot"));
        assert_eq!(e.path, "/x:/y");
        assert_eq!(e.timeout, probes::PROBE_TIMEOUT);

        let home_only: HashMap<&str, &str> = HashMap::from([("HOME", "/h")]);
        let get = |k: &str| home_only.get(k).map(|v| (*v).to_string());
        let e = ProbeEnv::from_lookup(&get, 1000, Path::new("/nonexistent"), NOW);
        assert_eq!(e.socket, None, "no runtime directory, no socket");
        assert_eq!(
            e.store_dir.as_deref(),
            Some(Path::new("/h/.local/state/rstory"))
        );
        assert_eq!(e.config_home.as_deref(), Some(Path::new("/h/.config")));
        assert_eq!(e.data_home.as_deref(), Some(Path::new("/h/.local/share")));

        let none = ProbeEnv::from_lookup(&|_| None, 1000, Path::new("/nonexistent"), NOW);
        assert_eq!(
            (none.store_dir, none.config_home, none.data_home),
            (None, None, None)
        );
    }

    #[test]
    fn session_kind_follows_the_type_then_the_displays() {
        let k = |t: Option<&str>, w: Option<&str>, d: Option<&str>| {
            probes::session_kind(&SessionEnv {
                wayland_display: w.map(Into::into),
                display: d.map(Into::into),
                session_type: t.map(Into::into),
                xdg_runtime_dir: None,
            })
        };
        assert_eq!(k(Some("wayland"), None, None), SessionKind::Wayland);
        assert_eq!(k(Some("X11"), Some("w"), None), SessionKind::X11);
        assert_eq!(k(None, Some("w"), Some(":0")), SessionKind::Wayland);
        assert_eq!(k(Some("tty"), None, Some(":0")), SessionKind::X11);
        assert_eq!(k(None, None, None), SessionKind::None);
        assert_eq!(k(Some(""), Some(""), Some("")), SessionKind::None);
    }

    #[test]
    fn shell_versions_are_parsed() {
        assert_eq!(
            probes::parse_shell_version("GNOME Shell 48.7\n").as_deref(),
            Some("48.7")
        );
        assert_eq!(
            probes::parse_shell_version("GNOME Shell 50.alpha").as_deref(),
            Some("50.alpha")
        );
        assert_eq!(probes::parse_shell_version("something else"), None);
        assert_eq!(probes::parse_shell_version(""), None);
    }

    #[test]
    fn run_capture_returns_output_and_gives_up_on_slow_or_failing_programs() {
        let rig = Rig::new();
        let ok = rig.bin("ok", "echo hello");
        assert_eq!(probes::run_capture(&ok, &[], T).as_deref(), Some("hello\n"));
        let args = rig.bin("args", "echo \"$1 $2\"");
        assert_eq!(
            probes::run_capture(&args, &["a", "b"], T).as_deref(),
            Some("a b\n")
        );
        let bad = rig.bin("bad", "echo out; exit 3");
        assert_eq!(probes::run_capture(&bad, &[], T), None);
        assert_eq!(
            probes::run_capture(Path::new("/nonexistent/prog"), &[], T),
            None
        );
        let slow = rig.bin("slow", "exec sleep 5");
        let t0 = Instant::now();
        assert_eq!(
            probes::run_capture(&slow, &[], Duration::from_millis(150)),
            None
        );
        assert!(t0.elapsed() < Duration::from_secs(2), "{:?}", t0.elapsed());
    }

    #[test]
    fn no_daemon_means_down() {
        let mut rig = Rig::new();
        assert_eq!(
            probes::daemon_fact(&rig.env),
            DaemonFact::Down,
            "no socket file"
        );
        rig.env.socket = None;
        assert_eq!(
            probes::daemon_fact(&rig.env),
            DaemonFact::Down,
            "no socket path"
        );
    }

    #[test]
    fn a_socket_file_nobody_listens_on_means_down() {
        let rig = Rig::new();
        drop(UnixListener::bind(rig.env.socket.as_ref().unwrap()).unwrap());
        assert_eq!(probes::daemon_fact(&rig.env), DaemonFact::Down);
    }

    #[test]
    fn a_daemon_that_answers_status_is_up() {
        let rig = Rig::new();
        let status = daemon_status(StateKind::Paused, Some(60), "active");
        rig.stub_daemon(&status);
        assert_eq!(
            probes::daemon_fact(&rig.env),
            DaemonFact::Up(Box::new(status))
        );
    }

    #[test]
    fn a_daemon_that_stays_silent_or_talks_nonsense_is_not_answering() {
        let rig = Rig::new();
        let listener = UnixListener::bind(rig.env.socket.as_ref().unwrap()).unwrap();
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            let _ = rx.recv(); // hold the connection open without answering
        });
        let t0 = Instant::now();
        assert_eq!(probes::daemon_fact(&rig.env), DaemonFact::NotAnswering);
        assert!(t0.elapsed() < Duration::from_secs(2));
        drop(tx);

        let rig = Rig::new();
        let listener = UnixListener::bind(rig.env.socket.as_ref().unwrap()).unwrap();
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut line = String::new();
            BufReader::new(&stream).read_line(&mut line).unwrap();
            writeln!(&stream, "not json at all").unwrap();
        });
        assert_eq!(probes::daemon_fact(&rig.env), DaemonFact::NotAnswering);

        let rig = Rig::new();
        fs::write(rig.env.socket.as_ref().unwrap(), "a plain file").unwrap();
        assert_eq!(probes::daemon_fact(&rig.env), DaemonFact::NotAnswering);
    }

    #[test]
    fn a_missing_store_is_missing_and_nothing_is_created() {
        let rig = Rig::new();
        let before = tree(rig.tmp.path());
        let (fact, pause) = probes::store_fact(&rig.env, &DaemonFact::Down);
        assert_eq!(fact, StoreFact::Missing(rig.env.store_dir.clone().unwrap()));
        assert_eq!(pause, None);
        assert_eq!(tree(rig.tmp.path()), before);
        let mut none = rig.env.clone();
        none.store_dir = None;
        assert_eq!(
            probes::store_fact(&none, &DaemonFact::Down).0,
            StoreFact::Unknown
        );
    }

    #[test]
    fn a_private_empty_store_counts_zero_entries_without_writing() {
        let rig = Rig::new();
        rig.store();
        let before = tree(rig.tmp.path());
        let (fact, _) = probes::store_fact(&rig.env, &DaemonFact::Down);
        assert_eq!(fact, StoreFact::Private { entries: Some(0) });
        assert_eq!(tree(rig.tmp.path()), before, "doctor writes nothing");
    }

    #[test]
    fn a_store_open_to_the_group_is_too_open() {
        let rig = Rig::new();
        let dir = rig.store();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o750)).unwrap();
        let (fact, _) = probes::store_fact(&rig.env, &DaemonFact::Down);
        assert_eq!(fact, StoreFact::TooOpen { dir: dir.clone() });
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(dir.join("index.json"), "{}").unwrap();
        fs::set_permissions(dir.join("index.json"), fs::Permissions::from_mode(0o644)).unwrap();
        let (fact, _) = probes::store_fact(&rig.env, &DaemonFact::Down);
        assert_eq!(fact, StoreFact::TooOpen { dir: dir.clone() });
    }

    #[test]
    fn a_running_daemon_supplies_the_count_and_the_store_is_not_opened() {
        let rig = Rig::new();
        rig.store();
        let up = DaemonFact::Up(Box::new(daemon_status(
            StateKind::Recording,
            None,
            "active",
        )));
        let (fact, pause) = probes::store_fact(&rig.env, &up);
        assert_eq!(fact, StoreFact::Private { entries: Some(8) });
        assert_eq!(pause, None);
    }

    #[test]
    fn the_shell_version_and_the_extension_come_from_the_injected_programs() {
        let rig = Rig::new();
        rig.bin("gnome-shell", "echo 'GNOME Shell 48.7'");
        rig.bin(
            "gnome-extensions",
            &format!("[ \"$1 $2\" = \"info {UUID}\" ] || exit 9\necho '  Enabled: Yes'"),
        );
        let ext = rig.extension_dir();
        fs::create_dir_all(&ext).unwrap();
        fs::write(ext.join("metadata.json"), r#"{"version-name":"0.0.9"}"#).unwrap();
        let g = probes::gnome_facts(&rig.env);
        assert_eq!(g.shell_version.as_deref(), Some("48.7"));
        let e = g.extension.unwrap();
        assert_eq!(e.installed_version.as_deref(), Some("0.0.9"));
        assert_eq!(e.enabled, Some(true));
        assert_eq!(g.shipped_version, crate::install::gnome::embedded_version());
        assert!(g.shipped_version.is_some());
    }

    #[test]
    fn missing_gnome_programs_leave_unknowns_not_failures() {
        let rig = Rig::new();
        let g = probes::gnome_facts(&rig.env);
        assert_eq!(g.shell_version, None);
        let e = g.extension.unwrap();
        assert_eq!((e.installed_version, e.enabled), (None, None));
    }

    #[test]
    fn a_slow_gnome_extensions_does_not_hold_doctor_up() {
        let mut rig = Rig::new();
        rig.env.timeout = Duration::from_millis(150);
        rig.bin("gnome-shell", "exec sleep 5");
        rig.bin("gnome-extensions", "exec sleep 5");
        let ext = rig.extension_dir();
        fs::create_dir_all(&ext).unwrap();
        fs::write(ext.join("metadata.json"), r#"{"version-name":"0.1.0"}"#).unwrap();
        let t0 = Instant::now();
        let g = probes::gnome_facts(&rig.env);
        assert!(t0.elapsed() < Duration::from_secs(1), "{:?}", t0.elapsed());
        assert_eq!(g.shell_version, None);
        let e = g.extension.unwrap();
        assert_eq!(
            e.installed_version.as_deref(),
            Some("0.1.0"),
            "read from the file"
        );
        assert_eq!(e.enabled, None);
    }

    #[test]
    fn the_picker_plan_searches_the_injected_path_and_reads_the_config() {
        let mut rig = Rig::new();
        assert!(matches!(
            probes::picker_plan(&rig.env),
            LaunchPlan::NotFound { .. }
        ));
        rig.bin("kitty", "true");
        match probes::picker_plan(&rig.env) {
            LaunchPlan::Found { source, argv } => {
                assert_eq!(source, Source::List);
                assert_eq!(argv[0], "kitty");
            }
            other => panic!("{other:?}"),
        }
        rig.bin("myterm", "true");
        rig.env.terminal = Some("myterm".into());
        assert!(matches!(
            probes::picker_plan(&rig.env),
            LaunchPlan::Found {
                source: Source::Env,
                ..
            }
        ));
        fs::write(
            rig.env.config_path.as_ref().unwrap(),
            "picker_command = \"/opt/x/term -e rstory pick\"\n",
        )
        .unwrap();
        match probes::picker_plan(&rig.env) {
            LaunchPlan::Found { source, argv } => {
                assert_eq!(source, Source::Config);
                assert_eq!(argv[0], "/opt/x/term");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_broken_config_file_does_not_stop_the_picker_search() {
        let rig = Rig::new();
        rig.bin("foot", "true");
        fs::write(rig.env.config_path.as_ref().unwrap(), "this is [ not toml").unwrap();
        assert!(matches!(
            probes::picker_plan(&rig.env),
            LaunchPlan::Found {
                source: Source::List,
                ..
            }
        ));
    }

    #[test]
    fn autostart_entries_are_found_where_the_installer_writes_them() {
        let rig = Rig::new();
        assert!(probes::autostart_facts(&rig.env).is_empty());
        let cfg = rig.env.config_home.clone().unwrap();
        fs::create_dir_all(cfg.join("autostart")).unwrap();
        fs::write(cfg.join("autostart/rstory.desktop"), "[Desktop Entry]").unwrap();
        let found = probes::autostart_facts(&rig.env);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, AutostartKind::Xdg);
        assert_eq!(found[0].1, cfg.join("autostart/rstory.desktop"));
        let mut none = rig.env.clone();
        none.config_home = None;
        assert!(probes::autostart_facts(&none).is_empty());
    }

    #[test]
    fn gather_and_evaluate_report_a_healthy_fake_setup() {
        let rig = Rig::new();
        rig.store();
        rig.bin("gnome-shell", "echo 'GNOME Shell 48.7'");
        rig.bin("gnome-extensions", "echo 'Enabled: Yes'");
        rig.bin("ghostty", "true");
        let shipped = crate::install::gnome::embedded_version().unwrap();
        let ext = rig.extension_dir();
        fs::create_dir_all(&ext).unwrap();
        fs::write(
            ext.join("metadata.json"),
            format!(r#"{{"version-name":"{shipped}"}}"#),
        )
        .unwrap();
        let cfg = rig.env.config_home.clone().unwrap();
        fs::create_dir_all(cfg.join("autostart")).unwrap();
        fs::write(cfg.join("autostart/rstory.desktop"), "x").unwrap();
        rig.stub_daemon(&daemon_status(StateKind::Recording, None, "no_host"));

        let before = tree(rig.tmp.path());
        let facts = probes::gather(&rig.env, &mut bridge());
        let checks = evaluate(&facts);
        assert!(checks.iter().all(|c| c.status == Status::Ok), "{checks:?}");
        assert_eq!(exit_code(&checks), 0);
        let session = checks.iter().find(|c| c.id == "session").unwrap();
        assert_eq!(session.message, "Wayland (GNOME Shell 48.7)");
        let after = tree(rig.tmp.path());
        let strip = |t: Vec<(PathBuf, u32, u64)>| -> Vec<PathBuf> {
            t.into_iter()
                .map(|(p, _, _)| p)
                .filter(|p| !p.ends_with("control.sock"))
                .collect()
        };
        assert_eq!(strip(before), strip(after), "doctor creates nothing");
    }

    #[test]
    fn gather_without_a_session_or_daemon_still_reports() {
        let mut rig = Rig::new();
        rig.env.session = SessionEnv::default();
        rig.env.current_desktop = None;
        let facts = probes::gather(
            &rig.env,
            &mut FakeProbe {
                data_control: DataControl::None,
                x11: false,
            },
        );
        assert_eq!(facts.session.kind, SessionKind::None);
        assert_eq!(facts.daemon, DaemonFact::Down);
        assert_eq!(facts.gnome, None);
        let checks = evaluate(&facts);
        assert_eq!(exit_code(&checks), 2);
        assert!(
            checks
                .iter()
                .any(|c| c.id == "session" && c.message.contains("no graphical session"))
        );
    }

    #[test]
    fn gather_stays_inside_the_time_budget_with_slow_programs() {
        let mut rig = Rig::new();
        rig.env.timeout = Duration::from_millis(200);
        rig.bin("gnome-shell", "exec sleep 5");
        rig.bin("gnome-extensions", "exec sleep 5");
        let ext = rig.extension_dir();
        fs::create_dir_all(&ext).unwrap();
        fs::write(ext.join("metadata.json"), r#"{"version-name":"0.1.0"}"#).unwrap();
        let t0 = Instant::now();
        let facts = probes::gather(&rig.env, &mut bridge());
        assert!(t0.elapsed() < Duration::from_secs(1), "{:?}", t0.elapsed());
        assert!(facts.gnome.is_some());
    }

    #[test]
    fn arguments_are_json_or_nothing() {
        let a = |v: &[&str]| v.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
        assert_eq!(parse_args(&a(&[])), Ok(Options { json: false }));
        assert_eq!(parse_args(&a(&["--json"])), Ok(Options { json: true }));
        assert!(parse_args(&a(&["--yaml"])).unwrap_err().contains("--yaml"));
        assert!(parse_args(&a(&["--json", "extra"])).is_err());
    }

    #[test]
    fn run_with_prints_the_report_and_returns_the_scale() {
        let rig = Rig::new();
        let term = Term::new(false, 80, Tier::Mono, false);
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = probes_run(&rig, &["--json"], &term, &mut out, &mut err);
        assert_eq!(code, 2, "no daemon: a failure");
        assert!(err.is_empty());
        let doc: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(doc["schema"], 1);
        assert!(
            doc["checks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["id"] == "daemon" && c["status"] == "fail")
        );

        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = probes_run(&rig, &[], &term, &mut out, &mut err);
        assert_eq!(code, 2);
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.contains("Daemon") && text.contains("failure"),
            "{text}"
        );

        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = probes_run(&rig, &["--nope"], &term, &mut out, &mut err);
        assert_eq!(code, 2);
        assert!(out.is_empty());
        assert!(String::from_utf8(err).unwrap().contains("--nope"));
    }

    fn probes_run(
        rig: &Rig,
        args: &[&str],
        term: &Term,
        out: &mut Vec<u8>,
        err: &mut Vec<u8>,
    ) -> u8 {
        let args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
        run_with(&args, &rig.env, &mut bridge(), term, out, err)
    }
}
