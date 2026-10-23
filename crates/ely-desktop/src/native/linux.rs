//! Linux: a StatusNotifierItem with a dbusmenu, on the session bus.

use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use anyhow::Context as _;
use futures::{
    StreamExt as _,
    channel::mpsc::{self, UnboundedSender},
};
use gpui::{App, Task};
use zbus::{
    Connection,
    zvariant::{OwnedObjectPath, OwnedValue, Type, Value},
};

use super::{Look, Route, TrayItem};

const ITEM: &str = "/StatusNotifierItem";
const MENU: &str = "/MenuBar";
/// A host that never answers leaves no tray.
const REGISTER_WAIT: Duration = Duration::from_secs(3);
const PIXELS: u32 = 64;

type Pixmap = (i32, i32, Vec<u8>);

struct Item {
    id: String,
    tip: String,
    pixmap: Pixmap,
    picks: UnboundedSender<usize>,
    first: Option<usize>,
}

#[zbus::interface(name = "org.kde.StatusNotifierItem")]
impl Item {
    #[zbus(property)]
    fn category(&self) -> &str {
        "ApplicationStatus"
    }

    #[zbus(property)]
    fn id(&self) -> &str {
        &self.id
    }

    #[zbus(property)]
    fn title(&self) -> &str {
        &self.tip
    }

    #[zbus(property)]
    fn status(&self) -> &str {
        "Active"
    }

    #[zbus(property)]
    fn icon_name(&self) -> &str {
        ""
    }

    #[zbus(property)]
    fn icon_pixmap(&self) -> Vec<Pixmap> {
        vec![self.pixmap.clone()]
    }

    #[zbus(property)]
    fn tool_tip(&self) -> (String, Vec<Pixmap>, String, String) {
        (String::new(), Vec::new(), self.tip.clone(), String::new())
    }

    #[zbus(property)]
    fn item_is_menu(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn menu(&self) -> OwnedObjectPath {
        OwnedObjectPath::try_from(MENU).expect("a valid path")
    }

    /// The left click picks the first action.
    fn activate(&self, _x: i32, _y: i32) {
        if let Some(index) = self.first {
            self.picks.unbounded_send(index).ok();
        }
    }

    fn secondary_activate(&self, _x: i32, _y: i32) {}

    fn context_menu(&self, _x: i32, _y: i32) {}

    fn scroll(&self, _delta: i32, _orientation: &str) {}
}

/// One dbusmenu node: id, properties, children.
#[derive(serde::Serialize, Type, Value, zbus::zvariant::OwnedValue)]
struct Node {
    id: i32,
    properties: HashMap<String, OwnedValue>,
    children: Vec<OwnedValue>,
}

struct Menu {
    items: Vec<TrayItem>,
    picks: UnboundedSender<usize>,
}

impl Menu {
    /// Item `index` is node `index + 1`; node 0 is the root.
    fn properties(&self, id: i32) -> HashMap<String, OwnedValue> {
        let owned = |value: Value<'_>| OwnedValue::try_from(value).expect("plain values own");
        let mut properties = HashMap::new();
        match id
            .checked_sub(1)
            .and_then(|index| self.items.get(index as usize))
        {
            None => {
                properties.insert("children-display".into(), owned(Value::from("submenu")));
            }
            Some(TrayItem::Separator) => {
                properties.insert("type".into(), owned(Value::from("separator")));
            }
            Some(TrayItem::Action(label)) => {
                properties.insert("label".into(), owned(Value::from(label.to_string())));
            }
        }
        properties
    }

    fn ids(&self) -> impl Iterator<Item = i32> {
        (1..=self.items.len()).map(|id| id as i32)
    }
}

#[zbus::interface(name = "com.canonical.dbusmenu")]
impl Menu {
    #[zbus(property)]
    fn version(&self) -> u32 {
        3
    }

    #[zbus(property)]
    fn text_direction(&self) -> &str {
        "ltr"
    }

    #[zbus(property)]
    fn status(&self) -> &str {
        "normal"
    }

    #[zbus(property)]
    fn icon_theme_path(&self) -> Vec<String> {
        Vec::new()
    }

