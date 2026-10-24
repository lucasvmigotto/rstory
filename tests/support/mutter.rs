//! Headless mutter launcher: GNOME's compositor with no data-control and XWayland on demand.
//!
//! Each session gets a private D-Bus, a private `XDG_RUNTIME_DIR` (so the Wayland socket and the
//! `.mutter-Xwaylandauth.*` file are unambiguous) and a unique Wayland display name. Readiness is
//! observed (log line, socket, auth file), never slept for.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use super::wait_for;

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct Mutter {
    child: Child,
    bus_pid: Option<i32>,
    dir: PathBuf,
    /// Wayland socket name inside [`Mutter::runtime_dir`].
    pub wayland_display: String,
    /// Public X display, `:N`. XWayland starts on the first connection.
    pub display: String,
}

impl Mutter {
    pub fn start() -> Result<Self, Box<dyn std::error::Error>> {
        Self::start_with(true)
    }

    /// `mutter --no-x11`: no data-control and no XWayland, so nothing can capture the clipboard.
    /// [`Mutter::display`] stays empty.
    pub fn start_no_x11() -> Result<Self, Box<dyn std::error::Error>> {
        Self::start_with(false)
    }

    fn start_with(x11: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let id = format!(
            "{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        );
        let dir = PathBuf::from(format!("/tmp/rstory-mutter-{id}"));
        std::fs::create_dir_all(&dir)?;
        std::fs::set_permissions(&dir, std::os::unix::fs::PermissionsExt::from_mode(0o700))?;
        let wayland_display = format!("rstory-wl-{id}");

        let bus = Command::new("dbus-launch").arg("--sh-syntax").output()?;
        let text = String::from_utf8_lossy(&bus.stdout).into_owned();
        let field = |key: &str| -> Option<String> {
            let rest = text.split(key).nth(1)?;
            let rest = rest.trim_start_matches(['=', '\'']);
            Some(rest.split(['\'', ';']).next()?.to_string())
        };
        let address = field("DBUS_SESSION_BUS_ADDRESS").ok_or("dbus-launch gave no address")?;
        let bus_pid = field("DBUS_SESSION_BUS_PID").and_then(|p| p.trim().parse().ok());

        let log_path = dir.join("mutter.log");
        let log = File::create(&log_path)?;
        let mut mutter = Command::new("mutter");
        mutter.args(["--headless", "--wayland", "--virtual-monitor", "1280x720"]);
        if !x11 {
            mutter.arg("--no-x11");
        }
        let child = mutter
            .arg(format!("--wayland-display={wayland_display}"))
            .env("XDG_RUNTIME_DIR", &dir)
            .env("DBUS_SESSION_BUS_ADDRESS", &address)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(log.try_clone()?)
            .spawn()?;
        let mut m = Self {
            child,
            bus_pid,
            dir,
            wayland_display,
            display: String::new(),
        };

        if !x11 {
            let wl = m.dir.join(&m.wayland_display);
            wait_for(Duration::from_secs(20), || wl.exists().then_some(()))
                .ok_or("mutter did not become ready")?;
            return Ok(m);
        }
        let display = wait_for(Duration::from_secs(20), || {
            let log = std::fs::read_to_string(&log_path).ok()?;
            let rest = log.split("Using public X11 display :").nth(1)?;
            let n: String = rest.chars().take_while(char::is_ascii_digit).collect();
            let sock = format!("/tmp/.X11-unix/X{n}");
            let wl = m.dir.join(&m.wayland_display);
            (!n.is_empty() && Path::new(&sock).exists() && wl.exists()).then(|| format!(":{n}"))
        })
        .ok_or("mutter did not become ready")?;
        m.display = display;
        Ok(m)
    }

    /// Private runtime dir holding the Wayland socket and the Xauthority file.
    pub fn runtime_dir(&self) -> &Path {
        &self.dir
    }

    /// Path of the Wayland socket.
    pub fn wayland_socket(&self) -> PathBuf {
        self.dir.join(&self.wayland_display)
    }

    /// `$XDG_RUNTIME_DIR/.mutter-Xwaylandauth.*`
    pub fn xauthority(&self) -> Option<PathBuf> {
        wait_for(Duration::from_secs(10), || {
            std::fs::read_dir(&self.dir)
                .ok()?
                .flatten()
                .map(|e| e.path())
                .find(|p| {
                    p.file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with(".mutter-Xwaylandauth."))
                })
        })
    }
}

impl Drop for Mutter {
    fn drop(&mut self) {
        use rustix::process::{Pid, Signal, kill_process};
        if let Some(pid) = Pid::from_raw(i32::try_from(self.child.id()).unwrap_or(0)) {
            let _ = kill_process(pid, Signal::TERM);
        }
        if wait_for(Duration::from_secs(3), || {
            self.child.try_wait().ok().flatten()
        })
        .is_none()
        {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if let Some(pid) = self.bus_pid.and_then(Pid::from_raw) {
            let _ = kill_process(pid, Signal::TERM);
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
