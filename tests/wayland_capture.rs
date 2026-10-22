//! US1: capture on Wayland under headless sway (SC-001). All clipboard text here is synthetic.
//!
//! sway 1.10 (wlroots 0.18) offers `zwlr_data_control_manager_v1` only, so the `ext` path is
//! exercised when the compositor advertises `ext_data_control_manager_v1` and reported as
//! skipped otherwise; the `zwlr` path always runs.

mod support;

use std::time::{Duration, Instant};

use rstory::backend::wayland::protocols::{EXT_MANAGER, Protocol};
use rstory::backend::wayland::{ConnectError, WaylandBackend, WaylandOptions};
use rstory::backend::{Backend, BackendEvent, ReadError};
use support::sway::Sway;
use support::wl_client::{Offer, Proto, WlClient};

const MIB: usize = 1024 * 1024;
const T: Duration = Duration::from_secs(5);

fn synthetic(n: usize) -> String {
    (0..n)
        .map(|i| char::from(b'a' + u8::try_from(i * 7 % 26).unwrap()))
        .collect()
}

fn opts(s: &Sway) -> WaylandOptions {
    WaylandOptions::new().socket(s.socket())
}

async fn backend(s: &Sway) -> WaylandBackend {
    WaylandBackend::connect(opts(s)).await.expect("connect")
}

fn client(s: &Sway, proto: Proto) -> WlClient {
    WlClient::connect(&s.socket(), proto).expect("client")
}

async fn next(b: &mut WaylandBackend) -> BackendEvent {
    tokio::time::timeout(T, b.events().next())
        .await
        .expect("event in time")
        .expect("stream open")
}

fn types_of(e: &BackendEvent) -> (&[String], bool) {
    match e {
        BackendEvent::SelectionChanged { types, self_owned } => (types, *self_owned),
        other => panic!("expected SelectionChanged, got {other:?}"),
    }
}

async fn capture_hello(s: &Sway, backend_proto: Option<Protocol>, client_proto: Proto) {
    let mut o = opts(s);
    o.protocol = backend_proto;
    let mut b = WaylandBackend::connect(o).await.expect("connect");
    let c = client(s, client_proto);
    c.set_selection(Offer::utf8("hello")).unwrap();
    let ev = next(&mut b).await;
    let (types, self_owned) = types_of(&ev);
    assert!(
        types.iter().any(|t| t == "text/plain;charset=utf-8"),
        "{types:?}"
    );
    assert!(!self_owned);
    assert_eq!(b.read_text(MIB).await.unwrap(), b"hello");
}

#[tokio::test]
async fn zwlr_capture_reads_the_text() {
    let s = Sway::start();
    capture_hello(&s, Some(Protocol::Zwlr), Proto::Zwlr).await;
}

#[tokio::test]
async fn default_binding_follows_what_the_compositor_offers() {
    let s = Sway::start();
    let b = backend(&s).await;
    let want = if s.has_global(EXT_MANAGER) {
        Protocol::Ext
    } else {
        Protocol::Zwlr
    };
    assert_eq!(b.protocol(), want);
}

#[tokio::test]
async fn ext_capture_reads_the_text() {
    let s = Sway::start();
    if !s.has_global(EXT_MANAGER) {
        eprintln!("SKIPPED: this sway does not offer {EXT_MANAGER}");
        return;
    }
    capture_hello(&s, None, Proto::Ext).await;
    let b = backend(&s).await;
    assert_eq!(b.protocol(), Protocol::Ext);
}

#[tokio::test]
async fn binding_a_protocol_the_compositor_lacks_is_refused() {
    let s = Sway::start();
    if s.has_global(EXT_MANAGER) {
        eprintln!("SKIPPED: this sway offers {EXT_MANAGER}");
        return;
    }
    let r = WaylandBackend::connect(opts(&s).protocol(Protocol::Ext)).await;
    assert_eq!(r.err(), Some(ConnectError::NoDataControl));
}

#[tokio::test]
async fn a_missing_socket_is_unreachable() {
    let r =
        WaylandBackend::connect(WaylandOptions::new().socket("/tmp/rstory-no-such-socket")).await;
    assert_eq!(r.err(), Some(ConnectError::Unreachable));
}

#[tokio::test]
async fn the_password_manager_hint_is_visible_in_the_types() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s, Proto::Auto);
    c.set_selection(Offer::utf8("secret-ish").with("x-kde-passwordManagerHint", b"secret"))
        .unwrap();
    let ev = next(&mut b).await;
    let (types, _) = types_of(&ev);
    assert!(
        types.iter().any(|t| t == "x-kde-passwordManagerHint"),
        "{types:?}"
    );
    assert!(types.iter().any(|t| t == "UTF8_STRING"), "{types:?}");
}

