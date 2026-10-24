//! Foundational socket tests (T018): runtime directory, binding, peer check, framing.
//! The daemon behind the socket is a stub that answers a few commands.

use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

use rstory::ipc::protocol::{Command, Event};
use rstory::ipc::server::{self, Link, MAX_CONNECTIONS, RuntimeDirError};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::watch;

fn my_uid() -> u32 {
    rustix::process::getuid().as_raw()
}

struct Harness {
    _rt: TempDir,
    path: PathBuf,
    link: Link,
    stop: watch::Sender<bool>,
    task: tokio::task::JoinHandle<()>,
}

/// Binds a socket in a fresh runtime dir; a stub core answers `status` and fails the rest.
async fn start(expected_uid: u32) -> Harness {
    let rt = tempfile::tempdir().unwrap();
    let listener = server::bind(rt.path()).unwrap();
    let path = server::socket_path(rt.path());
    let (link, mut calls) = Link::new();
    tokio::spawn(async move {
        while let Some(call) = calls.recv().await {
            let answer = match call.command {
                Command::Status => Ok(json!({"stub": true})),
                _ => Err(rstory::ipc::error::ProtoError::new(
                    rstory::ipc::error::ErrorCode::NotFound,
                    "no entry",
                )),
            };
            let _ = call.reply.send(answer);
        }
    });
    let (stop, rx) = watch::channel(false);
    let task = tokio::spawn(server::serve(listener, link.clone(), expected_uid, rx));
    Harness {
        _rt: rt,
        path,
        link,
        stop,
        task,
    }
}

struct Client {
    rd: BufReader<OwnedReadHalf>,
    wr: OwnedWriteHalf,
}

impl Client {
    async fn connect(path: &Path) -> Self {
        let (rd, wr) = UnixStream::connect(path).await.unwrap().into_split();
        Self {
            rd: BufReader::new(rd),
            wr,
        }
    }

    async fn send(&mut self, line: &str) {
        self.wr.write_all(line.as_bytes()).await.unwrap();
        self.wr.write_all(b"\n").await.unwrap();
    }

    /// The next line as JSON; `None` at end of stream or on a reset.
    async fn recv(&mut self) -> Option<Value> {
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_secs(5), self.rd.read_line(&mut line))
            .await
            .expect("an answer in time")
            .unwrap_or(0);
        (n > 0).then(|| serde_json::from_str(&line).unwrap())
    }
}

#[test]
fn runtime_dir_prefers_the_environment() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(
        server::runtime_dir(Some("/somewhere/run"), 1000, root.path()),
        Ok(PathBuf::from("/somewhere/run"))
    );
}

#[test]
fn runtime_dir_falls_back_to_run_user_uid_when_it_is_ours() {
    let root = tempfile::tempdir().unwrap();
    let mine = root.path().join(my_uid().to_string());
    std::fs::create_dir(&mine).unwrap();
    for unset in [None, Some("")] {
        assert_eq!(
            server::runtime_dir(unset, my_uid(), root.path()),
            Ok(mine.clone())
        );
    }
}

#[test]
fn runtime_dir_fails_when_the_fallback_is_missing_or_not_ours() {
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join(my_uid().to_string());
    assert_eq!(
        server::runtime_dir(None, my_uid(), root.path()),
        Err(RuntimeDirError::Unavailable(missing))
    );
    // A directory owned by someone else: ask for a uid that is not ours.
    let other = my_uid() + 1;
    std::fs::create_dir(root.path().join(other.to_string())).unwrap();
    assert!(server::runtime_dir(None, other, root.path()).is_err());
}

