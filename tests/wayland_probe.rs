//! The Wayland registry probe against two real compositors: headless sway offers
//! `zwlr_data_control_manager_v1`, headless mutter offers neither data-control global.

mod support;

use rstory::session::{DataControl, Probe, RealProbe, SessionEnv};
use support::mutter::Mutter;
use support::sway::Sway;

fn env(dir: &std::path::Path, name: &str) -> SessionEnv {
    SessionEnv {
        session_type: Some("wayland".into()),
        wayland_display: Some(name.into()),
        display: None,
        xdg_runtime_dir: Some(dir.to_string_lossy().into_owned()),
    }
}

#[test]
fn sway_offers_wlr_data_control() {
    let s = Sway::start();
    let mut probe = RealProbe::new(&env(s.dir(), s.display()), None);
    assert_eq!(probe.data_control(), DataControl::Zwlr);
}

#[test]
fn mutter_offers_no_data_control() {
    let m = Mutter::start().unwrap();
    let mut probe = RealProbe::new(&env(m.runtime_dir(), &m.wayland_display), None);
    assert_eq!(probe.data_control(), DataControl::None);
}

#[test]
fn a_missing_socket_means_no_data_control() {
    let mut probe = RealProbe::new(
        &env(
            std::path::Path::new("/tmp/rstory-nothing-here"),
            "wayland-9",
        ),
        None,
    );
    assert_eq!(probe.data_control(), DataControl::None);
}
