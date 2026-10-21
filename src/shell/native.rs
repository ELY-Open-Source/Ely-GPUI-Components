use std::rc::Rc;

use gpui::{App, AsyncApp, SharedString};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(all(test, feature = "test-support"))]
mod tests;
#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use mac as platform;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
use none as platform;
#[cfg(target_os = "windows")]
use win as platform;

/// Elsewhere (the web): no tray.
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod none {
    pub struct Tray;

    impl Tray {
        pub fn new(
            _: &super::Look,
            _: Vec<super::TrayItem>,
            _: super::Route,
            _: &mut gpui::App,
        ) -> anyhow::Result<Self> {
            anyhow::bail!("tray icons need macOS, Windows or Linux")
        }

        pub fn pick(&self, _: usize) {}
    }
}

/// One entry in a tray menu.
#[derive(Clone, Debug)]
pub enum TrayItem {
    Action(SharedString),
    Separator,
}

type OnPick = Rc<dyn Fn(usize, &mut App)>;

/// How the icon looks: macOS draws the symbol, others the PNG.
pub struct TrayLook {
    /// An SF Symbol, for the macOS menu bar.
    pub symbol: &'static str,
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
pub struct TrayIcon {
    inner: platform::Tray,
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
            inner: platform::Tray::new(&look, items, route, cx)?,
        })
    }

    /// Chooses item `index` as a click would.
    pub fn pick(&self, index: usize) {
        self.inner.pick(index);
    }
}

/// Sets or clears the badge on the app's Dock icon.
pub fn set_dock_badge(label: Option<&str>) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        platform::set_badge(label);
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
pub fn dock_badge() -> anyhow::Result<Option<String>> {
    #[cfg(target_os = "macos")]
    return Ok(platform::badge());
    #[cfg(not(target_os = "macos"))]
    anyhow::bail!("dock badges need macOS in this build")
}
