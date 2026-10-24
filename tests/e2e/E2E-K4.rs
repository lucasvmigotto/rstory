//! E2E-K4 No capture path (specs/004 qa.md): headless mutter with `--no-x11` offers neither
//! data-control nor XWayland. The daemon starts but is inactive with a reason, and `copy`
//! exits 7 on a store prepared earlier.

use crate::support::World;

#[test]
fn k4_no_capture_path_is_inactive_and_copy_exits_7() {
    // A store with one entry, captured in a normal session.
    let x = World::xvfb();
    let mut d = x.start_daemon();
    x.copy("k4 seed entry");
    x.wait_top("k4 seed entry");
    d.stop();

    let w = World::reuse(
        crate::support::Session::NoX11(
            crate::session::mutter::Mutter::start_no_x11().expect("mutter --no-x11"),
        ),
        x,
    );
    let _daemon = w.start_daemon();
    let st = w.status();
    assert_eq!(st["state"], "inactive", "{st}");
    assert_eq!(st["reason"], "no_data_control_no_xwayland", "{st}");
    assert_eq!(st["backend"], "none");
    assert_eq!(st["entries"], 1, "the prepared store is served");

    // 011 T012: doctor explains the same condition and exits 2.
    let (code, checks) = w.doctor();
    assert_eq!(checks["backend"]["status"], "fail", "{}", checks["backend"]);
    // The reason id is `status`'s; doctor words the same condition for people.
    let msg = checks["backend"]["message"].as_str().unwrap();
    assert!(
        msg.contains("no clipboard-monitoring protocol and no XWayland"),
        "{msg}"
    );
    assert!(
        checks["backend"]["hint"]
            .as_str()
            .unwrap()
            .contains("XWayland")
    );
    assert_eq!(code, 2, "doctor fails");

    let r = w.run(&["copy", "0"]);
    assert_eq!(r.code, 7, "inactive: {}", r.stderr);
    // Reads still work.
    assert_eq!(w.previews(), ["k4 seed entry"]);
}
