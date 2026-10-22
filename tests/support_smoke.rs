//! The fixtures themselves: Xvfb starts on its own display, the test client round-trips a value
//! (including INCR), and headless mutter exposes an X display with its Xauthority.

mod support;

use std::time::Duration;
use support::mutter::Mutter;
use support::x11::{Client, Offer, Owner, Xvfb};

fn synthetic(n: usize) -> String {
    (0..n)
        .map(|i| char::from(b'a' + u8::try_from(i % 26).unwrap()))
        .collect()
}

#[test]
fn xvfb_round_trip_small_and_incr() {
    let x = Xvfb::start().unwrap();
    let text = synthetic(300_000);
    let _o = Owner::start(&x.display, Offer::utf8(&text)).unwrap();
    let c = x.connect().unwrap();
    let got = c.convert("UTF8_STRING", Duration::from_secs(5)).unwrap();
    assert_eq!(got, text.as_bytes());
    assert!(c.targets().unwrap().iter().any(|t| t == "UTF8_STRING"));
}

#[test]
fn two_servers_get_distinct_displays() {
    let a = Xvfb::start().unwrap();
    let b = Xvfb::start().unwrap();
    assert_ne!(a.display, b.display);
}

#[test]
fn mutter_exposes_xwayland_with_auth() {
    let m = Mutter::start().unwrap();
    let auth = m.xauthority().expect("xauthority file");
    let c = Client::connect(&m.display, Some(&auth)).unwrap();
    let _o = Owner::start_with_auth(&m.display, Some(&auth), Offer::utf8("bridge")).unwrap();
    assert_eq!(
        c.convert("UTF8_STRING", Duration::from_secs(5)).unwrap(),
        b"bridge"
    );
}
