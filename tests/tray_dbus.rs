//! Tray over D-Bus (spec 009, T004 and T008): a private `dbus-daemon` in a temp dir, a stub
//! `StatusNotifierWatcher`, and a real daemon (FakeBackend, temp store) running the tray in a
//! child process whose session bus address is the private one.
//!
//! Safety: nothing here ever connects to the host's session bus. The parent uses the private
//! address explicitly; the child is the test binary re-executed with `DBUS_SESSION_BUS_ADDRESS`
//! set to it (ksni reads that variable, and `set_var` is unsafe). All clipboard text is synthetic.
//! `/bin/true` stands in for the picker terminal, so "Open picker" starts nothing real.

mod support;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use rstory::backend::fake::FakeBackend;
use rstory::backend::{BackendKind, Mode};
use rstory::config::Caps;
use rstory::daemon::LaunchEnv;
use rstory::daemon::start::{Extras, Provided, Runtime, serve_process_with};
use rstory::daemon::state::BackendInfo;
use rstory::ipc::server;
use rstory::tray::icons::{self, IconState};
use support::daemon::{Client, copy};
use tokio::sync::{mpsc, watch};
use zbus::zvariant::{OwnedValue, Value};

const CHILD_ENV: &str = "RSTORY_TRAY_CHILD";
const TRAY_ENV: &str = "RSTORY_TRAY_CHILD_TRAY";
const TEXTS: [&str; 3] = ["tray one", "tray two", "tray three"];

// ---------------------------------------------------------------- the child: a real daemon

/// Not a test unless the parent re-executed this binary with `CHILD_ENV`: then it is the daemon
/// plus tray under test, running until its stdin closes.
#[test]
fn child_daemon() {
    let Ok(base) = std::env::var(CHILD_ENV) else {
        return;
    };
    let base = PathBuf::from(base);
    let tray_on = std::env::var(TRAY_ENV).as_deref() != Ok("0");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        let runtime = base.join("run");
        let (backend, fake) = FakeBackend::new();
        let launch = LaunchEnv {
            picker_command: Some("/bin/true".to_string()),
            terminal: None,
            on_path: Arc::new(|_| true),
        };
        let rt = Runtime {
            store_dir: base.join("store"),
            runtime_dir: runtime.clone(),
            caps: Caps::default(),
            launch,
            lock_retry: Duration::from_secs(1),
        };
        let provide = || async move {
            Provided {
                backend: Some(backend),
                info: BackendInfo::active(BackendKind::X11, Mode::Native),
                connector: None,
            }
        };
        let (stop, shutdown) = watch::channel(false);
        let serving = tokio::spawn(serve_process_with(
            rt,
            provide,
            shutdown,
            Extras { tray: tray_on },
        ));
        let sock = server::socket_path(&runtime);
        support::wait_for_async(|| sock.exists()).await;
        let mut c = Client::connect(&sock).await;
        c.subscribe().await;
        for t in TEXTS {
            copy(&fake, t);
            c.event("history_changed").await;
        }
        drop(c);
        tokio::task::spawn_blocking(|| {
            let mut sink = Vec::new();
            let _ = std::io::Read::read_to_end(&mut std::io::stdin(), &mut sink);
        })
        .await
        .unwrap();
        let _ = stop.send(true);
        drop(serving.await);
    });
}

// ------------------------------------------------------------- the parent: bus, watcher, client

struct Guard(Child);
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct Bus {
    _dbus: Guard,
    addr: String,
    dir: tempfile::TempDir,
}

