//! The daemon with the real Wayland backend on headless sway: capture, list, restore (session
//! tests). All clipboard text here is synthetic.

mod support;

use std::time::Duration;

use rstory::backend::wayland::{WaylandBackend, WaylandOptions};
use rstory::backend::{BackendKind, Mode};
use rstory::config::Caps;
use rstory::daemon::LaunchEnv;
use rstory::daemon::start::{Provided, Runtime, serve_process};
use rstory::daemon::state::BackendInfo;
use serde_json::json;
use support::daemon::{Client, Dirs};
use support::sway::Sway;
use support::wl_client::{Offer, Proto, WlClient};
use tokio::sync::watch;

const T: Duration = Duration::from_secs(5);

async fn wait_for_top(c: &mut Client, text: &str) {
    for _ in 0..400 {
        let entries = c.list().await;
        if entries.first().is_some_and(|e| e["preview"] == text) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("{text:?} never became the top entry");
}

#[tokio::test]
async fn captures_from_a_wayland_owner_and_restores_into_the_clipboard() {
    let s = Sway::start();
    let dirs = Dirs::new();
    let rt = Runtime {
        store_dir: dirs.store.clone(),
        runtime_dir: dirs.runtime.clone(),
        caps: Caps::default(),
        launch: LaunchEnv::from_env(None),
        lock_retry: Duration::from_secs(2),
    };
    let socket = s.socket();
    let (stop, rx) = watch::channel(false);
    let daemon = tokio::spawn(serve_process(
        rt,
        move || async move {
            let backend = WaylandBackend::connect(WaylandOptions::new().socket(socket))
                .await
                .ok();
            Provided {
                info: BackendInfo::active(BackendKind::Wayland, Mode::Native),
                backend,
                connector: None,
            }
        },
        rx,
    ));
    support::wait_for_async(|| dirs.socket().exists()).await;
    let mut c = Client::connect(&dirs.socket()).await;
    assert_eq!(c.status().await["backend"], "wayland");

    let owner = WlClient::connect(&s.socket(), Proto::Auto).unwrap();
    owner.set_selection(Offer::utf8("first value")).unwrap();
    wait_for_top(&mut c, "first value").await;
    owner.set_selection(Offer::utf8("second value")).unwrap();
    wait_for_top(&mut c, "second value").await;
    assert_eq!(c.list().await.len(), 2);

    // Restore the older entry: the answer comes only once the daemon owns the selection.
    c.ok("restore", json!({"ref": "1"})).await;
    let reader = WlClient::connect(&s.socket(), Proto::Auto).unwrap();
    let got = tokio::task::spawn_blocking(move || {
        assert!(reader.wait_offering("UTF8_STRING", T));
        reader.read("text/plain;charset=utf-8", T).unwrap()
    })
    .await
    .unwrap();
    assert_eq!(got, b"first value");
    wait_for_top(&mut c, "first value").await;
    assert_eq!(c.list().await.len(), 2, "our own serve created no entry");

    drop(owner);
    stop.send(true).unwrap();
    daemon.await.unwrap().unwrap();
}
