//! `rstory install-autostart`: an XDG autostart entry (default) or a systemd user unit.
//!
//! Every path and program is injected through [`Env`]; only [`Env::from_process`] (used by
//! [`run`]) reads the real session. Tests build an [`Env`] over temp dirs and fake scripts, so
//! nothing here can reach the real `~/.config` or the real `systemctl` by accident.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Desktop entry template; `{exec}` is the quoted `Exec` value.
pub const DESKTOP_TEMPLATE: &str = "[Desktop Entry]\n\
Type=Application\n\
Name=rstory\n\
Exec={exec} daemon\n\
X-GNOME-Autostart-enabled=true\n\
NoDisplay=true\n";

/// systemd user unit template; `{exec}` is the quoted binary path.
pub const UNIT_TEMPLATE: &str = "[Unit]\n\
Description=rstory clipboard history daemon\n\
PartOf=graphical-session.target\n\
After=graphical-session.target\n\
\n\
[Service]\n\
ExecStart={exec} daemon\n\
Restart=on-failure\n\
RestartSec=1\n\
\n\
[Install]\n\
WantedBy=graphical-session.target\n";

pub const DESKTOP_NAME: &str = "rstory.desktop";
pub const UNIT_NAME: &str = "rstory.service";

/// Exit code for "no systemd user instance" (contracts/cli.md: environment error).
pub const EXIT_ENVIRONMENT: u8 = 5;
const EXIT_USAGE: u8 = 2;
const EXIT_UNEXPECTED: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutostartKind {
    Xdg,
    Systemd,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub systemd: bool,
    pub now: bool,
    pub uninstall: bool,
}

/// Everything the installer touches, injected. There is deliberately no `Default`.
#[derive(Clone, Debug)]
pub struct Env {
    /// The directory that holds `autostart/` and `systemd/user/` (`$XDG_CONFIG_HOME`).
    pub config_home: PathBuf,
    /// Absolute path of the rstory binary; written into the entry and run for `--now` on XDG.
    pub exe: PathBuf,
    /// The `systemctl` program (a path or a name looked up on `PATH`).
    pub systemctl: PathBuf,
    /// Directories that count as temporary when warning about the binary's location.
    pub temp_roots: Vec<PathBuf>,
}

impl Env {
    /// The real session. Only [`run`] calls this.
    pub fn from_process() -> Result<Env, String> {
        let config_home = match std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
            Some(v) => PathBuf::from(v),
            None => match std::env::var_os("HOME").filter(|v| !v.is_empty()) {
                Some(h) => PathBuf::from(h).join(".config"),
                None => return Err("neither XDG_CONFIG_HOME nor HOME is set".into()),
            },
        };
        let exe =
            std::env::current_exe().map_err(|e| format!("cannot find the rstory binary: {e}"))?;
        Ok(Env {
            config_home,
            exe,
            systemctl: PathBuf::from("systemctl"),
            temp_roots: vec![
                std::env::temp_dir(),
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
            ],
        })
    }
}

pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut o = Options::default();
    for a in args {
        match a.as_str() {
            "--systemd" => o.systemd = true,
            "--now" => o.now = true,
            "--uninstall" => o.uninstall = true,
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    if o.uninstall && (o.systemd || o.now) {
        return Err("--uninstall cannot be combined with --systemd or --now".into());
    }
    Ok(o)
}

pub fn xdg_path(config_home: &Path) -> PathBuf {
    config_home.join("autostart").join(DESKTOP_NAME)
}

pub fn unit_path(config_home: &Path) -> PathBuf {
    config_home.join("systemd").join("user").join(UNIT_NAME)
}

/// Which autostart kinds are installed (for `doctor`, FR-005).
pub fn detect(config_home: &Path) -> Vec<AutostartKind> {
    let mut v = Vec::new();
    if xdg_path(config_home).is_file() {
        v.push(AutostartKind::Xdg);
    }
    if unit_path(config_home).is_file() {
        v.push(AutostartKind::Systemd);
    }
    v
}

/// Quotes an `Exec` argument per the Desktop Entry spec, including the string-level escaping.
fn quote_desktop_exec(path: &str) -> String {
    const RESERVED: &[char] = &[
        ' ', '\t', '\n', '"', '\'', '\\', '>', '<', '~', '|', '&', ';', '$', '*', '?', '#', '(',
        ')', '`',
    ];
    let path = path.replace('%', "%%");
    if !path.contains(RESERVED) {
        return path;
    }
    let mut out = String::from("\"");
    for c in path.chars() {
        match c {
            '"' | '`' | '$' => {
                out.push_str("\\\\");
                out.push(c);
            }
            '\\' => out.push_str("\\\\\\\\"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Quotes a systemd `ExecStart` word.
fn quote_unit_exec(path: &str) -> String {
    let path = path.replace('%', "%%").replace('$', "$$");
    if !path.contains([' ', '\t', '"', '\'', '\\']) {
        return path;
    }
    format!("\"{}\"", path.replace('\\', "\\\\").replace('"', "\\\""))
}

pub fn desktop_entry(exe: &Path) -> String {
    DESKTOP_TEMPLATE.replace("{exec}", &quote_desktop_exec(&exe.to_string_lossy()))
}

pub fn unit_file(exe: &Path) -> String {
    UNIT_TEMPLATE.replace("{exec}", &quote_unit_exec(&exe.to_string_lossy()))
}

/// True when the binary sits somewhere a user would not keep it (a temp dir or a cargo target dir).
pub fn is_temporary_path(exe: &Path, temp_roots: &[PathBuf]) -> bool {
    if temp_roots.iter().any(|r| exe.starts_with(r)) {
        return true;
    }
    let comps: Vec<_> = exe.components().map(|c| c.as_os_str()).collect();
    comps
        .windows(2)
        .any(|w| w[0] == "target" && (w[1] == "debug" || w[1] == "release"))
}

/// Temp file in the same directory, then rename. Never needs root.
fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| io::Error::other("path has no parent"))?;
    fs::create_dir_all(dir)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = (|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
        fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn remove_if_exists(path: &Path) -> io::Result<bool> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

fn systemctl(env: &Env, args: &[&str]) -> io::Result<bool> {
    let status = Command::new(&env.systemctl)
        .arg("--user")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(status.success())
}

fn spawn_detached_daemon(env: &Env) -> io::Result<()> {
    use std::os::unix::process::CommandExt;
    let mut child = Command::new(&env.exe)
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()?;
    // Reap in the background so a short-lived child leaves no zombie; the daemon outlives us.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Runs the command against an injected environment. Returns the process exit code.
pub fn execute(env: &Env, opts: Options, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match execute_inner(env, opts, out, err) {
        Ok(code) => code,
        Err(e) => {
            let _ = writeln!(err, "rstory install-autostart: {e}");
            EXIT_UNEXPECTED
        }
    }
}

fn execute_inner(
    env: &Env,
    opts: Options,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> io::Result<u8> {
    let xdg = xdg_path(&env.config_home);
    let unit = unit_path(&env.config_home);

    if opts.uninstall {
        let mut removed = false;
        if unit.is_file() {
            // Best effort: the unit may never have been enabled, or systemd may be gone.
            let _ = systemctl(env, &["disable", "--now", UNIT_NAME]);
            removed |= remove_if_exists(&unit)?;
            let _ = systemctl(env, &["daemon-reload"]);
        }
        removed |= remove_if_exists(&xdg)?;
        writeln!(
            out,
            "{}",
            if removed {
                "Autostart removed."
            } else {
                "Nothing to remove."
            }
        )?;
        return Ok(0);
    }

    if is_temporary_path(&env.exe, &env.temp_roots) {
        writeln!(
            err,
            "Warning: rstory is running from {}; autostart will point there. Install the binary to ~/.local/bin first.",
            env.exe.display()
        )?;
    }

    if opts.systemd {
        if !matches!(systemctl(env, &["show-environment"]), Ok(true)) {
            writeln!(
                err,
                "systemd user instance not available. Run rstory install-autostart without --systemd."
            )?;
            return Ok(EXIT_ENVIRONMENT);
        }
        atomic_write(&unit, unit_file(&env.exe).as_bytes())?;
        remove_if_exists(&xdg)?;
        let _ = systemctl(env, &["daemon-reload"]);
        let enable: &[&str] = if opts.now {
            &["enable", "--now", UNIT_NAME]
        } else {
            &["enable", UNIT_NAME]
        };
        if !matches!(systemctl(env, enable), Ok(true)) {
            writeln!(
                err,
                "systemctl --user enable failed; the unit file was written to {}.",
                unit.display()
            )?;
            return Ok(EXIT_UNEXPECTED);
        }
        writeln!(
            out,
            "Autostart installed: {} (systemd user unit).",
            unit.display()
        )?;
        if opts.now {
            writeln!(out, "Daemon started.")?;
        }
    } else {
        atomic_write(&xdg, desktop_entry(&env.exe).as_bytes())?;
        writeln!(
            out,
            "{}",
            crate::ui::copy::format(
                crate::ui::copy::INSTALL_AUTOSTART_DONE,
                &[("path", &xdg.display().to_string())]
            )
        )?;
        if opts.now {
            spawn_detached_daemon(env)?;
            writeln!(out, "Daemon started.")?;
        }
    }
    Ok(0)
}

/// Entry point for `rstory install-autostart <args>`; uses the real session.
/// `rstory install-autostart --help`.
pub const HELP: &str = "Start the rstory daemon at login.

Usage: rstory install-autostart [--systemd] [--now] [--uninstall]

  --systemd    write a systemd user unit instead of an XDG autostart entry
  --now        also start the daemon now
  --uninstall  remove both kinds

Examples:
  rstory install-autostart
  rstory install-autostart --systemd --now
";

pub fn run(args: &[String]) -> u8 {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{HELP}");
        return 0;
    }
    let opts = match parse_args(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("rstory install-autostart: {e}");
            eprintln!("usage: rstory install-autostart [--systemd] [--now] [--uninstall]");
            return EXIT_USAGE;
        }
    };
    let env = match Env::from_process() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("rstory install-autostart: {e}");
            return EXIT_ENVIRONMENT;
        }
    };
    execute(&env, opts, &mut io::stdout(), &mut io::stderr())
}