    /// The menu is flat: the root has every item.
    fn get_layout(
        &self,
        parent_id: i32,
        _depth: i32,
        _names: Vec<String>,
    ) -> zbus::fdo::Result<(u32, Node)> {
        let children = match parent_id {
            0 => self
                .ids()
                .map(|id| {
                    let child = Node {
                        id,
                        properties: self.properties(id),
                        children: Vec::new(),
                    };
                    OwnedValue::try_from(child)
                        .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))
                })
                .collect::<zbus::fdo::Result<_>>()?,
            _ => Vec::new(),
        };
        Ok((
            0,
            Node {
                id: parent_id,
                properties: self.properties(parent_id),
                children,
            },
        ))
    }

    fn get_group_properties(
        &self,
        ids: Vec<i32>,
        _names: Vec<String>,
    ) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        ids.into_iter()
            .map(|id| (id, self.properties(id)))
            .collect()
    }

    fn get_property(&self, id: i32, name: String) -> zbus::fdo::Result<OwnedValue> {
        self.properties(id)
            .remove(&name)
            .ok_or_else(|| zbus::fdo::Error::InvalidArgs(format!("item {id} has no {name}")))
    }

    fn event(&self, id: i32, event_id: String, _data: OwnedValue, _timestamp: u32) {
        let action = id
            .checked_sub(1)
            .map(|index| index as usize)
            .filter(|index| matches!(self.items.get(*index), Some(TrayItem::Action(_))));
        if let (Some(index), "clicked") = (action, event_id.as_str()) {
            self.picks.unbounded_send(index).ok();
        }
    }

    fn event_group(&self, events: Vec<(i32, String, OwnedValue, u32)>) -> Vec<i32> {
        for (id, event, data, timestamp) in events {
            self.event(id, event, data, timestamp);
        }
        Vec::new()
    }

    fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    fn about_to_show_group(&self, _ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        (Vec::new(), Vec::new())
    }
}

pub(crate) struct Tray {
    _connection: Connection,
    _picks: Task<()>,
    items: usize,
    sender: UnboundedSender<usize>,
}

impl Tray {
    pub(crate) fn new(
        look: &Look,
        items: Vec<TrayItem>,
        route: Route,
        cx: &mut App,
    ) -> anyhow::Result<Self> {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let pixmap = pixmap(look.png)?;
        let (sender, mut picks) = mpsc::unbounded();
        let name = format!(
            "org.kde.StatusNotifierItem-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let item = Item {
            id: look.name.to_string(),
            tip: look.name.to_string(),
            pixmap,
            picks: sender.clone(),
            first: items
                .iter()
                .position(|item| matches!(item, TrayItem::Action(_))),
        };
        let count = items.len();
        let menu = Menu {
            items,
            picks: sender.clone(),
        };
        // Off this thread: a stuck bus must not hang the app.
        let (done, answer) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            done.send(futures::executor::block_on(register(name, item, menu)))
                .ok();
        });
        let connection = answer
            .recv_timeout(REGISTER_WAIT)
            .map_err(|_| anyhow::anyhow!("the tray host did not answer in {REGISTER_WAIT:?}"))??;
        let picked = cx.spawn(async move |_| {
            while let Some(index) = picks.next().await {
                route.pick(index);
            }
        });
        log::info!("tray: shown with {count} items");
        Ok(Self {
            _connection: connection,
            _picks: picked,
            items: count,
            sender,
        })
    }

    pub(crate) fn pick(&self, index: usize) {
        assert!(index < self.items, "tray has no item {index}");
        self.sender.unbounded_send(index).ok();
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        log::info!("tray: removed");
    }
}

async fn register(name: String, item: Item, menu: Menu) -> anyhow::Result<Connection> {
    let connection = zbus::connection::Builder::session()?
        .name(name.as_str())?
        .serve_at(ITEM, item)?
        .serve_at(MENU, menu)?
        .build()
        .await
        .context("no session bus for the tray")?;
    let watcher = zbus::Proxy::new(
        &connection,
        "org.kde.StatusNotifierWatcher",
        "/StatusNotifierWatcher",
        "org.kde.StatusNotifierWatcher",
    )
    .await?;
    watcher
        .call_method("RegisterStatusNotifierItem", &(name.as_str(),))
        .await
        .context("no tray host on this desktop")?;
    Ok(connection)
}

/// The PNG as the 64-pixel ARGB, in network byte order.
fn pixmap(png: &[u8]) -> anyhow::Result<Pixmap> {
    let image = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .context("the tray PNG does not decode")?
        .into_rgba8();
    let image = image::imageops::resize(
        &image,
        PIXELS,
        PIXELS,
        image::imageops::FilterType::Triangle,
    );
    let bytes = image
        .pixels()
        .flat_map(|pixel| {
            let [red, green, blue, alpha] = pixel.0;
            [alpha, red, green, blue]
        })
        .collect();
    Ok((PIXELS as i32, PIXELS as i32, bytes))
}
