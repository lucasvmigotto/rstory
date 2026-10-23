//! `rstory tray` as a socket client (spec 009): the bridge turns the control socket into the
//! in-process `Link` the tray runs on. No D-Bus here; synthetic text only.

mod support;

use std::time::Duration;

use rstory::domain::ids::EntryRef;
use rstory::ipc::error::ErrorCode;
use rstory::ipc::protocol::{Command, EventKind};
use rstory::tray::client::bridge;
use support::daemon::{Dirs, Opts, Rig, copy};

#[tokio::test]
async fn requests_and_errors_cross_the_socket() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let (link, _task) = bridge(&rig.socket).await.unwrap();
    let st = link.request(Command::Status).await.unwrap();
    assert_eq!(st["state"], "recording");
    assert_eq!(st["protocol"], 1);

    let mut events = link.subscribe();
    copy(&rig.fake, "alpha");
    copy(&rig.fake, "beta");
    let ev = tokio::time::timeout(Duration::from_secs(5), events.recv())
        .await
        .expect("an event in time")
        .unwrap();
    assert_eq!(ev.kind, EventKind::HistoryChanged);

    support::wait_for_async(|| rig.fake.read_limits().len() == 2).await;
    let listed = link
        .request(Command::List { limit: Some(10) })
        .await
        .unwrap();
    let entries = listed["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    let oldest = entries[1]["id"].as_str().unwrap().to_string();
    link.request(Command::Restore(EntryRef::ShortId(oldest)))
        .await
        .unwrap();
    assert_eq!(&*rig.fake.served()[0], b"alpha");

    let err = link
        .request(Command::Restore(EntryRef::ShortId("ZZZZ".into())))
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);

    let paused = link
        .request(Command::Pause {
            duration: Some(Duration::from_secs(60)),
        })
        .await
        .unwrap();
    assert_eq!(paused["state"]["state"], "paused");
    link.request(Command::Resume).await.unwrap();
    rig.stop().await;
}

#[tokio::test]
async fn a_stopped_daemon_fails_requests_instead_of_hanging() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let (link, task) = bridge(&rig.socket).await.unwrap();
    link.request(Command::Status).await.unwrap();
    rig.stop().await;
    let r = tokio::time::timeout(Duration::from_secs(5), link.request(Command::Status))
        .await
        .expect("no hang");
    assert!(r.is_err());
    tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .expect("the bridge ends")
        .unwrap();
}

#[tokio::test]
async fn no_socket_is_an_error_not_a_panic() {
    let dirs = Dirs::new();
    assert!(bridge(&dirs.socket()).await.is_err());
}
