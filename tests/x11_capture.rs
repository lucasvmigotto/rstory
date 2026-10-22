//! US1: capture on X11 under Xvfb (SC-001). All clipboard text here is synthetic.

mod support;

use std::time::{Duration, Instant};

use rstory::backend::x11::{X11Backend, X11Options};
use rstory::backend::{Backend, BackendEvent, ReadError};
use support::x11::{Offer, Owner, Xvfb};

const MIB: usize = 1024 * 1024;

fn synthetic(n: usize) -> String {
    (0..n)
        .map(|i| char::from(b'a' + u8::try_from(i * 7 % 26).unwrap()))
        .collect()
}

async fn backend(x: &Xvfb) -> X11Backend {
    X11Backend::connect(X11Options::new(&x.display))
        .await
        .expect("connect")
}

async fn next(b: &mut X11Backend) -> BackendEvent {
    tokio::time::timeout(Duration::from_secs(5), b.events().next())
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

#[tokio::test]
async fn utf8_string_is_captured() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let _o = Owner::start(&x.display, Offer::utf8("hello")).unwrap();
    let ev = next(&mut b).await;
    let (types, self_owned) = types_of(&ev);
    assert!(types.iter().any(|t| t == "UTF8_STRING"), "{types:?}");
    assert!(!self_owned);
    assert_eq!(b.read_text(MIB).await.unwrap(), b"hello");
}

#[tokio::test]
async fn string_only_owner_is_converted_from_latin1() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let _o = Owner::start(&x.display, Offer::latin1(b"caf\xe9")).unwrap();
    let ev = next(&mut b).await;
    let (types, _) = types_of(&ev);
    assert!(types.iter().any(|t| t == "STRING") && !types.iter().any(|t| t == "UTF8_STRING"));
    assert_eq!(b.read_text(MIB).await.unwrap(), "caf\u{e9}".as_bytes());
}

#[tokio::test]
async fn invalid_utf8_is_decoded_lossily() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let mut offer = Offer::default();
    offer = offer.with("UTF8_STRING", b"ab\xffcd");
    let _o = Owner::start(&x.display, offer).unwrap();
    next(&mut b).await;
    assert_eq!(b.read_text(MIB).await.unwrap(), "ab\u{fffd}cd".as_bytes());
}

#[tokio::test]
async fn target_preference_is_respected() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let offer = Offer::default()
        .with("STRING", b"from-string")
        .with("text/plain;charset=utf-8", b"from-plain")
        .with("UTF8_STRING", b"from-utf8");
    let o = Owner::start(&x.display, offer).unwrap();
    next(&mut b).await;
    assert_eq!(b.read_text(MIB).await.unwrap(), b"from-utf8");
    drop(o);
    assert_eq!(next(&mut b).await, BackendEvent::SelectionCleared);
    let offer = Offer::default()
        .with("STRING", b"from-string")
        .with("text/plain;charset=utf-8", b"from-plain");
    let _o = Owner::start(&x.display, offer).unwrap();
    next(&mut b).await;
    assert_eq!(b.read_text(MIB).await.unwrap(), b"from-plain");
}

#[tokio::test]
async fn incr_900_kib_arrives_whole() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let text = synthetic(900 * 1024);
    let _o = Owner::start(&x.display, Offer::utf8(&text)).unwrap();
    next(&mut b).await;
    assert_eq!(b.read_text(MIB).await.unwrap(), text.as_bytes());
}

#[tokio::test]
async fn owner_exit_clears_the_selection() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let o = Owner::start(&x.display, Offer::utf8("gone soon")).unwrap();
    next(&mut b).await;
    drop(o);
    assert_eq!(next(&mut b).await, BackendEvent::SelectionCleared);
    assert_eq!(b.read_text(MIB).await, Err(ReadError::NoText));
}

#[tokio::test]
async fn limit_is_enforced_for_plain_and_incr_transfers() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let o = Owner::start(&x.display, Offer::utf8(&synthetic(2048))).unwrap();
    next(&mut b).await;
    assert_eq!(b.read_text(1024).await, Err(ReadError::TooLarge));
    assert_eq!(b.read_text(2048).await.unwrap().len(), 2048);
    drop(o);
    next(&mut b).await;
    let _o = Owner::start(&x.display, Offer::utf8(&synthetic(900 * 1024))).unwrap();
    next(&mut b).await;
    assert_eq!(b.read_text(100 * 1024).await, Err(ReadError::TooLarge));
}

#[tokio::test]
async fn refused_conversion_is_reported() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let _o = Owner::start(
        &x.display,
        Offer::refusing(&["UTF8_STRING", "x-kde-passwordManagerHint"]),
    )
    .unwrap();
    let ev = next(&mut b).await;
    let (types, _) = types_of(&ev);
    assert!(
        types.iter().any(|t| t == "x-kde-passwordManagerHint"),
        "{types:?}"
    );
    assert_eq!(b.read_text(MIB).await, Err(ReadError::ConversionRefused));
}

#[tokio::test]
async fn silent_owner_times_out() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let offer = Offer {
        stall: true,
        ..Offer::utf8("never delivered")
    };
    let _o = Owner::start(&x.display, offer).unwrap();
    next(&mut b).await;
    let t = Instant::now();
    assert_eq!(b.read_text(MIB).await, Err(ReadError::Timeout));
    let took = t.elapsed();
    assert!(
        took >= Duration::from_millis(1500) && took < Duration::from_secs(4),
        "{took:?}"
    );
}

#[tokio::test]
async fn existing_selection_is_reported_at_connect() {
    let x = Xvfb::start().unwrap();
    let _o = Owner::start(&x.display, Offer::utf8("already here")).unwrap();
    let mut b = backend(&x).await;
    let ev = next(&mut b).await;
    assert!(types_of(&ev).0.iter().any(|t| t == "UTF8_STRING"));
    assert_eq!(b.read_text(MIB).await.unwrap(), b"already here");
}

#[tokio::test]
async fn a_hundred_copies_give_a_hundred_events() {
    let x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    let mut ms = Vec::new();
    for i in 0..100 {
        let text = format!("copy number {i}");
        let o = Owner::start(&x.display, Offer::utf8(&text)).unwrap();
        let t = Instant::now();
        let ev = next(&mut b).await;
        assert!(types_of(&ev).0.iter().any(|t| t == "UTF8_STRING"));
        assert_eq!(b.read_text(MIB).await.unwrap(), text.as_bytes());
        ms.push(t.elapsed());
        drop(o);
        assert_eq!(next(&mut b).await, BackendEvent::SelectionCleared);
    }
    ms.sort();
    let p95 = ms[94];
    eprintln!("event-to-text p95 = {p95:?}");
    assert!(p95 < Duration::from_millis(20), "p95 {p95:?}");
}

#[tokio::test]
async fn losing_the_server_reports_disconnected() {
    let mut x = Xvfb::start().unwrap();
    let mut b = backend(&x).await;
    x.kill();
    assert_eq!(next(&mut b).await, BackendEvent::Disconnected);
    assert_eq!(b.read_text(MIB).await, Err(ReadError::Disconnected));
}