#[tokio::test]
async fn bind_creates_a_private_directory_and_socket() {
    let rt = tempfile::tempdir().unwrap();
    let _listener = server::bind(rt.path()).unwrap();
    let dir = rt.path().join("rstory");
    let sock = dir.join("control.sock");
    assert_eq!(server::socket_path(rt.path()), sock);
    assert_eq!(
        std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let meta = std::fs::metadata(&sock).unwrap();
    assert!(meta.file_type().is_socket());
    assert_eq!(meta.permissions().mode() & 0o777, 0o600);
}

#[tokio::test]
async fn bind_tightens_a_loose_directory() {
    let rt = tempfile::tempdir().unwrap();
    let dir = rt.path().join("rstory");
    std::fs::create_dir(&dir).unwrap();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    let _listener = server::bind(rt.path()).unwrap();
    assert_eq!(
        std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
        0o700
    );
}

#[tokio::test]
async fn bind_replaces_a_stale_socket_left_by_a_crash() {
    let rt = tempfile::tempdir().unwrap();
    let first = server::bind(rt.path()).unwrap();
    drop(first); // the file stays behind, as after a SIGKILL
    assert!(server::socket_path(rt.path()).exists());
    let _second = server::bind(rt.path()).unwrap();
    assert!(
        UnixStream::connect(server::socket_path(rt.path()))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn bind_never_replaces_a_live_socket() {
    let rt = tempfile::tempdir().unwrap();
    let _live = server::bind(rt.path()).unwrap();
    let err = server::bind(rt.path()).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::AddrInUse);
}

#[tokio::test]
async fn bind_never_removes_a_file_that_is_not_a_socket() {
    let rt = tempfile::tempdir().unwrap();
    let dir = rt.path().join("rstory");
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("control.sock"), b"precious").unwrap();
    assert!(server::bind(rt.path()).is_err());
    assert_eq!(
        std::fs::read(dir.join("control.sock")).unwrap(),
        b"precious"
    );
}

#[tokio::test]
async fn requests_get_answers_with_their_ids() {
    let h = start(my_uid()).await;
    let mut c = Client::connect(&h.path).await;
    c.send(r#"{"v":1,"id":7,"op":"status","args":{}}"#).await;
    assert_eq!(
        c.recv().await.unwrap(),
        json!({"v":1,"id":7,"ok":true,"result":{"stub":true}})
    );
    c.send(r#"{"v":1,"id":8,"op":"show","args":{"ref":"0"}}"#)
        .await;
    let answer = c.recv().await.unwrap();
    assert_eq!(answer["id"], 8);
    assert_eq!(answer["error"]["code"], "not_found");
}

#[tokio::test]
async fn a_connection_from_another_uid_is_closed_without_a_byte() {
    let h = start(my_uid() + 1).await; // as if every peer were a stranger
    let mut stream = UnixStream::connect(&h.path).await.unwrap();
    let _ = stream
        .write_all(b"{\"v\":1,\"id\":1,\"op\":\"status\",\"args\":{}}\n")
        .await;
    let mut got = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(5), stream.read_to_end(&mut got))
        .await
        .expect("closed in time");
    assert!(got.is_empty(), "nothing is sent to a stranger");
}

#[tokio::test]
async fn a_malformed_line_gets_bad_request_and_the_connection_stays_open() {
    let h = start(my_uid()).await;
    let mut c = Client::connect(&h.path).await;
    c.send("{this is not json").await;
    let answer = c.recv().await.unwrap();
    assert_eq!(answer["ok"], false);
    assert_eq!(answer["error"]["code"], "bad_request");
    c.send(r#"{"v":2,"id":3,"op":"status"}"#).await;
    assert_eq!(
        c.recv().await.unwrap()["error"]["code"],
        "unsupported_version"
    );
    c.send(r#"{"v":1,"id":4,"op":"status"}"#).await;
    assert_eq!(c.recv().await.unwrap()["result"]["stub"], true);
}

#[tokio::test]
async fn an_oversize_line_gets_too_large_and_the_connection_is_closed() {
    let h = start(my_uid()).await;
    let mut c = Client::connect(&h.path).await;
    let blob = vec![b'x'; 2 * 1024 * 1024 + 4096];
    let _ = c.wr.write_all(&blob).await; // the server may close before we finish
    let first = c.recv().await.expect("an error line before the close");
    assert_eq!(first["ok"], false);
    assert_eq!(first["error"]["code"], "too_large");
    assert!(c.recv().await.is_none(), "closed after too_large");
}

#[tokio::test]
async fn subscribers_get_only_the_events_they_asked_for_after_the_answer() {
    let h = start(my_uid()).await;
    let mut c = Client::connect(&h.path).await;
    c.send(r#"{"v":1,"id":1,"op":"subscribe","args":{"events":["history_changed"]}}"#)
        .await;
    assert_eq!(
        c.recv().await.unwrap(),
        json!({"v":1,"id":1,"ok":true,"result":{}})
    );
    h.link
        .publish(Event::pause_changed(&rstory::ipc::protocol::State {
            state: rstory::ipc::protocol::StateKind::Paused,
            paused_until: None,
            reason: None,
        }));
    h.link.publish(Event::history_changed(
        3,
        Some("01JBD9A1B2C3D4E5F6G7H8J9K0".into()),
    ));
    let event = c.recv().await.unwrap();
    assert_eq!(
        event,
        json!({"v":1,"event":"history_changed","data":{"entries":3,"top":"01JBD9A1B2C3D4E5F6G7H8J9K0"}})
    );
}

#[tokio::test]
async fn a_client_that_did_not_subscribe_gets_no_events() {
    let h = start(my_uid()).await;
    let mut c = Client::connect(&h.path).await;
    c.send(r#"{"v":1,"id":1,"op":"status"}"#).await;
    c.recv().await.unwrap();
    h.link.publish(Event::history_changed(1, None));
    c.send(r#"{"v":1,"id":2,"op":"status"}"#).await;
    assert_eq!(
        c.recv().await.unwrap()["id"],
        2,
        "the next line is the answer"
    );
}

#[tokio::test]
async fn a_hundred_concurrent_connections_are_all_served() {
    let h = start(my_uid()).await;
    let mut tasks = Vec::new();
    for i in 0..100u64 {
        let path = h.path.clone();
        tasks.push(tokio::spawn(async move {
            let mut c = Client::connect(&path).await;
            c.send(&format!(r#"{{"v":1,"id":{i},"op":"status"}}"#))
                .await;
            let a = c.recv().await.unwrap();
            (a["id"].as_u64().unwrap(), a["ok"].as_bool().unwrap())
        }));
    }
    for (i, t) in tasks.into_iter().enumerate() {
        assert_eq!(t.await.unwrap(), (i as u64, true));
    }
    const { assert!(MAX_CONNECTIONS >= 100) };
}

#[tokio::test]
async fn shutdown_stops_accepting_and_closes_connections() {
    let h = start(my_uid()).await;
    let mut c = Client::connect(&h.path).await;
    c.send(r#"{"v":1,"id":1,"op":"status"}"#).await;
    c.recv().await.unwrap();
    h.stop.send(true).unwrap();
    tokio::time::timeout(Duration::from_secs(5), h.task)
        .await
        .expect("serve returns")
        .unwrap();
    assert!(c.recv().await.is_none());
}
