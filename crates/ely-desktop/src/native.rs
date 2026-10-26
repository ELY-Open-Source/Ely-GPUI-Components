use std::rc::Rc;

use gpui::{App, AsyncApp, SharedString};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(all(
    test,
    feature = "test-support",
    any(target_os = "windows", target_os = "linux")
))]
mod tests;

#[cfg(target_os = "macos")]
type Inner = crate::platform::mac::StatusItem;
#[cfg(target_os = "windows")]
type Inner = crate::platform::windows::Tray;
#[cfg(target_os = "linux")]
type Inner = linux::Tray;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
type Inner = none::Tray;

/// Elsewhere (the web): no tray.
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod none {
    pub(crate) struct Tray;

    impl Tray {
        pub(crate) fn new(
            _: &super::Look,
            _: Vec<super::TrayItem>,
            _: super::Route,
            _: &mut gpui::App,
        ) -> anyhow::Result<Self> {
            anyhow::bail!("tray icons need macOS, Windows or Linux")
        }

        pub(crate) fn pick(&self, _: usize) {}
    }
}

/// One entry in a tray menu.
#[derive(Clone, Debug)]
pub enum TrayItem {
    /// A row; a pick hands `on_pick` its place among all the items.
    Action(SharedString),
    /// A line between rows.
    Separator,
}

type OnPick = Rc<dyn Fn(usize, &mut App)>;

/// How the icon looks: macOS draws the symbol, others the PNG.
pub struct TrayLook {
    /// An SF Symbol, for the macOS menu bar.
    pub symbol: &'static str,
    /// A PNG, for the Windows tray and Linux's StatusNotifierItem.
    pub png: &'static [u8],
    /// The app's name, shown on hover.
    pub name: SharedString,
}

use TrayLook as Look;

/// Hands a pick to the app.
struct Route {
    app: AsyncApp,
    on_pick: OnPick,
}

impl Route {
    #[cfg_attr(
        not(any(target_os = "linux", target_os = "macos", target_os = "windows")),
        expect(dead_code, reason = "no tray takes it")
    )]
    fn pick(&self, index: usize) {
        log::info!("tray: picked item {index}");
        let on_pick = self.on_pick.clone();
        // The app outlives its run loop, the only place this task is polled.
        self.app
            .spawn(async move |cx| cx.update(|cx| on_pick(index, cx)))
            .detach();
    }
}

/// An icon in the menu bar or tray with a menu. Dropping it removes it.
///
/// Platforms: macOS, Windows, Linux.
pub struct TrayIcon {
    inner: Inner,
}

impl TrayIcon {
    /// `on_pick` gets the chosen item's index; a left click picks the first.
    pub fn new(
        look: TrayLook,
        items: Vec<TrayItem>,
        on_pick: impl Fn(usize, &mut App) + 'static,
        cx: &mut App,
    ) -> anyhow::Result<Self> {
        let route = Route {
            app: cx.to_async(),
            on_pick: Rc::new(on_pick),
        };
        Ok(Self {
            inner: open(&look, items, route, cx)?,
        })
    }

    /// Chooses item `index` as a click would.
    pub fn pick(&self, index: usize) {
        self.inner.pick(index);
    }
}

/// The OS's tray: macOS and Windows take labels and a pick callback.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn open(look: &Look, items: Vec<TrayItem>, route: Route, _: &mut App) -> anyhow::Result<Inner> {
    let entries: Vec<Option<&str>> = (items.iter())
        .map(|item| match item {
            TrayItem::Action(label) => Some(label.as_ref()),
            TrayItem::Separator => None,
        })
        .collect();
    let on_pick = Box::new(move |index| route.pick(index));
    #[cfg(target_os = "macos")]
    return Inner::new(look.symbol, &look.name, &entries, on_pick);
    #[cfg(target_os = "windows")]
    return Inner::new(look.png, &look.name, &entries, on_pick);
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn open(look: &Look, items: Vec<TrayItem>, route: Route, cx: &mut App) -> anyhow::Result<Inner> {
    Inner::new(look, items, route, cx)
}

/// Sets or clears the badge on the app's Dock icon.
///
/// Platforms: macOS.
pub fn set_dock_badge(label: Option<&str>) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        crate::platform::mac::set_dock_badge(label);
        log::info!("dock badge: {label:?}");
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = label;
        anyhow::bail!("dock badges need macOS in this build")
    }
}

/// The badge the Dock shows now.
///
/// Platforms: macOS.
pub fn dock_badge() -> anyhow::Result<Option<String>> {
    #[cfg(target_os = "macos")]
    return Ok(crate::platform::mac::dock_badge());
    #[cfg(not(target_os = "macos"))]
    anyhow::bail!("dock badges need macOS in this build")
}
