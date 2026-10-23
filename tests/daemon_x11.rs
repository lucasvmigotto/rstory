//! The daemon with the real X11 backend on Xvfb: capture, list, restore (session tests).

mod support;

use std::time::Duration;

use rstory::backend::x11::{X11Backend, X11Options};
use rstory::backend::{BackendKind, Mode};
use rstory::config::Caps;
use rstory::daemon::LaunchEnv;
use rstory::daemon::start::{Provided, Runtime, serve_process};
use rstory::daemon::state::BackendInfo;
use serde_json::json;
use support::daemon::{Client, Dirs};
use support::x11::{Offer, Owner, Xvfb};
use tokio::sync::watch;

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
async fn captures_from_an_x11_owner_and_restores_into_the_clipboard() {
    let x = Xvfb::start().unwrap();
    let dirs = Dirs::new();
    let rt = Runtime {
        store_dir: dirs.store.clone(),
        runtime_dir: dirs.runtime.clone(),
        caps: Caps::default(),
        launch: LaunchEnv::from_env(None),
        lock_retry: Duration::from_secs(2),
    };
    let display = x.display.clone();
    let (stop, rx) = watch::channel(false);
    let daemon = tokio::spawn(serve_process(
        rt,
        move || async move {
            let backend = X11Backend::connect(X11Options::new(&display)).await.ok();
            Provided {
                info: BackendInfo::active(BackendKind::X11, Mode::Native),
                backend,
                connector: None,
            }
        },
        rx,
    ));
    support::wait_for_async(|| dirs.socket().exists()).await;
    let mut c = Client::connect(&dirs.socket()).await;
    assert_eq!(c.status().await["backend"], "x11");

    let d = x.display.clone();
    let first =
        tokio::task::spawn_blocking(move || Owner::start(&d, Offer::utf8("first value")).unwrap())
            .await
            .unwrap();
    wait_for_top(&mut c, "first value").await;
    let d = x.display.clone();
    let second =
        tokio::task::spawn_blocking(move || Owner::start(&d, Offer::utf8("second value")).unwrap())
            .await
            .unwrap();
    wait_for_top(&mut c, "second value").await;
    assert_eq!(c.list().await.len(), 2);

    // Restore the older entry: the answer comes only once the daemon owns the selection.
    c.ok("restore", json!({"ref": "1"})).await;
    let reader = x.connect().unwrap();
    let got = tokio::task::spawn_blocking(move || {
        reader
            .convert("UTF8_STRING", Duration::from_secs(5))
            .unwrap()
    })
    .await
    .unwrap();
    assert_eq!(got, b"first value");
    wait_for_top(&mut c, "first value").await;
    assert_eq!(c.list().await.len(), 2, "our own serve created no entry");

    drop((first, second));
    stop.send(true).unwrap();
    daemon.await.unwrap().unwrap();
}
