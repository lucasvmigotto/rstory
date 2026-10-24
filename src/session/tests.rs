//! SC-003: the selection table, one row per environment.

use super::*;

struct FakeProbe {
    data_control: DataControl,
    x11: bool,
    asked_registry: bool,
    asked_x11: bool,
}

impl FakeProbe {
    fn new(data_control: DataControl, x11: bool) -> Self {
        Self {
            data_control,
            x11,
            asked_registry: false,
            asked_x11: false,
        }
    }
}

impl Probe for FakeProbe {
    fn data_control(&mut self) -> DataControl {
        self.asked_registry = true;
        self.data_control
    }
    fn x11_reachable(&mut self) -> bool {
        self.asked_x11 = true;
        self.x11
    }
}

fn env(session: Option<&str>, wayland: Option<&str>, display: Option<&str>) -> SessionEnv {
    SessionEnv {
        session_type: session.map(str::to_string),
        wayland_display: wayland.map(str::to_string),
        display: display.map(str::to_string),
        xdg_runtime_dir: Some("/run/user/1000".into()),
    }
}

const NONE: DataControl = DataControl::None;
const EXT: DataControl = DataControl::Ext;
const ZWLR: DataControl = DataControl::Zwlr;

use BackendKind as B;
use Mode as M;
use Reason as R;

struct Row {
    name: &'static str,
    env: SessionEnv,
    data_control: DataControl,
    x11_reachable: bool,
    override_backend: Option<BackendKind>,
    want: (BackendKind, Mode, Reason),
}

fn rows() -> Vec<Row> {
    let row = |name, env, data_control, x11_reachable, override_backend, want| Row {
        name,
        env,
        data_control,
        x11_reachable,
        override_backend,
        want,
    };
    vec![
        row(
            "x11 session",
            env(Some("x11"), None, Some(":0")),
            NONE,
            true,
            None,
            (B::X11, M::Native, R::X11Session),
        ),
        row(
            "x11 session, display unreachable",
            env(Some("x11"), None, Some(":0")),
            NONE,
            false,
            None,
            (B::None, M::None, R::X11Unreachable),
        ),
        row(
            "x11 session without DISPLAY",
            env(Some("x11"), None, None),
            NONE,
            true,
            None,
            (B::None, M::None, R::NoSession),
        ),
        row(
            "wayland with ext_data_control",
            env(Some("wayland"), Some("wayland-0"), Some(":0")),
            EXT,
            true,
            None,
            (B::Wayland, M::Native, R::DataControl),
        ),
        row(
            "wayland with zwlr_data_control",
            env(Some("wayland"), Some("wayland-0"), None),
            ZWLR,
            false,
            None,
            (B::Wayland, M::Native, R::DataControl),
        ),
        row(
            "gnome: wayland, no data-control, XWayland",
            env(Some("wayland"), Some("wayland-0"), Some(":0")),
            NONE,
            true,
            None,
            (B::X11, M::XwaylandBridge, R::NoDataControl),
        ),
        row(
            "wayland, no data-control, XWayland unreachable",
            env(Some("wayland"), Some("wayland-0"), Some(":0")),
            NONE,
            false,
            None,
            (B::None, M::None, R::X11Unreachable),
        ),
        row(
            "wayland, no data-control, no DISPLAY",
            env(Some("wayland"), Some("wayland-0"), None),
            NONE,
            true,
            None,
            (B::None, M::None, R::NoDataControlNoXwayland),
        ),
        row(
            "no session at all",
            env(None, None, None),
            NONE,
            true,
            None,
            (B::None, M::None, R::NoSession),
        ),
        row(
            "unknown session type, nothing set",
            env(Some("tty"), None, None),
            NONE,
            true,
            None,
            (B::None, M::None, R::NoSession),
        ),
        row(
            "no session type, WAYLAND_DISPLAY with zwlr",
            env(None, Some("wayland-1"), None),
            ZWLR,
            true,
            None,
            (B::Wayland, M::Native, R::DataControl),
        ),
        row(
            "no session type, only DISPLAY",
            env(None, None, Some(":1")),
            NONE,
            true,
            None,
            (B::X11, M::Native, R::X11Session),
        ),
        row(
            "wayland session without WAYLAND_DISPLAY, with DISPLAY",
            env(Some("wayland"), None, Some(":0")),
            EXT,
            true,
            None,
            (B::X11, M::XwaylandBridge, R::NoDataControl),
        ),
        row(
            "wayland session without any display",
            env(Some("wayland"), None, None),
            EXT,
            true,
            None,
            (B::None, M::None, R::NoSession),
        ),
        row(
            "empty variables count as unset",
            env(Some(""), Some(""), Some("")),
            EXT,
            true,
            None,
            (B::None, M::None, R::NoSession),
        ),
        row(
            "override x11 on x11",
            env(Some("x11"), None, Some(":0")),
            NONE,
            false,
            Some(B::X11),
            (B::X11, M::Native, R::Override),
        ),
        row(
            "override x11 on a data-control wayland session",
            env(Some("wayland"), Some("wayland-0"), Some(":0")),
            EXT,
            true,
            Some(B::X11),
            (B::X11, M::XwaylandBridge, R::Override),
        ),
        row(
            "override wayland on x11",
            env(Some("x11"), None, Some(":0")),
            NONE,
            true,
            Some(B::Wayland),
            (B::Wayland, M::Native, R::Override),
        ),
        row(
            "override wins with nothing set",
            env(None, None, None),
            NONE,
            false,
            Some(B::X11),
            (B::X11, M::Native, R::Override),
        ),
    ]
}

