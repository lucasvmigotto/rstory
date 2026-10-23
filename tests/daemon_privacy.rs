//! US1 to US3 through the daemon pipeline (spec 005): hinted changes are never read, oversize
//! texts are dropped whole, paused wins over the hint. Synthetic content only.

mod support;

use rstory::backend::BackendEvent;
use rstory::config::Caps;
use serde_json::json;
use support::daemon::{Dirs, Opts, Rig, copy};

const HINT: &str = "x-kde-passwordManagerHint";
const CAP: u64 = 1024;

fn opts() -> Opts {
    Opts {
        caps: Caps {
            max_entry_bytes: CAP,
            ..Caps::default()
        },
        ..Opts::default()
    }
}

fn offer(rig: &Rig, types: &[&str]) {
    rig.fake.emit(BackendEvent::SelectionChanged {
        types: types.iter().map(|t| (*t).to_string()).collect(),
        self_owned: false,
    });
}

#[tokio::test]
async fn hinted_changes_are_counted_and_never_read() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, opts()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    // Any read would show in `read_limits`.
    for types in [
        vec!["text/plain;charset=utf-8", HINT],
        vec![HINT],
        vec!["TARGETS", "UTF8_STRING", "STRING", HINT, "TEXT"],
    ] {
        offer(&rig, &types);
    }
    copy(&rig.fake, "visible");
    c.event("history_changed").await;
    let entries = c.list().await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["preview"], "visible");
    assert_eq!(c.status().await["counters"]["skipped_sensitive"], 3);
    assert_eq!(
        rig.fake.read_limits().len(),
        1,
        "only the plain copy was read"
    );
    rig.stop().await;
}

#[tokio::test]
async fn size_boundaries_cap_minus_one_cap_and_cap_plus_one() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, opts()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    let cap = usize::try_from(CAP).unwrap();
    for n in [cap - 1, cap, cap + 1] {
        copy(&rig.fake, &"a".repeat(n));
    }
    c.event("history_changed").await;
    let sizes: Vec<_> = c.list().await.iter().map(|e| e["size"].clone()).collect();
    assert_eq!(sizes.len(), 2, "cap+1 is dropped, not truncated");
    assert!(sizes.contains(&json!(cap - 1)) && sizes.contains(&json!(cap)));
    assert_eq!(c.status().await["counters"]["skipped_oversize"], 1);
    assert_eq!(rig.fake.read_limits(), vec![cap + 1; 3]);
    rig.stop().await;
}

#[tokio::test]
async fn pause_is_checked_before_the_hint() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, opts()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    c.ok("pause", json!({})).await;
    c.event("pause_changed").await;
    offer(&rig, &[HINT]);
    let st = loop {
        let st = c.status().await;
        if st["counters"]["skipped_paused"] == 1 {
            break st;
        }
        tokio::task::yield_now().await;
    };
    assert_eq!(st["counters"]["skipped_sensitive"], 0);
    assert!(rig.fake.read_limits().is_empty());
    rig.stop().await;
}
