//! E2E-K1 First run (specs/004 qa.md): fresh XDG dirs on Xvfb; `install-autostart` writes the
//! desktop entry, the daemon starts, `status` reports the session, backend, store and daemon.
//! `doctor` (spec 011) is not on dev yet: T012 reuses these assertions on its output.

use crate::support::{BIN, World};

#[test]
fn k1_first_run_installs_autostart_and_starts_the_daemon() {
    let w = World::xvfb();

    // Nothing exists yet: no daemon, no store.
    assert_eq!(w.run(&["status", "--json"]).code, 3, "no daemon yet");
    assert!(!w.store_dir().exists());

    let r = w.run(&["install-autostart"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let entry = w.config_home().join("autostart/rstory.desktop");
    let written = std::fs::read_to_string(&entry).expect("the desktop entry is written");
    assert_eq!(
        written,
        rstory::install::autostart::desktop_entry(std::path::Path::new(BIN)),
        "the entry matches the template"
    );
    assert!(written.starts_with("[Desktop Entry]\n"));
    assert!(written.contains("daemon"));

    let mut daemon = w.start_daemon();
    let st = w.status();
    assert_eq!(st["schema"], 1);
    assert_eq!(st["state"], "recording");
    assert_eq!(st["backend"], "x11");
    assert_eq!(st["mode"], "native");
    assert_eq!(st["entries"], 0);
    assert_eq!(st["protocol"], 1);
    assert!(w.store_dir().is_dir(), "the daemon created the store");
    assert!(w.socket().exists(), "the control socket is bound");

    // A second daemon is refused (exit 4), the first keeps serving.
    let second = w.run(&["daemon"]);
    assert_eq!(second.code, 4, "{}", second.stderr);
    assert_eq!(w.status()["state"], "recording");

    let st = daemon.stop().expect("the daemon was running");
    assert!(st.success(), "SIGTERM is a clean shutdown: {st}");
    assert!(!daemon.log_text().is_empty(), "the daemon logs to stderr");
}
