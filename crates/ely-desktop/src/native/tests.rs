use std::{cell::RefCell, rc::Rc};

use gpui::TestAppContext;

use super::{TrayIcon, TrayItem, TrayLook};

const PNG: &[u8] = include_bytes!("../../../ely-gallery/assets/stickers/olive.png");

fn look(png: &'static [u8]) -> TrayLook {
    TrayLook {
        symbol: "envelope",
        png,
        name: "Ely Test".into(),
    }
}

fn items() -> Vec<TrayItem> {
    vec![
        TrayItem::Action("Open".into()),
        TrayItem::Separator,
        TrayItem::Action("Quit".into()),
    ]
}

/// A tray whose picks land in the returned list.
fn tray(cx: &mut TestAppContext) -> (TrayIcon, Rc<RefCell<Vec<usize>>>) {
    let picked = Rc::new(RefCell::new(Vec::new()));
    let seen = picked.clone();
    let tray = cx
        .update(|cx| {
            TrayIcon::new(
                look(PNG),
                items(),
                move |index, _| seen.borrow_mut().push(index),
                cx,
            )
        })
        .expect("the tray shows");
    (tray, picked)
}

/// Runs until `picked` holds `want`, or fails after two seconds.
fn wait_for(cx: &mut TestAppContext, picked: &RefCell<Vec<usize>>, want: &[usize]) {
    cx.executor().allow_parking();
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(2);
    while picked.borrow().as_slice() != want && web_time::Instant::now() < deadline {
        cx.run_until_parked();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(picked.borrow().as_slice(), want);
}

#[cfg(target_os = "windows")]
#[gpui::test]
fn the_tray_picks_by_click_and_by_call(cx: &mut TestAppContext) {
    use crate::platform::windows::{click, window_lives};

    let (tray, picked) = tray(cx);
    tray.pick(2);
    wait_for(cx, &picked, &[2]);
    click(&tray.inner);
    wait_for(cx, &picked, &[2, 0]);
    let window = tray.inner.window();
    drop(tray);
    assert!(!window_lives(window), "the window goes with the tray");
    let refused = cx.update(|cx| TrayIcon::new(look(b"not a png"), items(), |_, _| {}, cx));
    assert!(refused.is_err(), "a bad PNG shows no tray");
}

#[cfg(target_os = "linux")]
#[gpui::test]
fn the_tray_serves_its_menu_on_the_bus(cx: &mut TestAppContext) {
    use std::io::BufRead as _;

    let bus = std::process::Command::new("dbus-daemon")
        .args(["--session", "--nofork", "--nopidfile", "--print-address=1"])
        .stdout(std::process::Stdio::piped())
        .spawn();
    let Ok(bus) = bus else {
        eprintln!("SKIP: no dbus-daemon");
        return;
    };
    // A failed check must not leave the bus running.
    struct Bus(std::process::Child);
    impl Drop for Bus {
        fn drop(&mut self) {
            self.0.kill().ok();
        }
    }
    let mut bus = Bus(bus);
    let mut address = String::new();
    std::io::BufReader::new(bus.0.stdout.take().expect("its address"))
        .read_line(&mut address)
        .expect("reads the address");
    crate::platform::use_session_bus(address.trim());
    let refused = cx.update(|cx| TrayIcon::new(look(PNG), items(), |_, _| {}, cx));
    let error = format!("{:#}", refused.err().expect("no host, no tray"));
    assert!(error.contains("no tray host"), "{error}");
    let watcher = futures::executor::block_on(host::serve());
    let (tray, picked) = tray(cx);
    let (labels, pixmap) = futures::executor::block_on(host::inspect(&watcher));
    assert_eq!(labels, ["label=Open", "type=separator", "label=Quit"]);
    assert_eq!((pixmap.0, pixmap.1, pixmap.2.len()), (64, 64, 64 * 64 * 4));
    futures::executor::block_on(host::click(&watcher, "Event", 3));
    wait_for(cx, &picked, &[2]);
    futures::executor::block_on(host::click(&watcher, "Activate", 0));
    wait_for(cx, &picked, &[2, 0]);
    tray.pick(0);
    wait_for(cx, &picked, &[2, 0, 0]);
    drop(tray);
}

/// A stand-in tray host: org.kde.StatusNotifierWatcher.
#[cfg(target_os = "linux")]
mod host {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use zbus::{
        Connection, Proxy,
        zvariant::{OwnedValue, Type, Value},
    };

    /// A dbusmenu node as the host reads it.
    #[derive(serde::Deserialize, Type, Value, zbus::zvariant::OwnedValue)]
    struct Node {
        _id: i32,
        properties: HashMap<String, OwnedValue>,
        children: Vec<OwnedValue>,
    }

    pub(crate) struct Host {
        connection: Connection,
        items: Arc<Mutex<Vec<String>>>,
    }

    struct Watcher(Arc<Mutex<Vec<String>>>);

    #[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
    impl Watcher {
        fn register_status_notifier_item(&self, service: String) {
            self.0.lock().expect("the list").push(service);
        }
    }

    pub(crate) async fn serve() -> Host {
        let items = Arc::new(Mutex::new(Vec::new()));
        let connection = zbus::connection::Builder::session()
            .expect("a bus")
            .name("org.kde.StatusNotifierWatcher")
            .expect("the name")
            .serve_at("/StatusNotifierWatcher", Watcher(items.clone()))
            .expect("served")
            .build()
            .await
            .expect("connected");
        Host { connection, items }
    }

    fn item(host: &Host) -> String {
        host.items
            .lock()
            .expect("the list")
            .last()
            .cloned()
            .expect("an item registered")
    }

    /// Each item's one property, and the icon.
    pub(crate) async fn inspect(host: &Host) -> (Vec<String>, (i32, i32, Vec<u8>)) {
        let name = item(host);
        let menu = Proxy::new(
            &host.connection,
            name.as_str(),
            "/MenuBar",
            "com.canonical.dbusmenu",
        )
        .await
        .expect("the menu");
        let reply = menu
            .call_method("GetLayout", &(0i32, -1i32, Vec::<String>::new()))
            .await
            .expect("a layout");
        let (_, root): (u32, Node) = reply.body().deserialize().expect("the layout");
        let labels = root
            .children
            .into_iter()
            .map(|child| {
                let child = Node::try_from(child).expect("a node");
                let (key, value) = child.properties.into_iter().next().expect("one property");
                let value: String = value.try_into().expect("text");
                format!("{key}={value}")
            })
            .collect();
        let item = Proxy::new(
            &host.connection,
            name.as_str(),
            "/StatusNotifierItem",
            "org.kde.StatusNotifierItem",
        )
        .await
        .expect("the item");
        let mut pixmaps: Vec<(i32, i32, Vec<u8>)> =
            item.get_property("IconPixmap").await.expect("the icon");
        (labels, pixmaps.remove(0))
    }

    /// `Event` clicks menu node `id`; `Activate` clicks the icon.
    pub(crate) async fn click(host: &Host, method: &str, id: i32) {
        let name = item(host);
        match method {
            "Event" => {
                let menu = Proxy::new(
                    &host.connection,
                    name.as_str(),
                    "/MenuBar",
                    "com.canonical.dbusmenu",
                )
                .await
                .expect("the menu");
                let data = zbus::zvariant::Value::from(0i32);
                menu.call_method("Event", &(id, "clicked", data, 0u32))
                    .await
                    .expect("the click");
            }
            _ => {
                let item = Proxy::new(
                    &host.connection,
                    name.as_str(),
                    "/StatusNotifierItem",
                    "org.kde.StatusNotifierItem",
                )
                .await
                .expect("the item");
                item.call_method("Activate", &(0i32, 0i32))
                    .await
                    .expect("the click");
            }
        }
    }
}
