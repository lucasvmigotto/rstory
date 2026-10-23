//! `rstory install-gnome-extension`: writes the embedded GNOME Shell extension into the user's
//! extensions directory (spec 010, ADR-0008).
//!
//! Every path and program is injected through [`Env`]; only [`Env::from_process`] (used by
//! [`run`]) reads the real session. Tests build an [`Env`] over temp dirs and a fake
//! `gnome-extensions` script, so nothing here can reach the real `~/.local/share/gnome-shell`
//! or the real `gnome-extensions` by accident.

use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use include_dir::{Dir, include_dir};

/// The extension's source, embedded at build time (FR-001).
static EXTENSION: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/extension");

/// The extension's uuid (architecture §13).
pub const UUID: &str = "rstory@lucasvmigotto.github.io";

const EXIT_USAGE: u8 = 2;
const EXIT_UNEXPECTED: u8 = 1;
/// Environment error (contracts/cli.md): no data directory.
const EXIT_ENVIRONMENT: u8 = 5;

const MSG_NOT_FOUND: &str = "gnome-extensions not found; enable it in the Extensions app.";
const MSG_NOT_GNOME: &str = "This session is not GNOME; installed anyway.";
const MSG_REMOVED: &str = "GNOME extension removed.";
const MSG_NOTHING: &str = "Nothing to remove.";
const MSG_ENABLED: &str = "Enabled.";

/// Top-level entries of `extension/` that are development tooling, never installed.
const DEV_ONLY: &[&str] = &[
    "tests",
    "node_modules",
    "package.json",
    "package-lock.json",
    "jsconfig.json",
    "eslint.config.js",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub enable: bool,
    pub uninstall: bool,
}

/// Everything the installer touches, injected. There is deliberately no `Default`.
#[derive(Clone, Debug)]
pub struct Env {
    /// `$XDG_DATA_HOME`: the directory that holds `gnome-shell/extensions/`.
    pub data_home: PathBuf,
    /// The `gnome-extensions` program (a path or a name looked up on `PATH`).
    pub gnome_extensions: PathBuf,
    /// `$XDG_CURRENT_DESKTOP`, for the "not GNOME" warning.
    pub current_desktop: Option<String>,
}

impl Env {
    /// The real session. Only [`run`] calls this.
    ///
    /// # Errors
    /// A message when neither `XDG_DATA_HOME` nor `HOME` is set.
    pub fn from_process() -> Result<Env, String> {
        let data_home = match std::env::var_os("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
            Some(v) => PathBuf::from(v),
            None => match std::env::var_os("HOME").filter(|v| !v.is_empty()) {
                Some(h) => PathBuf::from(h).join(".local/share"),
                None => return Err("neither XDG_DATA_HOME nor HOME is set".into()),
            },
        };
        Ok(Env {
            data_home,
            gnome_extensions: PathBuf::from("gnome-extensions"),
            current_desktop: std::env::var("XDG_CURRENT_DESKTOP").ok(),
        })
    }
}

/// What `doctor` and the data model call an extension install (data-model.md).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtensionInstall {
    pub path: PathBuf,
    /// `version-name` from the installed `metadata.json`.
    pub installed_version: Option<String>,
    /// From `gnome-extensions info`; `None` when the program is unavailable or silent.
    pub enabled: Option<bool>,
}