fn start_bus() -> Bus {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("bus");
    let conf = dir.path().join("bus.conf");
    std::fs::write(
        &conf,
        format!(
            r#"<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig><type>session</type><listen>unix:path={}</listen><auth>EXTERNAL</auth>
<policy context="default"><allow send_destination="*" eavesdrop="true"/><allow eavesdrop="true"/><allow own="*"/></policy>
</busconfig>"#,
            sock.display()
        ),
    )
    .unwrap();
    let child = Command::new("dbus-daemon")
        .arg(format!("--config-file={}", conf.display()))
        .arg("--nofork")
        .env_clear()
        .stdin(Stdio::null())
        .spawn()
        .expect("dbus-daemon (run the tests in the tools container)");
    let guard = Guard(child);
    support::wait_for(Duration::from_secs(10), || sock.exists().then_some(()))
        .expect("the private bus socket");
    let addr = format!("unix:path={}", sock.display());
    assert!(
        addr.contains(dir.path().to_str().unwrap()),
        "private bus only"
    );
    Bus {
        _dbus: guard,
        addr,
        dir,
    }
}

async fn connect(addr: &str) -> zbus::Connection {
    zbus::connection::Builder::address(addr)
        .unwrap()
        .build()
        .await
        .unwrap()
}

struct Watcher {
    tx: mpsc::UnboundedSender<String>,
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl Watcher {
    fn register_status_notifier_item(&self, service: String) {
        let _ = self.tx.send(service);
    }
    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }
    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Owns `org.kde.StatusNotifierWatcher` on the private bus; registrations arrive on `rx`.
async fn start_watcher(addr: &str) -> (zbus::Connection, mpsc::UnboundedReceiver<String>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let conn = zbus::connection::Builder::address(addr)
        .unwrap()
        .serve_at("/StatusNotifierWatcher", Watcher { tx })
        .unwrap()
        .name("org.kde.StatusNotifierWatcher")
        .unwrap()
        .build()
        .await
        .unwrap();
    (conn, rx)
}

struct Child_ {
    _guard: Guard,
    runtime: PathBuf,
}

fn spawn_child(bus: &Bus, base: &Path, bus_addr: &str, tray_on: bool) -> Child_ {
    std::fs::create_dir_all(base.join("run")).unwrap();
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_daemon", "--nocapture", "--test-threads=1"])
        .env_clear()
        .env(CHILD_ENV, base)
        .env(TRAY_ENV, if tray_on { "1" } else { "0" })
        .env("DBUS_SESSION_BUS_ADDRESS", bus_addr)
        .env("XDG_RUNTIME_DIR", base.join("run"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    assert!(base.starts_with(bus.dir.path()) || !bus_addr.is_empty());
    Child_ {
        _guard: Guard(child),
        runtime: base.join("run"),
    }
}

async fn client(child: &Child_) -> Client {
    let sock = server::socket_path(&child.runtime);
    support::wait_for_async(|| sock.exists()).await;
    Client::connect(&sock).await
}

async fn until<T>(what: &str, mut f: impl AsyncFnMut() -> Option<T>) -> T {
    let end = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(v) = f().await {
            return v;
        }
        assert!(std::time::Instant::now() < end, "timed out: {what}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

// ---- dbusmenu / SNI reads

/// `(id, properties, children)` as dbusmenu sends a node.
type RawNode = (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>);

#[derive(Debug, Clone)]
struct Node {
    id: i32,
    label: String,
    enabled: bool,
    children: Vec<Node>,
}

fn s_prop(props: &HashMap<String, OwnedValue>, key: &str) -> String {
    props
        .get(key)
        .and_then(|v| <&str>::try_from(&**v).ok())
        .unwrap_or("")
        .to_string()
}

fn parse_node(v: &Value<'_>) -> Node {
    let Value::Structure(s) = v else {
        panic!("layout node is not a struct: {v:?}")
    };
    let f = s.fields();
    let id = i32::try_from(&f[0]).unwrap();
    let props: HashMap<String, OwnedValue> = f[1].clone().try_into().unwrap();
    let Value::Array(kids) = &f[2] else {
        panic!("children are not an array")
    };
    let children = kids
        .iter()
        .map(|k| match k {
            Value::Value(inner) => parse_node(inner),
            other => parse_node(other),
        })
        .collect();
    Node {
        id,
        label: s_prop(&props, "label"),
        enabled: props
            .get("enabled")
            .and_then(|v| bool::try_from(&**v).ok())
            .unwrap_or(true),
        children,
    }
}

async fn layout(conn: &zbus::Connection, dest: &str) -> Node {
    let m = conn
        .call_method(
            Some(dest),
            "/MenuBar",
            Some("com.canonical.dbusmenu"),
            "GetLayout",
            &(0i32, -1i32, Vec::<String>::new()),
        )
        .await
        .unwrap();
    let body = m.body();
    let (_rev, (id, props, kids)): (u32, RawNode) = body.deserialize().unwrap();
    Node {
        id,
        label: s_prop(&props, "label"),
        enabled: true,
        children: kids.iter().map(|k| parse_node(k)).collect(),
    }
}

fn find<'a>(n: &'a Node, label: &str) -> Option<&'a Node> {
    if n.label == label {
        return Some(n);
    }
    n.children.iter().find_map(|c| find(c, label))
}

async fn click(conn: &zbus::Connection, dest: &str, id: i32) {
    conn.call_method(
        Some(dest),
        "/MenuBar",
        Some("com.canonical.dbusmenu"),
        "Event",
        &(id, "clicked", Value::from(0i32), 0u32),
    )
    .await
    .unwrap();
}

async fn sni_prop(conn: &zbus::Connection, dest: &str, name: &str) -> OwnedValue {
    let m = conn
        .call_method(
            Some(dest),
            "/StatusNotifierItem",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.kde.StatusNotifierItem", name),
        )
        .await
        .unwrap();
    let body = m.body();
    body.deserialize::<OwnedValue>().unwrap()
}

/// The 48 px pixmap bytes the item exposes.
async fn icon_48(conn: &zbus::Connection, dest: &str) -> Vec<u8> {
    let v = sni_prop(conn, dest, "IconPixmap").await;
    let pix: Vec<(i32, i32, Vec<u8>)> = v.try_into().unwrap();
    assert_eq!(pix.len(), icons::SIZES.len(), "one pixmap per size");
    pix.into_iter().find(|p| p.0 == 48).unwrap().2
}

fn expected_48(state: IconState) -> Vec<u8> {
    icons::pixmaps(state)
        .into_iter()
        .find(|p| p.size == 48)
        .unwrap()
        .argb
}

struct Rig {
    _bus: Bus,
    conn: zbus::Connection,
    _watcher: zbus::Connection,
    item: String,
    child: Child_,
}

/// Bus + stub watcher + daemon with tray; waits for the item to register.
async fn rig() -> Rig {
    let bus = start_bus();
    let (watcher, mut rx) = start_watcher(&bus.addr).await;
    let base = bus.dir.path().join("child");
    let child = spawn_child(&bus, &base, &bus.addr, true);
    let item = tokio::time::timeout(Duration::from_secs(10), rx.recv())
        .await
        .expect("the item registers")
        .unwrap();
    let conn = connect(&bus.addr).await;
    // The child seeds three entries after the tray starts: wait until the menu shows them all.
    until("the seeded entries in the menu", async || {
        find(&layout(&conn, &item).await, "tray one").map(|_| ())
    })
    .await;
    Rig {
        _bus: bus,
        conn,
        _watcher: watcher,
        item,
        child,
    }
}

// ---------------------------------------------------------------------------------- tests

#[tokio::test]
async fn the_item_registers_with_pixel_icons_and_a_recording_tooltip() {
    let r = rig().await;
    assert!(
        r.item.starts_with("org.kde.StatusNotifierItem-"),
        "{}",
        r.item
    );
    let icon = until("recording icon", async || {
        let i = icon_48(&r.conn, &r.item).await;
        (i == expected_48(IconState::Recording)).then_some(i)
    })
    .await;
    assert!(!icon.is_empty());
    let tip = sni_prop(&r.conn, &r.item, "ToolTip").await;
    let Value::Structure(s) = &*tip else {
        panic!("tooltip is not a struct")
    };
    let title = <&str>::try_from(&s.fields()[2]).unwrap();
    assert_eq!(title, "rstory — recording");
    let mut c = client(&r.child).await;
    assert_eq!(c.status().await["tray"], "active");
}

#[tokio::test]
async fn the_menu_lists_the_entries_and_activating_one_restores_it() {
    let r = rig().await;
    let root = until("three entries in the menu", async || {
        let l = layout(&r.conn, &r.item).await;
        find(&l, "tray three").is_some().then_some(l)
    })
    .await;
    // Newest first, so the third item is the oldest copy.
    let labels: Vec<&str> = root
        .children
        .iter()
        .take(3)
        .map(|n| n.label.as_str())
        .collect();
    assert_eq!(labels, ["tray three", "tray two", "tray one"]);
    let status = find(&root, "Recording").unwrap();
    assert!(!status.enabled, "the status label is insensitive");
    assert!(find(&root, "Pause recording").is_some());
    assert!(find(&root, "Pause for 1 minute").is_some());
    assert!(find(&root, "Open picker").is_some());
    assert!(find(&root, "Clear history…").is_some());

    let third = root.children[2].id;
    click(&r.conn, &r.item, third).await;
    let mut c = client(&r.child).await;
    let previews = until("restored entry on top", async || {
        let l = c.list().await;
        (l[0]["preview"] == "tray one").then_some(l)
    })
    .await;
    assert_eq!(previews.len(), 3, "restore adds nothing");
    // The menu follows the history.
    until("menu reordered", async || {
        let l = layout(&r.conn, &r.item).await;
        (l.children[0].label == "tray one").then_some(())
    })
    .await;
}

#[tokio::test]
async fn pause_switches_the_icon_label_and_state_and_resume_undoes_it() {
    let r = rig().await;
    let mut c = client(&r.child).await;
    let root = layout(&r.conn, &r.item).await;
    click(&r.conn, &r.item, find(&root, "Pause recording").unwrap().id).await;
    until("paused state", async || {
        (c.status().await["state"] == "paused").then_some(())
    })
    .await;
    let root = until("resume label and paused icon", async || {
        let l = layout(&r.conn, &r.item).await;
        let icon = icon_48(&r.conn, &r.item).await;
        (find(&l, "Resume recording").is_some() && icon == expected_48(IconState::Paused))
            .then_some(l)
    })
    .await;
    assert!(find(&root, "Paused").is_some());
    assert!(
        find(&root, "Pause for 1 minute").is_none(),
        "hidden while paused"
    );
    assert_ne!(
        expected_48(IconState::Paused),
        expected_48(IconState::Recording)
    );
    let tip = sni_prop(&r.conn, &r.item, "ToolTip").await;
    let Value::Structure(s) = &*tip else { panic!() };
    assert_eq!(<&str>::try_from(&s.fields()[2]).unwrap(), "rstory — paused");

    click(
        &r.conn,
        &r.item,
        find(&root, "Resume recording").unwrap().id,
    )
    .await;
    until("recording again", async || {
        (c.status().await["state"] == "recording").then_some(())
    })
    .await;
}

#[tokio::test]
async fn pause_for_one_minute_starts_a_timed_pause_with_the_remaining_time() {
    let r = rig().await;
    let mut c = client(&r.child).await;
    let root = layout(&r.conn, &r.item).await;
    click(
        &r.conn,
        &r.item,
        find(&root, "Pause for 1 minute").unwrap().id,
    )
    .await;
    let st = until("timed pause", async || {
        let s = c.status().await;
        (s["state"] == "paused").then_some(s)
    })
    .await;
    assert!(st["paused_until"].is_string());
    let l = until("remaining time in the status label", async || {
        let l = layout(&r.conn, &r.item).await;
        l.children
            .iter()
            .any(|n| {
                n.label.starts_with("Paused — resumes in 0:")
                    || n.label == "Paused — resumes in 1:00"
            })
            .then_some(l)
    })
    .await;
    assert!(find(&l, "Resume recording").is_some());
}

#[tokio::test]
async fn open_picker_is_offered_and_uses_the_daemon_handler() {
    let r = rig().await;
    let root = layout(&r.conn, &r.item).await;
    let id = find(&root, "Open picker").unwrap().id;
    click(&r.conn, &r.item, id).await;
    // The daemon is still healthy afterwards (the picker command is `/bin/true`).
    let mut c = client(&r.child).await;
    assert_eq!(c.status().await["picker_available"], true);
}

#[tokio::test]
async fn clear_goes_through_a_confirm_step() {
    let r = rig().await;
    let mut c = client(&r.child).await;
    let root = layout(&r.conn, &r.item).await;
    click(&r.conn, &r.item, find(&root, "Clear history…").unwrap().id).await;
    let confirm = until("confirm submenu", async || {
        let l = layout(&r.conn, &r.item).await;
        find(&l, "Delete 3 unpinned entries").map(|n| (n.id, find(&l, "Cancel").unwrap().id))
    })
    .await;
    assert_eq!(
        c.list().await.len(),
        3,
        "nothing is deleted before the confirmation"
    );

    click(&r.conn, &r.item, confirm.1).await;
    until("back to idle", async || {
        let l = layout(&r.conn, &r.item).await;
        find(&l, "Delete 3 unpinned entries")
            .is_none()
            .then_some(())
    })
    .await;
    assert_eq!(c.list().await.len(), 3, "cancel deletes nothing");

    let root = layout(&r.conn, &r.item).await;
    click(&r.conn, &r.item, find(&root, "Clear history…").unwrap().id).await;
    let confirm = until("confirm submenu again", async || {
        let l = layout(&r.conn, &r.item).await;
        find(&l, "Delete 3 unpinned entries").map(|n| n.id)
    })
    .await;
    click(&r.conn, &r.item, confirm).await;
    until("history cleared", async || {
        (c.list().await.is_empty()).then_some(())
    })
    .await;
    until("menu says empty", async || {
        let l = layout(&r.conn, &r.item).await;
        find(&l, "No entries yet").map(|_| ())
    })
    .await;
}

#[tokio::test]
async fn without_a_watcher_the_daemon_runs_with_no_host_and_the_item_registers_when_one_appears() {
    let bus = start_bus();
    let base = bus.dir.path().join("child");
    let child = spawn_child(&bus, &base, &bus.addr, true);
    let mut c = client(&child).await;
    until("no_host", async || {
        (c.status().await["tray"] == "no_host").then_some(())
    })
    .await;
    assert_eq!(
        c.status().await["state"],
        "recording",
        "the daemon is unaffected"
    );

    let started = std::time::Instant::now();
    let (_watcher, mut rx) = start_watcher(&bus.addr).await;
    let item = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("registers within 2 s of the watcher appearing")
        .unwrap();
    assert!(item.starts_with("org.kde.StatusNotifierItem-"));
    assert!(started.elapsed() < Duration::from_secs(2));
    until("active", async || {
        (c.status().await["tray"] == "active").then_some(())
    })
    .await;
}

#[tokio::test]
async fn no_session_bus_gives_no_bus() {
    let bus = start_bus();
    let base = bus.dir.path().join("child");
    let nowhere = format!("unix:path={}", bus.dir.path().join("no-such-bus").display());
    let child = spawn_child(&bus, &base, &nowhere, true);
    let mut c = client(&child).await;
    until("no_bus", async || {
        (c.status().await["tray"] == "no_bus").then_some(())
    })
    .await;
    assert_eq!(c.status().await["state"], "recording");
}

#[tokio::test]
async fn tray_false_disables_it() {
    let bus = start_bus();
    let (_watcher, mut rx) = start_watcher(&bus.addr).await;
    let base = bus.dir.path().join("child");
    let child = spawn_child(&bus, &base, &bus.addr, false);
    let mut c = client(&child).await;
    assert_eq!(c.status().await["tray"], "disabled");
    assert!(
        tokio::time::timeout(Duration::from_millis(500), rx.recv())
            .await
            .is_err(),
        "a disabled tray registers nothing"
    );
}
