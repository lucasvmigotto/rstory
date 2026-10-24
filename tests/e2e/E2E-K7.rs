//! E2E-K7 Tray (specs/009 qa.md): the real daemon on Xvfb with a private `dbus-daemon` and a
//! stub StatusNotifierWatcher. The item registers, the menu layout is read over D-Bus, entry 2
//! is activated (clipboard == entry 2), then Pause (status paused, label `Resume recording`,
//! the paused icon). Nothing here touches the host session bus.

use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use rstory::tray::icons::{self, IconState};
use tokio::sync::mpsc;
use zbus::zvariant::{OwnedValue, Value};

use crate::session::wait_for;
use crate::support::{SECS15, World};

const TEXTS: [&str; 3] = ["k7 entry one", "k7 entry two", "k7 entry three"];

struct Guard(Child);
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start_bus(dir: &std::path::Path) -> (Guard, String) {
    let sock = dir.join("bus");
    let conf = dir.join("bus.conf");
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
    wait_for(SECS15, || sock.exists().then_some(())).expect("the private bus socket");
    (guard, format!("unix:path={}", sock.display()))
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

type RawNode = (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>);

#[derive(Debug, Clone)]
struct Node {
    id: i32,
    label: String,
    children: Vec<Node>,
}

fn label_of(props: &HashMap<String, OwnedValue>) -> String {
    props
        .get("label")
        .and_then(|v| <&str>::try_from(&**v).ok())
        .unwrap_or("")
        .to_string()
}

fn parse_node(v: &Value<'_>) -> Node {
    let Value::Structure(s) = v else {
        panic!("layout node is not a struct")
    };
    let f = s.fields();
    let props: HashMap<String, OwnedValue> = f[1].clone().try_into().unwrap();
    let Value::Array(kids) = &f[2] else {
        panic!("children are not an array")
    };
    Node {
        id: i32::try_from(&f[0]).unwrap(),
        label: label_of(&props),
        children: kids
            .iter()
            .map(|k| match k {
                Value::Value(inner) => parse_node(inner),
                other => parse_node(other),
            })
            .collect(),
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
        label: label_of(&props),
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

async fn icon_48(conn: &zbus::Connection, dest: &str) -> Vec<u8> {
    let m = conn
        .call_method(
            Some(dest),
            "/StatusNotifierItem",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.kde.StatusNotifierItem", "IconPixmap"),
        )
        .await
        .unwrap();
    let body = m.body();
    let v = body.deserialize::<OwnedValue>().unwrap();
    let pix: Vec<(i32, i32, Vec<u8>)> = v.try_into().unwrap();
    pix.into_iter().find(|p| p.0 == 48).unwrap().2
}

fn expected_48(state: IconState) -> Vec<u8> {
    icons::pixmaps(state)
        .into_iter()
        .find(|p| p.size == 48)
        .unwrap()
        .argb
}

async fn until<T>(what: &str, mut f: impl AsyncFnMut() -> Option<T>) -> T {
    let end = std::time::Instant::now() + SECS15;
    loop {
        if let Some(v) = f().await {
            return v;
        }
        assert!(std::time::Instant::now() < end, "timed out: {what}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn k7_tray_registers_restores_an_entry_and_pauses() {
    let w = World::xvfb();
    let bus_dir = tempfile::tempdir().unwrap();
    let (_bus, addr) = start_bus(bus_dir.path());
    assert!(
        addr.contains(bus_dir.path().to_str().unwrap()),
        "private bus only"
    );

    let (tx, mut rx) = mpsc::unbounded_channel();
    let _watcher = zbus::connection::Builder::address(addr.as_str())
        .unwrap()
        .serve_at("/StatusNotifierWatcher", Watcher { tx })
        .unwrap()
        .name("org.kde.StatusNotifierWatcher")
        .unwrap()
        .build()
        .await
        .unwrap();

    w.set_env("DBUS_SESSION_BUS_ADDRESS", &addr);
    let _daemon = w.start_daemon();
    let item = tokio::time::timeout(SECS15, rx.recv())
        .await
        .expect("the item registers")
        .unwrap();
    assert!(item.starts_with("org.kde.StatusNotifierItem-"), "{item}");
    assert_eq!(w.status()["tray"], "active");

    for t in TEXTS {
        w.copy(t);
        w.wait_top(t);
    }

    let conn = zbus::connection::Builder::address(addr.as_str())
        .unwrap()
        .build()
        .await
        .unwrap();
    let root = until("the three entries in the menu", async || {
        let l = layout(&conn, &item).await;
        find(&l, TEXTS[0]).is_some().then_some(l)
    })
    .await;
    // Newest first: [three, two, one]. Entry 2 is "two".
    assert_eq!(root.children[1].label, TEXTS[1]);
    assert_eq!(
        icon_48(&conn, &item).await,
        expected_48(IconState::Recording)
    );

    click(&conn, &item, root.children[1].id).await;
    until("entry 2 restored", async || {
        (w.previews().first().map(String::as_str) == Some(TEXTS[1])).then_some(())
    })
    .await;
    w.wait_clipboard(TEXTS[1]);

    let pause = find(&root, "Pause recording").expect("a Pause item").id;
    click(&conn, &item, pause).await;
    until("status paused", async || {
        (w.status()["state"] == "paused").then_some(())
    })
    .await;
    until("the Resume label and the paused icon", async || {
        let l = layout(&conn, &item).await;
        let icon = icon_48(&conn, &item).await;
        (find(&l, "Resume recording").is_some() && icon == expected_48(IconState::Paused))
            .then_some(())
    })
    .await;
}
