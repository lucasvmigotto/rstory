//! The Wayland registry probe against two real compositors: headless sway offers
//! `zwlr_data_control_manager_v1`, headless mutter offers neither data-control global.

mod support;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use rstory::session::{DataControl, Probe, RealProbe, SessionEnv};
use support::mutter::Mutter;
use support::wait_for;

struct Sway {
    child: Child,
    dir: PathBuf,
}

impl Sway {
    fn start() -> Self {
        let dir = PathBuf::from(format!("/tmp/rstory-sway-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::set_permissions(&dir, std::os::unix::fs::PermissionsExt::from_mode(0o700))
            .unwrap();
        let child = Command::new("sway")
            .args(["--unsupported-gpu", "-c", "/dev/null"])
            .env("XDG_RUNTIME_DIR", &dir)
            .env("WLR_BACKENDS", "headless")
            .env("WLR_RENDERER", "pixman")
            .env("WLR_LIBINPUT_NO_DEVICES", "1")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("DISPLAY")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let s = Self { child, dir };
        let sock = s.dir.join("wayland-1");
        wait_for(Duration::from_secs(20), || sock.exists().then_some(())).expect("sway socket");
        s
    }
}

impl Drop for Sway {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

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
    let mut probe = RealProbe::new(&env(&s.dir, "wayland-1"), None);
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
