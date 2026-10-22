//! US3: backend selection and capture/restore through headless mutter's XWayland.
//!
//! Headless mutter has no data-control and starts XWayland on demand, like GNOME. It has no
//! keyboard seat, so no Wayland client can own a selection here: cross-protocol copies are
//! checked by hand on GNOME 48-51 (docs/release.md).

mod support;

use std::sync::Arc;
use std::time::Duration;

use rstory::backend::x11::{X11Backend, X11Options};
use rstory::backend::{Backend, BackendEvent, BackendKind, Mode};
use rstory::session::{RealProbe, Reason, SessionEnv, select};
use support::mutter::Mutter;
use support::x11::{Client, Offer, Owner};

const T: Duration = Duration::from_secs(10);

fn env_of(m: &Mutter) -> SessionEnv {
    SessionEnv {
        session_type: Some("wayland".into()),
        wayland_display: Some(m.wayland_socket().to_string_lossy().into_owned()),
        display: Some(m.display.clone()),
        xdg_runtime_dir: Some(m.runtime_dir().to_string_lossy().into_owned()),
    }
}

#[tokio::test]
async fn mutter_selects_x11_through_the_bridge() {
    let m = Mutter::start().unwrap();
    let env = env_of(&m);
    let auth = m.xauthority().unwrap();
    let sel = tokio::task::spawn_blocking(move || {
        let mut probe = RealProbe::new(&env, Some(auth));
        select(&env, &mut probe, None)
    })
    .await
    .unwrap();
    assert_eq!(sel.backend, BackendKind::X11);
    assert_eq!(sel.mode, Mode::XwaylandBridge);
    assert_eq!(sel.reason, Reason::NoDataControl);
}

#[tokio::test]
async fn capture_and_restore_between_x11_clients_on_xwayland() {
    let m = Mutter::start().unwrap();
    let auth = m.xauthority().unwrap();
    let opts = X11Options::new(&m.display).xauthority(&auth).bridge();
    let mut b = X11Backend::connect(opts).await.expect("bridge connect");

    let owner =
        Owner::start_with_auth(&m.display, Some(&auth), Offer::utf8("via xwayland")).unwrap();
    let ev = tokio::time::timeout(T, b.events().next())
        .await
        .unwrap()
        .unwrap();
    match ev {
        BackendEvent::SelectionChanged { types, self_owned } => {
            assert!(!self_owned);
            assert!(types.iter().any(|t| t == "UTF8_STRING"), "{types:?}");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(b.read_text(1 << 20).await.unwrap(), b"via xwayland");
    drop(owner);

    b.serve(Arc::from(&b"restored on bridge"[..]))
        .await
        .unwrap();
    let (d, a) = (m.display.clone(), auth.clone());
    let got = tokio::task::spawn_blocking(move || {
        Client::connect(&d, Some(&a))
            .unwrap()
            .convert("UTF8_STRING", T)
            .unwrap()
    })
    .await
    .unwrap();
    assert_eq!(got, b"restored on bridge");
}

#[tokio::test]
async fn bridge_mode_waits_for_a_display_that_is_not_there_yet() {
    // Nothing listens on this display: bridge mode retries for connect_timeout, then gives up.
    let mut opts = X11Options::new(":59").bridge();
    opts.connect_timeout = Duration::from_millis(600);
    let t = std::time::Instant::now();
    assert!(X11Backend::connect(opts).await.is_err());
    assert!(t.elapsed() >= Duration::from_millis(500));
    let t = std::time::Instant::now();
    assert!(X11Backend::connect(X11Options::new(":59")).await.is_err());
    assert!(
        t.elapsed() < Duration::from_millis(400),
        "native mode does not retry"
    );
}
