//! E2E-K2 Recall via picker (specs/007 qa.md): the copier sets A then B, `rstory pick` runs on
//! a pty, the keys are typed with no mouse (A's prefix, Enter), and the clipboard then holds A.
//! Negative: with the daemon stopped the picker lists read-only, Enter exits 3 and the
//! clipboard is unchanged. Run on Xvfb and headless sway.

use expectrl::Expect;

use crate::support::{World, pty_exit_code};

const A: &str = "k2 alpha entry";
const B: &str = "k2 bravo entry";
const C: &str = "k2 unrecorded copy";

fn seed(w: &World) {
    w.copy(A);
    w.wait_top(A);
    w.copy(B);
    w.wait_top(B);
    assert_eq!(w.previews(), [B, A]);
}

fn recall(w: &World) {
    let mut daemon = w.start_daemon();
    seed(w);

    let mut p = w.pty(&["pick"]);
    // The first frame counts both entries and highlights the newest, so the picker reads keys.
    // (Only the selected row is drawn as one run of text; the others are cell-by-cell.)
    p.expect("2/2").expect("both entries are listed");
    p.expect(B).expect("the newest entry is highlighted");
    // Keys only: A's distinguishing prefix, then Enter.
    p.send("alpha").unwrap();
    p.send("\r").unwrap();
    assert_eq!(pty_exit_code(&p), 0, "the picker restored and exited 0");

    w.wait_clipboard(A);
    assert_eq!(w.previews(), [A, B], "A is now at index 0");
    daemon.stop();
}

fn read_only(w: &World) {
    let mut daemon = w.start_daemon();
    seed(w);
    daemon.stop();
    // The store keeps its entries. Something else is copied meanwhile (nobody records it).
    w.copy(C);
    let mut p = w.pty(&["pick"]);
    p.expect("2/2")
        .expect("the read-only picker lists the entries");
    p.expect(B).expect("the newest entry is highlighted");
    p.send("\r").unwrap();
    assert_eq!(pty_exit_code(&p), 3, "restore needs the daemon");
    assert_eq!(w.clipboard(), C.as_bytes(), "the clipboard is unchanged");
    // And it did not start a daemon behind our back.
    assert_eq!(w.run(&["status", "--json"]).code, 3);
}

#[test]
fn k2_recall_through_the_picker_on_xvfb() {
    recall(&World::xvfb());
}

#[test]
fn k2_recall_through_the_picker_on_sway() {
    recall(&World::sway());
}

#[test]
fn k2_without_a_daemon_the_picker_is_read_only_and_enter_exits_3() {
    read_only(&World::xvfb());
}
