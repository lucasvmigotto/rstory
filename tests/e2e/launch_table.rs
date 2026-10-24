//! 008 T009: the launch table against the terminals installed in the tools container.
//!
//! No terminal is ever started. Each row of `TERMINALS` is resolved through a directory of stub
//! executables, which checks the argv the daemon would run. Rows whose program is really
//! installed in this container are also asked for `--version` (it prints and exits, with no
//! window). The tools image installs none of them today, so those checks report what was found.

use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

use rstory::launch::{LaunchPlan, PICKER_ARGV, TERMINALS, on_path, plan, which_in};

#[test]
fn every_row_of_the_table_resolves_to_its_flags_and_the_picker_argv() {
    let dir = tempfile::tempdir().unwrap();
    for (name, prefix) in TERMINALS {
        // A PATH holding only this row's stub: it must win, and nothing is run.
        let bin = dir.path().join(name);
        std::fs::write(&bin, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        let path = dir.path().to_string_lossy().into_owned();
        let found = plan(None, None, &|n| which_in(&path, n));
        let LaunchPlan::Found { argv, .. } = found else {
            panic!("{name} not resolved");
        };
        let mut want: Vec<String> = vec![(*name).to_string()];
        want.extend(prefix.iter().map(|s| (*s).to_string()));
        want.extend(PICKER_ARGV.iter().map(|s| (*s).to_string()));
        assert_eq!(argv, want, "{name}");
        std::fs::remove_file(&bin).unwrap();
    }
    assert!(matches!(
        plan(None, None, &|_| false),
        LaunchPlan::NotFound { .. }
    ));
}

#[test]
fn terminals_installed_in_this_container_answer_version() {
    let installed: Vec<&str> = TERMINALS
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| on_path(n))
        .collect();
    eprintln!(
        "terminals installed in this container: {}",
        if installed.is_empty() {
            "none".to_string()
        } else {
            installed.join(", ")
        }
    );
    // Only inside the tools container (podman or docker marker): never start anything on a host.
    let in_container = std::path::Path::new("/run/.containerenv").exists()
        || std::path::Path::new("/.dockerenv").exists();
    for name in installed.into_iter().filter(|_| in_container) {
        // `--version` prints and exits for every terminal in the table; no window is created.
        let ok = Command::new(name)
            .arg("--version")
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        assert!(ok, "{name} --version");
    }
}