#[test]
fn table() {
    let rows = rows();
    assert!(rows.len() >= 8);
    for r in rows {
        let mut probe = FakeProbe::new(r.data_control, r.x11_reachable);
        let got = select(&r.env, &mut probe, r.override_backend);
        assert_eq!(
            (got.backend, got.mode, got.reason),
            r.want,
            "row: {}",
            r.name
        );
    }
}

#[test]
fn the_probe_is_only_asked_what_the_decision_needs() {
    let mut p = FakeProbe::new(EXT, true);
    let _ = select(&env(Some("x11"), None, Some(":0")), &mut p, None);
    assert!(
        !p.asked_registry && p.asked_x11,
        "x11 session: no registry round trip"
    );

    let mut p = FakeProbe::new(EXT, true);
    let _ = select(&env(Some("wayland"), Some("w"), Some(":0")), &mut p, None);
    assert!(
        p.asked_registry && !p.asked_x11,
        "data-control found: no X connection"
    );

    let mut p = FakeProbe::new(NONE, true);
    let _ = select(
        &env(Some("wayland"), None, Some(":0")),
        &mut p,
        Some(B::X11),
    );
    assert!(
        !p.asked_registry && !p.asked_x11,
        "an override asks nothing"
    );
}

#[test]
fn interfaces_pick_ext_over_zwlr() {
    let v = |names: &[&str]| names.iter().map(|n| (*n).to_string()).collect::<Vec<_>>();
    assert_eq!(
        data_control_from(&v(&["wl_seat", "zwlr_data_control_manager_v1"])),
        ZWLR
    );
    assert_eq!(
        data_control_from(&v(&[
            "ext_data_control_manager_v1",
            "zwlr_data_control_manager_v1"
        ])),
        EXT
    );
    assert_eq!(data_control_from(&v(&["wl_seat", "wl_compositor"])), NONE);
    assert_eq!(data_control_from(&[]), NONE);
}

#[test]
fn override_values() {
    assert_eq!(parse_override("x11"), Some(B::X11));
    assert_eq!(parse_override("wayland"), Some(B::Wayland));
    assert_eq!(parse_override("auto"), None);
    assert_eq!(parse_override(""), None);
    assert_eq!(parse_override("bogus"), None);
}

#[test]
fn names_are_stable() {
    assert_eq!(
        R::NoDataControlNoXwayland.as_str(),
        "no_data_control_no_xwayland"
    );
    assert_eq!(R::X11Unreachable.as_str(), "x11_unreachable");
    assert_eq!(M::XwaylandBridge.as_str(), "xwayland_bridge");
    assert_eq!(B::X11.as_str(), "x11");
}
