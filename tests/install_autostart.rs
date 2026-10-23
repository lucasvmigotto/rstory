//! Spec 012: autostart. Every path and program is injected; nothing here touches the real
//! `~/.config` or the real `systemctl`.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use rstory::install::autostart::{
    AutostartKind, Env, Options, detect, execute, is_temporary_path, parse_args, unit_path,
    xdg_path,
};
use tempfile::TempDir;

const DESKTOP: &str = "[Desktop Entry]\nType=Application\nName=rstory\nExec=/opt/bin/rstory daemon\nX-GNOME-Autostart-enabled=true\nNoDisplay=true\n";
const UNIT: &str = "[Unit]\nDescription=rstory clipboard history daemon\nPartOf=graphical-session.target\nAfter=graphical-session.target\n\n[Service]\nExecStart=/opt/bin/rstory daemon\nRestart=on-failure\nRestartSec=1\n\n[Install]\nWantedBy=graphical-session.target\n";

struct Fixture {
    dir: TempDir,
    env: Env,
}

impl Fixture {
    /// `systemctl_ok`: whether the fake `systemctl` succeeds (None: no such program at all).
    fn new(exe: &str, systemctl_ok: Option<bool>) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let systemctl = match systemctl_ok {
            Some(ok) => {
                let p = dir.path().join("fake-systemctl");
                let code = if ok { 0 } else { 1 };
                write_script(
                    &p,
                    &format!(
                        "echo \"$@\" >> '{}'\nexit {code}\n",
                        dir.path().join("systemctl.log").display()
                    ),
                );
                p
            }
            None => dir.path().join("no-such-systemctl"),
        };
        let env = Env {
            config_home: dir.path().join("config"),
            exe: PathBuf::from(exe),
            systemctl,
            temp_roots: vec![dir.path().to_path_buf()],
        };
        Fixture { dir, env }
    }

    fn systemctl_calls(&self) -> Vec<String> {
        fs::read_to_string(self.dir.path().join("systemctl.log"))
            .map(|s| s.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn run(&self, opts: Options) -> (u8, String, String) {
        let _g = spawn_guard();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = execute(&self.env, opts, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }
}

/// Serialises script creation and process spawning: a fork in one test while another test holds
/// a script open for writing makes the exec fail with "text file busy".
static SPAWN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn spawn_guard() -> std::sync::MutexGuard<'static, ()> {
    SPAWN_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn write_script(path: &Path, body: &str) {
    let _g = spawn_guard();
    fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn install() -> Options {
    Options::default()
}

#[test]
fn xdg_entry_is_byte_exact() {
    let f = Fixture::new("/opt/bin/rstory", Some(true));
    let (code, out, _) = f.run(install());
    assert_eq!(code, 0);
    let path = xdg_path(&f.env.config_home);
    assert_eq!(fs::read_to_string(&path).unwrap(), DESKTOP);
    assert_eq!(
        out,
        format!(
            "Autostart installed: {}. rstory starts at your next login, or run `rstory daemon` now.\n",
            path.display()
        )
    );
    assert!(f.systemctl_calls().is_empty());
}

#[test]
fn xdg_install_is_idempotent() {
    let f = Fixture::new("/opt/bin/rstory", Some(true));
    f.run(install());
    let (code, _, _) = f.run(install());
    assert_eq!(code, 0);
    assert_eq!(
        fs::read_to_string(xdg_path(&f.env.config_home)).unwrap(),
        DESKTOP
    );
    let leftovers: Vec<_> = fs::read_dir(f.env.config_home.join("autostart"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(leftovers.len(), 1, "no temp files left: {leftovers:?}");
}

#[test]
fn exec_with_spaces_is_quoted() {
    let f = Fixture::new("/opt/my bin/rstory", Some(true));
    f.run(install());
    let text = fs::read_to_string(xdg_path(&f.env.config_home)).unwrap();
    assert!(
        text.contains("Exec=\"/opt/my bin/rstory\" daemon\n"),
        "{text}"
    );
}

#[test]
fn uninstall_removes_xdg_entry() {
    let f = Fixture::new("/opt/bin/rstory", Some(true));
    f.run(install());
    let (code, out, _) = f.run(Options {
        uninstall: true,
        ..install()
    });
    assert_eq!(code, 0);
    assert_eq!(out, "Autostart removed.\n");
    assert!(!xdg_path(&f.env.config_home).exists());
    assert!(f.systemctl_calls().is_empty(), "no unit, so no systemctl");
}

#[test]
fn uninstall_with_nothing_installed() {
    let f = Fixture::new("/opt/bin/rstory", Some(true));
    let (code, out, _) = f.run(Options {
        uninstall: true,
        ..install()
    });
    assert_eq!((code, out.as_str()), (0, "Nothing to remove.\n"));
}

#[test]
fn temp_path_warns_on_stderr_but_installs() {
    let f = Fixture::new("/placeholder", Some(true));
    let mut f = f;
    f.env.exe = f.dir.path().join("rstory");
    let (code, _, err) = f.run(install());
    assert_eq!(code, 0);
    assert_eq!(
        err,
        format!(
            "Warning: rstory is running from {}; autostart will point there. Install the binary to ~/.local/bin first.\n",
            f.env.exe.display()
        )
    );
    assert!(xdg_path(&f.env.config_home).is_file());
}

#[test]
fn installed_binary_does_not_warn() {
    let f = Fixture::new("/opt/bin/rstory", Some(true));
    let (_, _, err) = f.run(install());
    assert_eq!(err, "");
}

#[test]
fn temporary_path_detection() {
    let roots = vec![PathBuf::from("/tmp")];
    assert!(is_temporary_path(Path::new("/tmp/x/rstory"), &roots));
    assert!(is_temporary_path(
        Path::new("/home/u/src/rstory/target/debug/rstory"),
        &roots
    ));
    assert!(is_temporary_path(
        Path::new("/home/u/src/rstory/target/release/rstory"),
        &roots
    ));
    assert!(!is_temporary_path(
        Path::new("/home/u/.local/bin/rstory"),
        &roots
    ));
}

#[test]
fn parse_args_cases() {
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    assert_eq!(parse_args(&s(&[])).unwrap(), Options::default());
    assert_eq!(
        parse_args(&s(&["--systemd", "--now"])).unwrap(),
        Options {
            systemd: true,
            now: true,
            uninstall: false
        }
    );
    assert!(parse_args(&s(&["--bogus"])).is_err());
    assert!(parse_args(&s(&["--uninstall", "--now"])).is_err());
}

#[test]
fn systemd_unit_is_byte_exact_and_removes_xdg_entry() {
    let f = Fixture::new("/opt/bin/rstory", Some(true));
    f.run(install());
    assert!(xdg_path(&f.env.config_home).exists());
    let (code, out, _) = f.run(Options {
        systemd: true,
        ..install()
    });
    assert_eq!(code, 0);
    let unit = unit_path(&f.env.config_home);
    assert_eq!(fs::read_to_string(&unit).unwrap(), UNIT);
    assert!(
        !xdg_path(&f.env.config_home).exists(),
        "no XDG entry left behind"
    );
    assert_eq!(
        out,
        format!(
            "Autostart installed: {} (systemd user unit).\n",
            unit.display()
        )
    );
    assert_eq!(detect(&f.env.config_home), vec![AutostartKind::Systemd]);
}

#[test]
fn systemd_now_calls_enable_now() {
    let f = Fixture::new("/opt/bin/rstory", Some(true));
    let (code, out, _) = f.run(Options {
        systemd: true,
        now: true,
        uninstall: false,
    });
    assert_eq!(code, 0);
    assert!(out.ends_with("Daemon started.\n"), "{out}");
    let calls = f.systemctl_calls();
    assert!(
        calls.contains(&"--user enable --now rstory.service".to_string()),
        "{calls:?}"
    );
}

#[test]
fn systemd_without_now_enables_but_does_not_start() {
    let f = Fixture::new("/opt/bin/rstory", Some(true));
    f.run(Options {
        systemd: true,
        ..install()
    });
    let calls = f.systemctl_calls();
    assert!(
        calls.contains(&"--user enable rstory.service".to_string()),
        "{calls:?}"
    );
    assert!(calls.iter().all(|c| !c.contains("--now")), "{calls:?}");
}

#[test]
fn systemd_unavailable_exits_5_and_writes_nothing() {
    for ok in [None, Some(false)] {
        let f = Fixture::new("/opt/bin/rstory", ok);
        let (code, out, err) = f.run(Options {
            systemd: true,
            ..install()
        });
        assert_eq!(code, 5);
        assert_eq!(out, "");
        assert_eq!(
            err,
            "systemd user instance not available. Run rstory install-autostart without --systemd.\n"
        );
        assert!(detect(&f.env.config_home).is_empty());
    }
}

#[test]
fn uninstall_removes_unit_and_disables_it() {
    let f = Fixture::new("/opt/bin/rstory", Some(true));
    f.run(Options {
        systemd: true,
        ..install()
    });
    let (code, out, _) = f.run(Options {
        uninstall: true,
        ..install()
    });
    assert_eq!((code, out.as_str()), (0, "Autostart removed.\n"));
    assert!(detect(&f.env.config_home).is_empty());
    assert!(
        f.systemctl_calls()
            .contains(&"--user disable --now rstory.service".to_string())
    );
}

#[test]
fn xdg_now_spawns_the_daemon_detached() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("daemon-ran");
    let exe = dir.path().join("fake-rstory");
    write_script(&exe, &format!("echo \"$@\" > '{}'\n", marker.display()));
    let env = Env {
        config_home: dir.path().join("config"),
        exe,
        systemctl: dir.path().join("no-such-systemctl"),
        temp_roots: vec![],
    };
    let _g = spawn_guard();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = execute(
        &env,
        Options {
            now: true,
            ..install()
        },
        &mut out,
        &mut err,
    );
    assert_eq!(code, 0);
    assert!(
        String::from_utf8(out)
            .unwrap()
            .ends_with("Daemon started.\n")
    );
    for _ in 0..100 {
        if marker.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert_eq!(fs::read_to_string(&marker).unwrap(), "daemon\n");
}
