//! `rstory tray`: the tray as a client of the control socket, for a daemon that runs without its
//! own tray (`tray = false`). The bridge gives the tray the same [`Link`] it has in the daemon.

use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

use crate::domain::ids::EntryRef;
use crate::ipc::codec::{LineReader, encode};
use crate::ipc::error::{Correlation, ErrorCode, ProtoError};
use crate::ipc::protocol::{Command, Event, EventKind, VERSION};
use crate::ipc::server::{self, Call, Link};
use crate::ui::copy;

fn reference(r: &EntryRef) -> String {
    match r {
        EntryRef::Index(n) => n.to_string(),
        EntryRef::ShortId(s) => s.clone(),
    }
}

/// The wire request of `command` (the operations the tray uses, plus the rest for completeness).
fn request_of(id: u64, command: &Command) -> Value {
    let (op, args) = match command {
        Command::List { limit } => ("list", json!({ "limit": limit })),
        Command::Show(r) => ("show", json!({ "ref": reference(r) })),
        Command::Restore(r) => ("restore", json!({ "ref": reference(r) })),
        Command::Pin(r) => ("pin", json!({ "ref": reference(r) })),
        Command::Unpin(r) => ("unpin", json!({ "ref": reference(r) })),
        Command::Rm(r) => ("rm", json!({ "ref": reference(r) })),
        Command::Clear { all } => ("clear", json!({ "all": all })),
        Command::Pause { duration } => (
            "pause",
            duration.map_or_else(
                || json!({}),
                |d| json!({ "for": format!("{}s", d.as_secs().max(1)) }),
            ),
        ),
        Command::Resume => ("resume", json!({})),
        Command::Status => ("status", json!({})),
        Command::OpenPicker => ("open_picker", json!({})),
        // An empty list means both events: omit the key.
        Command::Subscribe { events } if events.is_empty() => ("subscribe", json!({})),
        Command::Subscribe { events } => (
            "subscribe",
            json!({ "events": events.iter().map(|e| e.as_str()).collect::<Vec<_>>() }),
        ),
    };
    json!({"v": VERSION, "id": id, "op": op, "args": args})
}

fn answer_of(line: &Value) -> Result<Value, ProtoError> {
    if line.get("ok") == Some(&Value::Bool(true)) {
        return Ok(line.get("result").cloned().unwrap_or(Value::Null));
    }
    let code = line
        .pointer("/error/code")
        .and_then(|c| serde_json::from_value::<ErrorCode>(c.clone()).ok())
        .unwrap_or(ErrorCode::Internal);
    let message = line
        .pointer("/error/message")
        .and_then(Value::as_str)
        .unwrap_or("request failed");
    Err(ProtoError::new(code, message))
}

fn event_of(line: &Value) -> Option<Event> {
    let name = line.get("event")?.as_str()?;
    let kind = EventKind::ALL.into_iter().find(|k| k.as_str() == name)?;
    Some(Event {
        kind,
        data: line.get("data").cloned().unwrap_or(Value::Null),
    })
}

/// Connects to the control socket and returns a [`Link`] that forwards requests to it and
/// republishes its events. The task ends when the daemon goes away.
///
/// # Errors
/// The connection failure.
pub async fn bridge(socket: &Path) -> io::Result<(Link, JoinHandle<()>)> {
    let (rd, mut wr) = UnixStream::connect(socket).await?.into_split();
    let mut reader = LineReader::new(rd);
    let (link, mut calls): (Link, mpsc::Receiver<Call>) = Link::new();
    let publish = link.clone();
    let task = tokio::spawn(async move {
        let mut next_id = 1u64;
        if wr
            .write_all(&encode(&request_of(
                next_id,
                &Command::Subscribe { events: vec![] },
            )))
            .await
            .is_err()
        {
            return;
        }
        let mut waiting: Option<(u64, Call)> = None;
        loop {
            tokio::select! {
                // One request at a time: the next call waits for the answer to the last.
                call = calls.recv(), if waiting.is_none() => {
                    let Some(call) = call else { break };
                    next_id += 1;
                    let line = encode(&request_of(next_id, &call.command));
                    if wr.write_all(&line).await.is_err() {
                        let _ = call.reply.send(Err(ProtoError::internal(Correlation::next())));
                        break;
                    }
                    waiting = Some((next_id, call));
                }
                line = reader.next_line() => {
                    let Ok(Some(raw)) = line else { break };
                    let Ok(value) = serde_json::from_slice::<Value>(&raw) else { continue };
                    if let Some(ev) = event_of(&value) {
                        publish.publish(ev);
                    } else if let Some(id) = value.get("id").and_then(Value::as_u64)
                        && waiting.as_ref().is_some_and(|(w, _)| *w == id)
                        && let Some((_, call)) = waiting.take()
                    {
                        let _ = call.reply.send(answer_of(&value));
                    }
                }
            }
        }
        if let Some((_, call)) = waiting.take() {
            let _ = call
                .reply
                .send(Err(ProtoError::internal(Correlation::next())));
        }
        // `calls` drops here: later requests fail at once instead of hanging.
    });
    Ok((link, task))
}

/// `rstory tray`: runs the tray against a running daemon until it is stopped (SIGINT/SIGTERM)
/// or the daemon goes away. Returns the exit code (3: daemon not running, as for the CLI).
#[must_use]
pub fn run_main() -> u8 {
    let Ok(rt) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        eprintln!("rstory tray: could not start the runtime");
        return 1;
    };
    rt.block_on(run())
}

async fn run() -> u8 {
    crate::daemon::log::init_logging();
    let uid = rustix::process::getuid().as_raw();
    let env = std::env::var("XDG_RUNTIME_DIR").ok();
    let Ok(dir) = server::runtime_dir(env.as_deref(), uid, Path::new("/run/user")) else {
        eprintln!("rstory tray: {}", copy::CLI_DAEMON_DOWN);
        return 3;
    };
    let socket: PathBuf = server::socket_path(&dir);
    let Ok((link, bridge_task)) = bridge(&socket).await else {
        eprintln!("rstory tray: {}", copy::CLI_DAEMON_DOWN);
        return 3;
    };
    let (stop, shutdown) = watch::channel(false);
    let (mut status, tray_task) = super::spawn(link, shutdown);
    let (Ok(mut term), Ok(mut int)) = (
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()),
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()),
    ) else {
        eprintln!("rstory tray: could not listen for signals");
        return 1;
    };
    let mut code = 0;
    tokio::select! {
        _ = term.recv() => {}
        _ = int.recv() => {}
        _ = bridge_task => { eprintln!("rstory tray: the daemon went away"); code = 3; }
        _ = tray_task_done(&mut status) => { code = 1; }
    }
    let _ = stop.send(true);
    let _ = tray_task.await;
    code
}

/// Resolves when the tray cannot run here (no bus, no watcher): a foreground tray has no use.
async fn tray_task_done(status: &mut watch::Receiver<super::TrayStatus>) {
    loop {
        if status.changed().await.is_err() {
            return;
        }
        let s = *status.borrow();
        if matches!(s, super::TrayStatus::NoBus) {
            eprintln!("rstory tray: no D-Bus session bus");
            return;
        }
    }
}
