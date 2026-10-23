//! Where the CLI finds the daemon socket, the store and the config file. The same rules as the
//! daemon (spec 004): `XDG_RUNTIME_DIR`, `XDG_STATE_HOME`/`HOME`, `XDG_CONFIG_HOME`/`HOME`.
//! Everything is read once into [`Env`], so tests pass their own directories.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::error::CliError;
use crate::ipc::server;

const VARS: [&str; 4] = [
    "XDG_RUNTIME_DIR",
    "XDG_STATE_HOME",
    "XDG_CONFIG_HOME",
    "HOME",
];

/// The environment the paths derive from.
#[derive(Debug, Clone)]
pub struct Env {
    vars: HashMap<&'static str, String>,
    uid: u32,
    run_user_root: PathBuf,
}

impl Env {
    /// The process environment and uid.
    #[must_use]
    pub fn process() -> Self {
        let pairs: Vec<(&'static str, String)> = VARS
            .iter()
            .filter_map(|k| std::env::var(k).ok().map(|v| (*k, v)))
            .collect();
        Self::new(
            &pairs,
            rustix::process::getuid().as_raw(),
            Path::new("/run/user"),
        )
    }

    /// An environment of the given variables (empty values count as unset).
    #[must_use]
    pub fn new(vars: &[(&'static str, String)], uid: u32, run_user_root: &Path) -> Self {
        Self {
            vars: vars
                .iter()
                .filter(|(_, v)| !v.is_empty())
                .cloned()
                .collect(),
            uid,
            run_user_root: run_user_root.to_path_buf(),
        }
    }

    fn var(&self, name: &str) -> Option<&str> {
        self.vars.get(name).map(String::as_str)
    }

    /// The control socket path.
    ///
    /// # Errors
    /// An environment error (exit 5) when there is no runtime directory.
    pub fn socket_path(&self) -> Result<PathBuf, CliError> {
        server::runtime_dir(self.var("XDG_RUNTIME_DIR"), self.uid, &self.run_user_root)
            .map(|dir| server::socket_path(&dir))
            .map_err(|e| CliError::environment(e.to_string()))
    }

    /// The store directory: `$XDG_STATE_HOME/rstory`, else `~/.local/state/rstory`.
    ///
    /// # Errors
    /// An environment error when neither variable is set.
    pub fn store_dir(&self) -> Result<PathBuf, CliError> {
        if let Some(state) = self.var("XDG_STATE_HOME") {
            return Ok(PathBuf::from(state).join("rstory"));
        }
        let home = self.var("HOME").ok_or_else(|| {
            CliError::environment(
                "Neither XDG_STATE_HOME nor HOME is set, so the store is unknown.",
            )
        })?;
        Ok(PathBuf::from(home).join(".local/state/rstory"))
    }

    /// The config file: `$XDG_CONFIG_HOME/rstory/config.toml`, else `~/.config/rstory/config.toml`.
    ///
    /// # Errors
    /// An environment error when neither variable is set.
    pub fn config_path(&self) -> Result<PathBuf, CliError> {
        let dir = match self.var("XDG_CONFIG_HOME") {
            Some(c) => PathBuf::from(c),
            None => PathBuf::from(self.var("HOME").ok_or_else(|| {
                CliError::environment(
                    "Neither XDG_CONFIG_HOME nor HOME is set, so the config path is unknown.",
                )
            })?)
            .join(".config"),
        };
        Ok(dir.join("rstory/config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ExitCode;

    fn env(pairs: &[(&'static str, &str)]) -> Env {
        let owned: Vec<_> = pairs.iter().map(|(k, v)| (*k, (*v).to_string())).collect();
        Env::new(&owned, 1000, Path::new("/nonexistent"))
    }

    #[test]
    fn xdg_variables_win_over_home() {
        let e = env(&[
            ("XDG_RUNTIME_DIR", "/r"),
            ("XDG_STATE_HOME", "/s"),
            ("XDG_CONFIG_HOME", "/c"),
            ("HOME", "/h"),
        ]);
        assert_eq!(
            e.socket_path().unwrap(),
            Path::new("/r/rstory/control.sock")
        );
        assert_eq!(e.store_dir().unwrap(), Path::new("/s/rstory"));
        assert_eq!(e.config_path().unwrap(), Path::new("/c/rstory/config.toml"));
    }

    #[test]
    fn home_gives_the_defaults() {
        let e = env(&[("HOME", "/h")]);
        assert_eq!(e.store_dir().unwrap(), Path::new("/h/.local/state/rstory"));
        assert_eq!(
            e.config_path().unwrap(),
            Path::new("/h/.config/rstory/config.toml")
        );
    }

    #[test]
    fn empty_values_count_as_unset() {
        let e = env(&[("XDG_STATE_HOME", ""), ("HOME", "/h")]);
        assert_eq!(e.store_dir().unwrap(), Path::new("/h/.local/state/rstory"));
    }

    #[test]
    fn nothing_set_is_an_environment_error() {
        let e = env(&[]);
        for r in [e.socket_path(), e.store_dir(), e.config_path()] {
            assert_eq!(r.unwrap_err().exit, ExitCode::Environment);
        }
    }
}
