//! E2E-K3b Timed pause around a password-manager paste (specs/005 qa.md): `pause --for 3s`,
//! the copier sets S (not recorded), the pause ends by itself, the copier sets T (recorded);
//! `skipped_paused == 1`. Negative: with the daemon down `pause` exits 3 and the copier is
//! never run, as in `rstory pause --for 3s && copier`.

use std::time::Duration;

use crate::support::World;

#[test]
fn k3b_timed_pause_skips_one_copy_then_records_again() {
    let w = World::xvfb();
    let _daemon = w.start_daemon();

    assert_eq!(w.run(&["pause", "--for", "3s"]).code, 0);
    w.copy("k3b paused copy S");
    w.wait_status("the paused copy counted", Duration::from_secs(5), |s| {
        s["counters"]["skipped_paused"] == 1
    });
    let paused = w.status();
    assert_eq!(paused["state"], "paused", "{paused}");
    assert!(paused["paused_until"].is_string());

    // The pause ends on its own, within 5 s.
    w.wait_status("recording again", Duration::from_secs(5), |s| {
        s["state"] == "recording"
    });
    w.copy("k3b recorded copy T");
    w.wait_top("k3b recorded copy T");

    assert_eq!(
        w.previews(),
        ["k3b recorded copy T"],
        "S is absent, T present"
    );
    assert_eq!(w.status()["counters"]["skipped_paused"], 1);
}

#[test]
fn k3b_pause_without_a_daemon_exits_3_and_the_copier_does_not_run() {
    let w = World::xvfb();
    let r = w.run(&["pause", "--for", "3s"]);
    assert_eq!(r.code, 3, "{}", r.stderr);
    // `pause && copier`: the copier runs only on success, so nothing was copied.
    if r.code == 0 {
        w.copy("k3b must not run");
    }
    assert!(!w.store_dir().exists(), "no daemon, no store");
}
