//! US2: restore on Wayland under headless sway (SC-002). All clipboard text here is synthetic.

mod support;

use std::sync::Arc;
use std::time::{Duration, Instant};

use rstory::backend::wayland::{WaylandBackend, WaylandOptions};
use rstory::backend::{Backend, BackendEvent};
use support::sway::Sway;
use support::wl_client::{Offer, Proto, WlClient};

const MIB: usize = 1024 * 1024;
const T: Duration = Duration::from_secs(5);

fn synthetic(n: usize) -> String {
    (0..n)
        .map(|i| char::from(b'a' + u8::try_from(i * 11 % 26).unwrap()))
        .collect()
}

async fn backend(s: &Sway) -> WaylandBackend {
    WaylandBackend::connect(WaylandOptions::new().socket(s.socket()))
        .await
        .expect("connect")
}

fn client(s: &Sway) -> WlClient {
    WlClient::connect(&s.socket(), Proto::Auto).expect("client")
}

async fn next(b: &mut WaylandBackend) -> BackendEvent {
    tokio::time::timeout(T, b.events().next())
        .await
        .expect("event in time")
        .expect("open")
}

fn bytes(s: &str) -> Arc<[u8]> {
    Arc::from(s.as_bytes())
}

async fn blocking<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> R {
    tokio::task::spawn_blocking(f).await.unwrap()
}

#[tokio::test]
async fn served_text_is_read_by_another_client() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s);
    b.serve(bytes("restored")).await.unwrap();
    assert!(c.wait_offering("UTF8_STRING", T));
    let got = blocking(move || c.read("text/plain;charset=utf-8", T).unwrap()).await;
    assert_eq!(got, b"restored");
}

#[tokio::test]
async fn the_served_mime_set_is_offered() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s);
    b.serve(bytes("t")).await.unwrap();
    assert!(c.wait_offering("TEXT", T));
    let types = c.types().unwrap();
    for want in [
        "text/plain;charset=utf-8",
        "text/plain",
        "UTF8_STRING",
        "STRING",
        "TEXT",
    ] {
        assert!(
            types.iter().any(|t| t == want),
            "missing {want} in {types:?}"
        );
    }
    for mime in ["text/plain", "STRING"] {
        let c2 = client(&s);
        let got = blocking(move || c2.read(mime, T).unwrap()).await;
        assert_eq!(got, b"t");
    }
}

#[tokio::test]
async fn the_wl_paste_reference_client_reads_it() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    b.serve(bytes("via wl-paste")).await.unwrap();
    let (dir, disp) = (s.dir().to_path_buf(), s.display());
    let out = blocking(move || {
        std::process::Command::new("wl-paste")
            .args(["--no-newline", "--type", "text/plain"])
            .env("XDG_RUNTIME_DIR", dir)
            .env("WAYLAND_DISPLAY", disp)
            .output()
            .unwrap()
    })
    .await;
    assert_eq!(out.stdout, b"via wl-paste");
}

#[tokio::test]
async fn serving_is_reported_as_self_owned() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    b.serve(bytes("mine")).await.unwrap();
    match next(&mut b).await {
        BackendEvent::SelectionChanged { types, self_owned } => {
            assert!(self_owned, "{types:?}");
            assert!(types.iter().any(|t| t == "text/plain;charset=utf-8"));
        }
        other => panic!("expected SelectionChanged, got {other:?}"),
    }
    assert!(b.events().try_next().is_none(), "exactly one event");
}

#[tokio::test]
async fn a_new_owner_cancels_serving() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    b.serve(bytes("first")).await.unwrap();
    next(&mut b).await;
    let c = client(&s);
    c.set_selection(Offer::utf8("second")).unwrap();
    match next(&mut b).await {
        BackendEvent::SelectionChanged { self_owned, .. } => assert!(!self_owned),
        other => panic!("expected SelectionChanged, got {other:?}"),
    }
    assert_eq!(b.read_text(MIB).await.unwrap(), b"second");
}

#[tokio::test]
async fn a_cancelled_source_stops_answering() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s);
    b.serve(bytes("old")).await.unwrap();
    assert!(c.wait_offering("TEXT", T));
    let stale = c.generation();
    let taker = client(&s);
    taker.set_selection(Offer::utf8("new")).unwrap();
    assert!(c.wait_change(stale, T).is_some());
    let got = blocking(move || c.read("text/plain", T).unwrap()).await;
    assert_eq!(got, b"new", "the old text is no longer served");
}

#[tokio::test]
async fn one_mebibyte_round_trips_byte_for_byte() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s);
    let text = synthetic(MIB);
    let t = Instant::now();
    b.serve(bytes(&text)).await.unwrap();
    assert!(c.wait_offering("TEXT", T));
    let want = text.clone();
    let got = blocking(move || c.read("text/plain;charset=utf-8", T).unwrap()).await;
    assert_eq!(got, want.as_bytes());
    assert!(
        t.elapsed() < Duration::from_millis(200) * 5,
        "{:?}",
        t.elapsed()
    );
}

#[tokio::test]
async fn a_reader_that_leaves_early_does_not_hurt() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s);
    let text = synthetic(MIB);
    b.serve(bytes(&text)).await.unwrap();
    assert!(c.wait_offering("TEXT", T));
    // Ask, then close the read end at once: the writer sees EPIPE.
    drop(c.receive_fd("text/plain").unwrap());
    // The backend still serves the next reader in full.
    let c2 = client(&s);
    let got = blocking(move || c2.read("text/plain", T).unwrap()).await;
    assert_eq!(got.len(), MIB);
}

#[tokio::test]
async fn a_slow_reader_does_not_block_others() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s);
    b.serve(bytes(&synthetic(MIB))).await.unwrap();
    assert!(c.wait_offering("TEXT", T));
    // Never read this one; its pipe fills and its writer parks.
    let _held = c.receive_fd("text/plain").unwrap();
    let c2 = client(&s);
    let got = blocking(move || c2.read("text/plain", T).unwrap()).await;
    assert_eq!(got.len(), MIB);
}

#[tokio::test]
async fn stop_serving_gives_up_ownership() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s);
    b.serve(bytes("temp")).await.unwrap();
    next(&mut b).await;
    b.stop_serving();
    assert_eq!(next(&mut b).await, BackendEvent::SelectionCleared);
    assert!(c.wait_change(0, T).is_some());
}
