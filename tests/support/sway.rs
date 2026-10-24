//! Headless sway launcher for the Wayland session tests.
//!
//! Each instance gets a private `XDG_RUNTIME_DIR`, so tests can run in parallel. sway 1.10
//! (wlroots 0.18) offers `zwlr_data_control_manager_v1` only; tests that need
//! `ext_data_control_manager_v1` check `Sway::has_global` and say so when they skip.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use super::wait_for;

static SEQ: AtomicU32 = AtomicU32::new(0);

/// A private headless sway. Killed on drop.
pub struct Sway {
    child: Child,
    dir: PathBuf,
}

impl Sway {
    /// Start sway and return once its Wayland socket exists.
    pub fn start() -> Self {
        let dir = PathBuf::from(format!(
            "/tmp/rstory-sway-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
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
        let sock = s.socket();
        wait_for(Duration::from_secs(20), || sock.exists().then_some(())).expect("sway socket");
        s
    }

    /// The private runtime directory.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// `WAYLAND_DISPLAY` value.
    pub fn display(&self) -> &'static str {
        "wayland-1"
    }

    /// Path of the Wayland socket.
    pub fn socket(&self) -> PathBuf {
        self.dir.join(self.display())
    }

    /// Whether the registry advertises `interface`.
    pub fn has_global(&self, interface: &str) -> bool {
        super::wl_client::registry_interfaces(&self.socket())
            .is_some_and(|v| v.iter().any(|i| i == interface))
    }
}

impl Drop for Sway {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
