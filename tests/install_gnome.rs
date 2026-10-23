//! Spec 010 (US1): `install-gnome-extension`. Every path and program is injected; nothing here
//! touches the real `~/.local/share/gnome-shell` or the real `gnome-extensions`.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use rstory::install::gnome::{
    Env, ExtensionInstall, Options, UUID, embedded_files, execute, extension_dir, inspect,
    parse_args,
};
use tempfile::TempDir;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DONE: &str = "GNOME extension installed. Log out and back in, then run: gnome-extensions enable rstory@lucasvmigotto.github.io\n";

struct Fixture {
    dir: TempDir,
    env: Env,
}

impl Fixture {
    /// `gnome_extensions_ok`: whether the fake `gnome-extensions` succeeds (None: no such program).
    fn new(gnome_extensions_ok: Option<bool>) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let program = match gnome_extensions_ok {
            Some(ok) => {
                let p = dir.path().join("fake-gnome-extensions");
                let code = if ok { 0 } else { 1 };
                write_script(
                    &p,
                    &format!(
                        "echo \"$@\" >> '{}'\n\
                         if [ \"$1\" = info ]; then printf '%s\\n' '  Name: rstory' '  State: ACTIVE' '  Enabled: Yes'; fi\n\
                         exit {code}\n",
                        dir.path().join("calls.log").display()
                    ),
                );
                p
            }
            None => dir.path().join("no-such-gnome-extensions"),
        };
        let env = Env {
            data_home: dir.path().join("data"),
            gnome_extensions: program,
            current_desktop: Some("GNOME".into()),
        };
        Fixture { dir, env }
    }

    fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.dir.path().join("calls.log"))
            .map(|s| s.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn target(&self) -> PathBuf {
        extension_dir(&self.env.data_home)
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

/// Serialises script creation and process spawning (a fork while another test holds a script
/// open for writing makes the exec fail with "text file busy").
static SPAWN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn spawn_guard() -> std::sync::MutexGuard<'static, ()> {
    SPAWN_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn write_script(path: &Path, body: &str) {
    let _g = spawn_guard();
    fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn plain() -> Options {
    Options::default()
}

fn enable() -> Options {
    Options {
        enable: true,
        ..Options::default()
    }
}

fn uninstall() -> Options {
    Options {
        uninstall: true,
        ..Options::default()
    }
}

fn metadata(dir: &Path) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(dir.join("metadata.json")).unwrap()).unwrap()
}

#[test]
fn the_uuid_is_the_documented_one() {
    assert_eq!(UUID, "rstory@lucasvmigotto.github.io");
    let f = Fixture::new(None);
    assert_eq!(
        f.target(),
        f.env
            .data_home
            .join("gnome-shell/extensions/rstory@lucasvmigotto.github.io")
    );
}

#[test]
fn fresh_install_writes_the_runtime_files_and_explains_the_relogin() {
    let f = Fixture::new(Some(true));
    let (code, out, err) = f.run(plain());
    assert_eq!((code, out.as_str(), err.as_str()), (0, DONE, ""));
    let t = f.target();
    for file in [
        "metadata.json",
        "extension.js",
        "client.js",
        "controller.js",
        "menu.js",
        "stylesheet.css",
        "icons/recording-symbolic.svg",
        "icons/paused-symbolic.svg",
        "icons/disconnected-symbolic.svg",
    ] {
        assert!(t.join(file).is_file(), "{file} is installed");
    }
    // Dev-only files never ship to the user's machine.
    for dev in [
        "tests",
        "node_modules",
        "package.json",
        "package-lock.json",
        "jsconfig.json",
        "eslint.config.js",
    ] {
        assert!(!t.join(dev).exists(), "{dev} is not installed");
    }
    assert!(f.calls().is_empty(), "no --enable: gnome-extensions is not run");
}

#[test]
fn metadata_declares_uuid_shell_versions_and_the_crate_version() {
    let f = Fixture::new(None);
    f.run(plain());
    let m = metadata(&f.target());
    assert_eq!(m["uuid"], UUID);
    assert_eq!(m["shell-version"], serde_json::json!(["48", "49", "50", "51"]));
    assert_eq!(m["version-name"], VERSION);
}

#[test]
fn installed_files_are_byte_equal_to_the_embedded_ones() {
    let f = Fixture::new(None);
    f.run(plain());
    let files = embedded_files();
    assert!(files.len() >= 9, "got {} embedded files", files.len());
    for (rel, bytes) in files {
        assert_eq!(fs::read(f.target().join(&rel)).unwrap(), bytes, "{rel:?}");
    }
}

#[test]
fn the_embedded_icons_are_the_design_system_icons() {
    for name in ["recording", "paused", "disconnected"] {
        let file = format!("{name}-symbolic.svg");
        let source = fs::read(Path::new("assets/icons").join(&file)).unwrap();
        let embedded = embedded_files()
            .into_iter()
            .find(|(p, _)| p == &PathBuf::from("icons").join(&file))
            .unwrap_or_else(|| panic!("{file} is embedded"));
        assert_eq!(embedded.1, source, "{file} drifted from assets/icons");
    }
}

#[test]
fn enable_runs_gnome_extensions_and_reports_it() {
    let f = Fixture::new(Some(true));
    let (code, out, err) = f.run(enable());
    assert_eq!(code, 0);
    assert_eq!(out, format!("{DONE}Enabled.\n"));
    assert_eq!(err, "");
    assert_eq!(f.calls(), [format!("enable {UUID}")]);
}

#[test]
fn enable_without_gnome_extensions_points_to_the_extensions_app() {
    let f = Fixture::new(None);
    let (code, out, err) = f.run(enable());
    assert_eq!(code, 0);
    assert_eq!(out, DONE);
    assert_eq!(
        err,
        "gnome-extensions not found; enable it in the Extensions app.\n"
    );
    assert!(f.target().join("metadata.json").is_file());
}

#[test]
fn enable_that_fails_keeps_the_install_and_says_what_to_do() {
    let f = Fixture::new(Some(false));
    let (code, out, err) = f.run(enable());
    assert_eq!(code, 0, "the files are installed; the Shell may just not see them yet");
    assert_eq!(out, DONE);
    assert!(err.contains("gnome-extensions enable did not succeed"), "{err}");
    assert!(err.contains(UUID), "{err}");
    assert!(f.target().join("metadata.json").is_file());
}

#[test]
fn reinstall_over_an_older_version_replaces_the_files() {
    let f = Fixture::new(None);
    let t = f.target();
    fs::create_dir_all(&t).unwrap();
    fs::write(
        t.join("metadata.json"),
        r#"{"uuid":"rstory@lucasvmigotto.github.io","version-name":"0.0.1"}"#,
    )
    .unwrap();
    fs::write(t.join("stale.js"), "old").unwrap();
    let (code, out, err) = f.run(plain());
    assert_eq!(code, 0);
    assert_eq!(
        out,
        format!("GNOME extension updated to {VERSION}. Log out and back in to load it.\n")
    );
    assert_eq!(err, "");
    assert!(!t.join("stale.js").exists(), "files of the old version are gone");
    assert_eq!(metadata(&t)["version-name"], VERSION);
}

#[test]
fn reinstall_of_the_same_version_is_idempotent() {
    let f = Fixture::new(None);
    f.run(plain());
    let before = fs::read(f.target().join("extension.js")).unwrap();
    let (code, out, _) = f.run(plain());
    assert_eq!((code, out.as_str()), (0, DONE));
    assert_eq!(fs::read(f.target().join("extension.js")).unwrap(), before);
}

#[test]
fn a_newer_installed_version_is_replaced_and_reported_as_updated() {
    let f = Fixture::new(None);
    let t = f.target();
    fs::create_dir_all(&t).unwrap();
    fs::write(t.join("metadata.json"), r#"{"version-name":"99.0.0"}"#).unwrap();
    let (_, out, _) = f.run(plain());
    assert!(out.starts_with("GNOME extension updated to "), "{out}");
    assert_eq!(metadata(&t)["version-name"], VERSION);
}

#[test]
fn uninstall_disables_then_removes_the_directory() {
    let f = Fixture::new(Some(true));
    f.run(plain());
    let (code, out, err) = f.run(uninstall());
    assert_eq!(
        (code, out.as_str(), err.as_str()),
        (0, "GNOME extension removed.\n", "")
    );
    assert!(!f.target().exists());
    assert_eq!(f.calls(), [format!("disable {UUID}")]);
}

#[test]
fn uninstall_works_without_gnome_extensions() {
    let f = Fixture::new(None);
    f.run(plain());
    let (code, out, _) = f.run(uninstall());
    assert_eq!((code, out.as_str()), (0, "GNOME extension removed.\n"));
    assert!(!f.target().exists());
}

#[test]
fn uninstall_when_nothing_is_installed_says_so() {
    let f = Fixture::new(Some(true));
    let (code, out, _) = f.run(uninstall());
    assert_eq!((code, out.as_str()), (0, "Nothing to remove.\n"));
    assert!(f.calls().is_empty(), "nothing to disable");
}

#[test]
fn a_non_gnome_session_warns_but_installs() {
    let mut f = Fixture::new(None);
    f.env.current_desktop = Some("KDE".into());
    let (code, out, err) = f.run(plain());
    assert_eq!(code, 0);
    assert_eq!(out, DONE);
    assert_eq!(err, "This session is not GNOME; installed anyway.\n");
    assert!(f.target().join("metadata.json").is_file());

    f.env.current_desktop = None;
    assert!(f.run(plain()).2.contains("not GNOME"));
}

#[test]
fn gnome_variants_do_not_warn() {
    for desktop in ["GNOME", "ubuntu:GNOME", "GNOME-Classic:GNOME", "gnome"] {
        let mut f = Fixture::new(None);
        f.env.current_desktop = Some(desktop.into());
        assert_eq!(f.run(plain()).2, "", "{desktop}");
    }
}

#[test]
fn arguments() {
    let a = |v: &[&str]| parse_args(&v.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    assert_eq!(a(&[]), Ok(Options::default()));
    assert_eq!(a(&["--enable"]).map(|o| o.enable), Ok(true));
    assert_eq!(a(&["--uninstall"]).map(|o| o.uninstall), Ok(true));
    assert!(a(&["--bogus"]).unwrap_err().contains("--bogus"));
    assert!(a(&["--enable", "--uninstall"]).is_err());
}

#[test]
fn inspect_reads_the_installed_version_and_the_enabled_state() {
    let f = Fixture::new(Some(true));
    assert_eq!(
        inspect(&f.env),
        ExtensionInstall {
            path: f.target(),
            installed_version: None,
            enabled: None
        }
    );
    f.run(plain());
    assert_eq!(
        inspect(&f.env),
        ExtensionInstall {
            path: f.target(),
            installed_version: Some(VERSION.into()),
            enabled: Some(true)
        }
    );
    assert_eq!(f.calls(), [format!("info {UUID}")]);
}

#[test]
fn inspect_without_gnome_extensions_leaves_enabled_unknown() {
    let f = Fixture::new(None);
    f.run(plain());
    let i = inspect(&f.env);
    assert_eq!(i.installed_version.as_deref(), Some(VERSION));
    assert_eq!(i.enabled, None);
}
