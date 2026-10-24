//! US2: restore on X11 under Xvfb (SC-002). All clipboard text here is synthetic.

mod support;

use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use rstory::backend::x11::{X11Backend, X11Options};
use rstory::backend::{Backend, BackendEvent};
use support::x11::{Client, Offer, Owner, Xvfb};

const MIB: usize = 1024 * 1024;
const T: Duration = Duration::from_secs(5);

fn synthetic(n: usize) -> String {
    (0..n)
        .map(|i| char::from(b'a' + u8::try_from(i * 11 % 26).unwrap()))
        .collect()
}

async fn backend(x: &Xvfb) -> X11Backend {
    X11Backend::connect(X11Options::new(&x.display))
        .await
        .expect("connect")
}

async fn next(b: &mut X11Backend) -> BackendEvent {
    tokio::time::timeout(T, b.events().next())
        .await
        .expect("event in time")
        .expect("open")
}

fn bytes(s: &str) -> Arc<[u8]> {
    Arc::from(s.as_bytes())
}

/// Run a blocking client closure off the async executor so the backend task keeps running.
async fn blocking<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> R {
    tokio::task::spawn_blocking(f).await.unwrap()
}

#[tokio::test]
async fn served_text_is_read_by_another_client() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    b.serve(bytes("restored")).await.unwrap();
    let d = x.display.clone();
    let got = blocking(move || {
        let c = Client::connect(&d, None).unwrap();
        assert_ne!(
            c.clipboard_owner().unwrap(),
            0,
            "ownership confirmed before serve returns"
        );
        c.convert("UTF8_STRING", T).unwrap()
    })
    .await;
    assert_eq!(got, b"restored");
}

#[tokio::test]
async fn targets_list_the_served_types() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    b.serve(bytes("t")).await.unwrap();
    let d = x.display.clone();
    let targets = blocking(move || Client::connect(&d, None).unwrap().targets().unwrap()).await;
    for want in [
        "TARGETS",
        "TIMESTAMP",
        "UTF8_STRING",
        "STRING",
        "TEXT",
        "text/plain;charset=utf-8",
    ] {
        assert!(
            targets.iter().any(|t| t == want),
            "missing {want} in {targets:?}"
        );
    }
    assert!(!targets.iter().any(|t| t == "MULTIPLE"));
}

#[tokio::test]
async fn every_served_target_converts_and_others_are_refused() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    b.serve(bytes("caf\u{e9} \u{20ac}")).await.unwrap();
    let d = x.display.clone();
    blocking(move || {
        let c = Client::connect(&d, None).unwrap();
        assert_eq!(
            c.convert("UTF8_STRING", T).unwrap(),
            "caf\u{e9} \u{20ac}".as_bytes()
        );
        assert_eq!(
            c.convert("text/plain;charset=utf-8", T).unwrap(),
            "caf\u{e9} \u{20ac}".as_bytes()
        );
        assert_eq!(
            c.convert("TEXT", T).unwrap(),
            "caf\u{e9} \u{20ac}".as_bytes()
        );
        assert_eq!(c.convert("STRING", T).unwrap(), b"caf\xe9 ?");
        assert_eq!(c.convert("TIMESTAMP", T).unwrap().len(), 4);
        assert!(c.convert("image/png", T).is_err());
        assert!(c.convert("MULTIPLE", T).is_err());
    })
    .await;
}

#[tokio::test]
async fn one_mib_round_trips_byte_for_byte() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let text = synthetic(MIB);
    b.serve(bytes(&text)).await.unwrap();
    let d = x.display.clone();
    let got = blocking(move || {
        Client::connect(&d, None)
            .unwrap()
            .convert("UTF8_STRING", T)
            .unwrap()
    })
    .await;
    assert!(got == text.as_bytes(), "1 MiB differs");
}

#[tokio::test]
async fn xclip_reads_a_large_value_too() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let text = synthetic(600 * 1024);
    b.serve(bytes(&text)).await.unwrap();
    let d = x.display.clone();
    let out = blocking(move || {
        Command::new("xclip")
            .args(["-o", "-selection", "clipboard"])
            .env("DISPLAY", d)
            .output()
            .unwrap()
    })
    .await;
    assert!(out.status.success());
    assert!(out.stdout == text.as_bytes(), "xclip got different bytes");
}

#[tokio::test]
async fn concurrent_requestors_each_get_the_whole_value() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let text = synthetic(MIB);
    b.serve(bytes(&text)).await.unwrap();
    let jobs: Vec<_> = (0..4)
        .map(|_| {
            let d = x.display.clone();
            tokio::task::spawn_blocking(move || {
                Client::connect(&d, None)
                    .unwrap()
                    .convert("UTF8_STRING", Duration::from_secs(10))
                    .unwrap()
            })
        })
        .collect();
    for job in jobs {
        assert!(job.await.unwrap() == text.as_bytes());
    }
}

#[tokio::test]
async fn our_own_ownership_is_flagged_self_owned() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    b.serve(bytes("mine")).await.unwrap();
    match next(&mut b).await {
        BackendEvent::SelectionChanged { types, self_owned } => {
            assert!(self_owned);
            assert!(types.iter().any(|t| t == "UTF8_STRING"));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(b.read_text(MIB).await.unwrap(), b"mine");
}

#[tokio::test]
async fn another_owner_ends_serving_and_is_captured() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    b.serve(bytes("first")).await.unwrap();
    assert!(matches!(
        next(&mut b).await,
        BackendEvent::SelectionChanged {
            self_owned: true,
            ..
        }
    ));
    let o = Owner::start(&x.display, Offer::utf8("second")).unwrap();
    match next(&mut b).await {
        BackendEvent::SelectionChanged { self_owned, .. } => assert!(!self_owned),
        other => panic!("{other:?}"),
    }
    assert_eq!(b.read_text(MIB).await.unwrap(), b"second");
    drop(o);
}

#[tokio::test]
async fn serving_again_replaces_the_value() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    b.serve(bytes("one")).await.unwrap();
    b.serve(bytes("two")).await.unwrap();
    let d = x.display.clone();
    let got = blocking(move || {
        Client::connect(&d, None)
            .unwrap()
            .convert("UTF8_STRING", T)
            .unwrap()
    })
    .await;
    assert_eq!(got, b"two");
}

#[tokio::test]
async fn stop_serving_releases_the_selection() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    b.serve(bytes("bye")).await.unwrap();
    assert!(matches!(
        next(&mut b).await,
        BackendEvent::SelectionChanged {
            self_owned: true,
            ..
        }
    ));
    b.stop_serving();
    assert_eq!(next(&mut b).await, BackendEvent::SelectionCleared);
    let d = x.display.clone();
    assert_eq!(
        blocking(move || Client::connect(&d, None)
            .unwrap()
            .clipboard_owner()
            .unwrap())
        .await,
        0
    );
}