/// Parses the arguments after `install-gnome-extension`.
///
/// # Errors
/// A message for an unknown argument or the incompatible `--enable --uninstall`.
pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut o = Options::default();
    for a in args {
        match a.as_str() {
            "--enable" => o.enable = true,
            "--uninstall" => o.uninstall = true,
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    if o.enable && o.uninstall {
        return Err("--enable cannot be combined with --uninstall".into());
    }
    Ok(o)
}

/// `<data_home>/gnome-shell/extensions/<uuid>`.
#[must_use]
pub fn extension_dir(data_home: &Path) -> PathBuf {
    data_home.join("gnome-shell").join("extensions").join(UUID)
}

/// The files that get installed, as paths relative to the extension directory.
#[must_use]
pub fn embedded_files() -> Vec<(PathBuf, &'static [u8])> {
    fn walk(dir: &'static Dir<'static>, out: &mut Vec<(PathBuf, &'static [u8])>) {
        for f in dir.files() {
            out.push((f.path().to_path_buf(), f.contents()));
        }
        for d in dir.dirs() {
            walk(d, out);
        }
    }
    let mut all = Vec::new();
    walk(&EXTENSION, &mut all);
    all.retain(|(p, _)| {
        !matches!(p.components().next(), Some(Component::Normal(c))
            if DEV_ONLY.iter().any(|d| c == *d))
    });
    all.sort();
    all
}

/// `version-name` of the embedded `metadata.json`.
#[must_use]
pub fn embedded_version() -> Option<String> {
    let (_, bytes) = embedded_files()
        .into_iter()
        .find(|(p, _)| p == Path::new("metadata.json"))?;
    version_name(&String::from_utf8_lossy(bytes))
}

fn version_name(metadata: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(metadata).ok()?;
    v.get("version-name")?.as_str().map(str::to_string)
}

fn installed_version(dir: &Path) -> Option<String> {
    version_name(&fs::read_to_string(dir.join("metadata.json")).ok()?)
}

/// Reads the installed version and, when `gnome-extensions` answers, the enabled state.
#[must_use]
pub fn inspect(env: &Env) -> ExtensionInstall {
    let path = extension_dir(&env.data_home);
    // Nothing installed: nothing to ask the Shell about.
    let enabled = path
        .is_dir()
        .then(|| {
            Command::new(&env.gnome_extensions)
                .args(["info", UUID])
                .stdin(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| parse_enabled(&String::from_utf8_lossy(&o.stdout)))
        })
        .flatten();
    ExtensionInstall {
        installed_version: installed_version(&path),
        path,
        enabled,
    }
}

fn parse_enabled(info: &str) -> Option<bool> {
    info.lines().find_map(|l| {
        let (key, value) = l.trim().split_once(':')?;
        (key.trim() == "Enabled").then(|| value.trim().eq_ignore_ascii_case("yes"))
    })
}

fn is_gnome(desktop: Option<&str>) -> bool {
    desktop.is_some_and(|d| d.split(':').any(|p| p.eq_ignore_ascii_case("gnome")))
}

/// Writes the files into a sibling staging directory, then swaps it in, so a failure never
/// leaves a half-written extension behind.
fn write_extension(env: &Env) -> io::Result<()> {
    let target = extension_dir(&env.data_home);
    let shell_dir = env.data_home.join("gnome-shell");
    fs::create_dir_all(target.parent().unwrap_or(&shell_dir))?;
    let staging = shell_dir.join(format!(".rstory-extension.{}.tmp", std::process::id()));
    let _ = fs::remove_dir_all(&staging);
    let result = (|| {
        for (rel, bytes) in embedded_files() {
            let dest = staging.join(&rel);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dest, bytes)?;
        }
        remove_dir_if_exists(&target)?;
        fs::rename(&staging, &target)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn remove_dir_if_exists(dir: &Path) -> io::Result<bool> {
    match fs::remove_dir_all(dir) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

/// Outcome of running `gnome-extensions <verb> <uuid>`.
enum Ran {
    Ok,
    Failed,
    NotFound,
}

fn gnome_extensions(env: &Env, verb: &str) -> Ran {
    match Command::new(&env.gnome_extensions)
        .args([verb, UUID])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(s) if s.success() => Ran::Ok,
        Ok(_) => Ran::Failed,
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ran::NotFound,
        Err(_) => Ran::Failed,
    }
}

/// Runs the command against an injected environment. Returns the process exit code.
pub fn execute(env: &Env, opts: Options, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match execute_inner(env, opts, out, err) {
        Ok(code) => code,
        Err(e) => {
            let _ = writeln!(err, "rstory install-gnome-extension: {e}");
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
    let target = extension_dir(&env.data_home);

    if opts.uninstall {
        if !target.exists() {
            writeln!(out, "{MSG_NOTHING}")?;
            return Ok(0);
        }
        // Best effort: the Shell may not know the extension, or gnome-extensions may be absent.
        let _ = gnome_extensions(env, "disable");
        remove_dir_if_exists(&target)?;
        writeln!(out, "{MSG_REMOVED}")?;
        return Ok(0);
    }

    if !is_gnome(env.current_desktop.as_deref()) {
        writeln!(err, "{MSG_NOT_GNOME}")?;
    }
    let embedded = embedded_version();
    let previous = installed_version(&target);
    write_extension(env)?;
    match (&previous, &embedded) {
        (Some(old), Some(new)) if old != new => writeln!(
            out,
            "GNOME extension updated to {new}. Log out and back in to load it."
        )?,
        _ => writeln!(out, "{}", crate::ui::copy::INSTALL_GNOME_DONE)?,
    }

    if opts.enable {
        match gnome_extensions(env, "enable") {
            Ran::Ok => writeln!(out, "{MSG_ENABLED}")?,
            Ran::NotFound => writeln!(err, "{MSG_NOT_FOUND}")?,
            // The Shell only sees a new extension after a re-login, so a failure here is the
            // normal first-install case: the files are in place, tell the user what to do.
            Ran::Failed => writeln!(
                err,
                "gnome-extensions enable did not succeed (the Shell may need a re-login to see \
                 the new extension). Log out and back in, then run: gnome-extensions enable {UUID}"
            )?,
        }
    }
    Ok(0)
}

/// Entry point for `rstory install-gnome-extension <args>`; uses the real session.
#[must_use]
pub fn run(args: &[String]) -> u8 {
    let opts = match parse_args(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("rstory install-gnome-extension: {e}");
            eprintln!("usage: rstory install-gnome-extension [--enable | --uninstall]");
            return EXIT_USAGE;
        }
    };
    let env = match Env::from_process() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("rstory install-gnome-extension: {e}");
            return EXIT_ENVIRONMENT;
        }
    };
    execute(&env, opts, &mut io::stdout(), &mut io::stderr())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_binary_never_embeds_node_modules() {
        // `npm ci` belongs in the `rstory-tools-npm` volume (`make ext-deps`); a populated
        // `extension/node_modules` on the host would be compiled into the binary.
        let embedded = EXTENSION
            .get_dir("node_modules")
            .map_or(0, |d| d.entries().len());
        assert_eq!(
            embedded, 0,
            "extension/node_modules is populated; remove it and use `make ext-deps`"
        );
    }
}
