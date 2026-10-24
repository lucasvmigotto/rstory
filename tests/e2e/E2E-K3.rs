//! E2E-K3 Sensitive copy (specs/005 qa.md): a copier offers text plus the password-manager
//! hint with a canary. It is not in `ls`, the canary is in no store file and no log, and
//! `skipped_sensitive == 1`. Run on Xvfb, headless sway and the headless-mutter bridge.

use crate::support::{SECS15, World, assert_absent};

const CANARY: &str = "canary-k3-5f2c9a71";

fn journey(w: &World) {
    let mut daemon = w.start_daemon();
    w.copy_sensitive(CANARY);
    // The daemon has seen the sensitive copy before the next one replaces it.
    w.wait_status("the sensitive copy counted", SECS15, |s| {
        s["counters"]["skipped_sensitive"] == 1
    });
    // A normal copy afterwards proves the daemon keeps recording.
    w.copy("k3 plain follow-up");
    w.wait_top("k3 plain follow-up");

    assert_eq!(
        w.previews(),
        ["k3 plain follow-up"],
        "the canary is not listed"
    );
    let st = w.status();
    assert_eq!(st["counters"]["skipped_sensitive"], 1, "{st}");
    assert_eq!(st["entries"], 1);

    let ls_text = w.run(&["ls"]);
    assert_absent(&ls_text.stdout, CANARY, "ls");
    daemon.stop();
    assert_absent(&w.store_bytes(), CANARY, "the store directory");
    assert_absent(daemon.log_text().as_bytes(), CANARY, "the daemon log");
    assert_absent(ls_text.stderr.as_bytes(), CANARY, "stderr");
}

#[test]
fn k3_sensitive_copy_is_dropped_on_xvfb() {
    journey(&World::xvfb());
}

#[test]
fn k3_sensitive_copy_is_dropped_on_sway() {
    journey(&World::sway());
}

#[test]
fn k3_sensitive_copy_is_dropped_on_the_mutter_bridge() {
    journey(&World::bridge());
}
