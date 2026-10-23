//! AppKit through objc2: the menu bar's status item and the Dock badge.

use anyhow::Context as _;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained,
    runtime::NSObject, sel,
};
use objc2_app_kit::{
    NSApplication, NSImage, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::NSString;

/// AppKit's menus and the Dock live on the main thread; gpui calls from there.
fn main_thread(what: &str) -> MainThreadMarker {
    MainThreadMarker::new().unwrap_or_else(|| panic!("{what} runs on the main thread"))
}

/// What a menu target hands a pick to.
type OnPick = Box<dyn Fn(usize)>;

define_class!(
    // SAFETY: NSObject has no subclassing rules; the class adds no Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "ElyTrayTarget"]
    #[ivars = OnPick]
    struct Target;

    impl Target {
        #[unsafe(method(elyPicked:))]
        fn picked(&self, item: &NSMenuItem) {
            (self.ivars())(item.tag() as usize);
        }
    }
);

impl Target {
    fn new(on_pick: OnPick, mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(on_pick);
        // SAFETY: NSObject's init takes no arguments and returns the object.
        unsafe { msg_send![super(this), init] }
    }
}

/// An SF Symbol in the menu bar with a menu; dropping it removes it.
pub(crate) struct StatusItem {
    item: Retained<NSStatusItem>,
    menu: Retained<NSMenu>,
    _target: Retained<Target>,
}

impl StatusItem {
    /// `entries` are labels, `None` a separator; `on_pick` gets a picked entry's index.
    pub(crate) fn new(
        symbol: &str,
        name: &str,
        entries: &[Option<&str>],
        on_pick: OnPick,
    ) -> anyhow::Result<Self> {
        let mtm = main_thread("the tray");
        let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str(symbol),
            None,
        )
        .with_context(|| format!("no SF Symbol named {symbol}"))?;
        let target = Target::new(on_pick, mtm);
        let menu = NSMenu::new(mtm);
        for (index, entry) in entries.iter().enumerate() {
            let entry = match entry {
                None => NSMenuItem::separatorItem(mtm),
                Some(label) => {
                    // SAFETY: the target answers `elyPicked:` with a menu item.
                    let entry = unsafe {
                        NSMenuItem::initWithTitle_action_keyEquivalent(
                            mtm.alloc(),
                            &NSString::from_str(label),
                            Some(sel!(elyPicked:)),
                            &NSString::new(),
                        )
                    };
                    // SAFETY: the menu item keeps no ownership of its target, which `self` keeps.
                    unsafe { entry.setTarget(Some(&target)) };
                    entry
                }
            };
            entry.setTag(index as isize);
            menu.addItem(&entry);
        }
        let item = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
        let button = item.button(mtm).context("status item has no button")?;
        button.setImage(Some(&image));
        button.setToolTip(Some(&NSString::from_str(name)));
        item.setMenu(Some(&menu));
        log::info!("tray: shown with {} items", entries.len());
        Ok(Self {
            item,
            menu,
            _target: target,
        })
    }

    /// Chooses entry `index` as a click would.
    pub(crate) fn pick(&self, index: usize) {
        assert!(
            (index as isize) < self.menu.numberOfItems(),
            "tray has no item {index}"
        );
        self.menu.performActionForItemAtIndex(index as isize);
    }
}

impl Drop for StatusItem {
    fn drop(&mut self) {
        NSStatusBar::systemStatusBar().removeStatusItem(&self.item);
        log::info!("tray: removed");
    }
}

/// Sets or clears the badge on the app's Dock icon.
pub(crate) fn set_dock_badge(label: Option<&str>) {
    let mtm = main_thread("the Dock badge");
    let label = label.map(NSString::from_str);
    NSApplication::sharedApplication(mtm)
        .dockTile()
        .setBadgeLabel(label.as_deref());
}

/// The badge the Dock shows now.
pub(crate) fn dock_badge() -> Option<String> {
    let mtm = main_thread("the Dock badge");
    let label = NSApplication::sharedApplication(mtm)
        .dockTile()
        .badgeLabel();
    label.map(|label| label.to_string())
}
