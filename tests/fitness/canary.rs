//! F11 (T014): a canary copied, listed, shown and restored at `trace` level never reaches the
//! logs (Constitution III).

#[path = "../support/daemon.rs"]
mod rig;

use std::io;
use std::sync::{Arc, Mutex};

use rig::{Dirs, Opts, Rig, copy};
use serde_json::json;
use tracing_subscriber::fmt::MakeWriter;

const CANARY: &str = "CANARY-7f3a91-do-not-log";

#[derive(Clone, Default)]
struct Buf(Arc<Mutex<Vec<u8>>>);

impl io::Write for Buf {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Buf {
    type Writer = Buf;
    fn make_writer(&'a self) -> Buf {
        self.clone()
    }
}

#[tokio::test]
async fn the_canary_never_appears_in_trace_logs() {
    let buf = Buf::default();
    let sub = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(buf.clone())
        .with_ansi(false)
        .finish();
    let _guard = tracing::subscriber::set_default(sub);

    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    copy(&rig.fake, CANARY);
    let event = c.event("history_changed").await;
    assert!(
        !event.to_string().contains(CANARY),
        "events carry no content"
    );
    c.ok("list", json!({})).await;
    let shown = c.ok("show", json!({"ref": "0"})).await;
    assert_eq!(shown["content"], CANARY, "content is only in show results");
    c.ok("restore", json!({"ref": "0"})).await;
    c.ok("pin", json!({"ref": "0"})).await;
    c.err("rm", json!({"ref": "9"})).await;
    c.send_line(&format!("{{\"v\":1,\"id\":99,\"op\":\"{CANARY}\"}}"))
        .await;
    let bad = c.recv().await;
    assert!(
        !bad.to_string().contains(CANARY),
        "errors carry no content: {bad}"
    );
    rig.stop().await;

    let logs = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
    assert!(
        logs.contains("daemon.stop") || logs.contains("request"),
        "the log was captured: {logs}"
    );
    assert!(!logs.contains(CANARY), "canary leaked into the logs");
}