#[tokio::test]
async fn clearing_the_selection_is_reported() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s, Proto::Auto);
    c.set_selection(Offer::utf8("soon gone")).unwrap();
    next(&mut b).await;
    c.clear_selection().unwrap();
    assert_eq!(next(&mut b).await, BackendEvent::SelectionCleared);
    assert_eq!(b.read_text(MIB).await, Err(ReadError::NoText));
}

#[tokio::test]
async fn an_owner_present_at_connect_is_reported_first() {
    let s = Sway::start();
    let c = client(&s, Proto::Auto);
    c.set_selection(Offer::utf8("already here")).unwrap();
    let mut b = backend(&s).await;
    let ev = next(&mut b).await;
    let (types, self_owned) = types_of(&ev);
    assert!(types.iter().any(|t| t == "text/plain"));
    assert!(!self_owned);
    assert_eq!(b.read_text(MIB).await.unwrap(), b"already here");
}

#[tokio::test]
async fn mime_preference_is_respected() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s, Proto::Auto);
    let order = [
        "text/plain;charset=utf-8",
        "UTF8_STRING",
        "text/plain",
        "STRING",
        "TEXT",
    ];
    for (i, want) in order.iter().enumerate() {
        // Offer the wanted type and everything less preferred, each tagged with its own name.
        let mut offer = Offer::default();
        for m in &order[i..] {
            offer = offer.with(m, format!("from-{m}").as_bytes());
        }
        c.set_selection(offer).unwrap();
        next(&mut b).await;
        assert_eq!(
            b.read_text(MIB).await.unwrap(),
            format!("from-{want}").as_bytes()
        );
    }
}

#[tokio::test]
async fn an_owner_with_no_text_type_offers_nothing_to_read() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s, Proto::Auto);
    c.set_selection(Offer::default().with("image/png", b"not text"))
        .unwrap();
    let ev = next(&mut b).await;
    let (types, _) = types_of(&ev);
    assert_eq!(types, ["image/png"]);
    assert_eq!(b.read_text(MIB).await, Err(ReadError::NoText));
}

#[tokio::test]
async fn invalid_utf8_is_decoded_lossily() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s, Proto::Auto);
    c.set_selection(Offer::default().with("text/plain", b"ab\xffcd"))
        .unwrap();
    next(&mut b).await;
    assert_eq!(b.read_text(MIB).await.unwrap(), "ab\u{fffd}cd".as_bytes());
}

#[tokio::test]
async fn limit_is_enforced() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s, Proto::Auto);
    c.set_selection(Offer::utf8(&synthetic(2048))).unwrap();
    next(&mut b).await;
    assert_eq!(b.read_text(1024).await, Err(ReadError::TooLarge));
    assert_eq!(b.read_text(2048).await.unwrap().len(), 2048);
}

#[tokio::test]
async fn one_mebibyte_is_read_whole() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s, Proto::Auto);
    let text = synthetic(MIB);
    c.set_selection(Offer::utf8(&text)).unwrap();
    next(&mut b).await;
    assert_eq!(b.read_text(MIB).await.unwrap(), text.as_bytes());
}

#[tokio::test]
async fn a_stuck_owner_times_out_without_blocking_the_backend() {
    let s = Sway::start();
    let mut o = opts(&s);
    o.read_timeout = Duration::from_millis(300);
    let mut b = WaylandBackend::connect(o).await.unwrap();
    let c = client(&s, Proto::Auto);
    c.set_selection(Offer::default().stalling("text/plain;charset=utf-8"))
        .unwrap();
    next(&mut b).await;
    let t = Instant::now();
    assert_eq!(b.read_text(MIB).await, Err(ReadError::Timeout));
    assert!(t.elapsed() < Duration::from_secs(2));
    // The loop is alive: the next selection is still captured and read.
    c.set_selection(Offer::utf8("recovered")).unwrap();
    next(&mut b).await;
    assert_eq!(b.read_text(MIB).await.unwrap(), b"recovered");
}

#[tokio::test]
async fn a_compositor_exit_disconnects() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    drop(s);
    assert_eq!(next(&mut b).await, BackendEvent::Disconnected);
    assert_eq!(b.read_text(MIB).await, Err(ReadError::Disconnected));
}

#[tokio::test]
async fn a_hundred_copies_make_a_hundred_correct_events_within_budget() {
    let s = Sway::start();
    let mut b = backend(&s).await;
    let c = client(&s, Proto::Auto);
    let mut times = Vec::new();
    for i in 0..100 {
        let text = format!("copy number {i}");
        c.set_selection(Offer::utf8(&text)).unwrap();
        let ev = next(&mut b).await;
        let t = Instant::now();
        let (types, _) = types_of(&ev);
        assert!(types.iter().any(|m| m == "text/plain;charset=utf-8"));
        assert_eq!(b.read_text(MIB).await.unwrap(), text.as_bytes());
        times.push(t.elapsed());
    }
    times.sort();
    let p95 = times[94];
    assert!(p95 < Duration::from_millis(20), "p95 {p95:?}");
    assert!(
        b.events().try_next().is_none(),
        "exactly one event per copy"
    );
}
