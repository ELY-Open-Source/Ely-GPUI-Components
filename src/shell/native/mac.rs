//! macOS: an SF Symbol in the menu bar.

use std::{ffi::CStr, sync::Once};

use cocoa::{
    appkit::{NSApp, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength},
    base::{id, nil},
    foundation::{NSAutoreleasePool, NSString},
};
use gpui::App;
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{Class, Object, Sel},
    sel, sel_impl,
};

use super::{Look, Route, TrayItem};

const TARGET: &str = "ElyTrayTarget";
const ROUTE: &str = "elyRoute";

extern "C" fn picked(this: &Object, _: Sel, item: id) {
    unsafe {
        let route = *this.get_ivar::<*const std::ffi::c_void>(ROUTE) as *const Route;
        let tag: isize = msg_send![item, tag];
        (*route).pick(tag as usize);
    }
}

fn target_class() -> &'static Class {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        let mut decl = ClassDecl::new(TARGET, class!(NSObject))
            .unwrap_or_else(|| panic!("objc class {TARGET} already exists"));
        decl.add_ivar::<*const std::ffi::c_void>(ROUTE);
        unsafe {
            decl.add_method(sel!(elyPicked:), picked as extern "C" fn(&Object, Sel, id));
        }
        decl.register();
    });
    Class::get(TARGET).expect("registered above")
}

fn string(text: &str) -> id {
    unsafe { NSString::alloc(nil).init_str(text).autorelease() }
}

pub struct Tray {
    item: id,
    menu: id,
    target: id,
    _route: Box<Route>,
}

impl Tray {
    pub fn new(
        look: &Look,
        items: Vec<TrayItem>,
        route: Route,
        _: &mut App,
    ) -> anyhow::Result<Self> {
        let (symbol, route) = (look.symbol, Box::new(route));
        unsafe {
            let image: id = msg_send![class!(NSImage),
                imageWithSystemSymbolName: string(symbol)
                accessibilityDescription: nil];
            anyhow::ensure!(image != nil, "no SF Symbol named {symbol}");
            let target: id = msg_send![target_class(), new];
            (*target).set_ivar(ROUTE, &*route as *const Route as *const std::ffi::c_void);
            let menu = NSMenu::new(nil);
            for (index, entry) in items.iter().enumerate() {
                let entry = match entry {
                    TrayItem::Separator => NSMenuItem::separatorItem(nil),
                    TrayItem::Action(label) => {
                        let entry = NSMenuItem::alloc(nil)
                            .initWithTitle_action_keyEquivalent_(
                                string(label),
                                sel!(elyPicked:),
                                string(""),
                            )
                            .autorelease();
                        entry.setTarget_(target);
                        entry
                    }
                };
                let _: () = msg_send![entry, setTag: index as isize];
                menu.addItem_(entry);
            }
            let item =
                NSStatusBar::systemStatusBar(nil).statusItemWithLength_(NSVariableStatusItemLength);
            let item: id = msg_send![item, retain];
            let button = item.button();
            anyhow::ensure!(button != nil, "status item has no button");
            let _: () = msg_send![button, setImage: image];
            let _: () = msg_send![button, setToolTip: string(&look.name)];
            let _: () = msg_send![item, setMenu: menu];
            log::info!("tray: shown with {} items", items.len());
            Ok(Self {
                item,
                menu,
                target,
                _route: route,
            })
        }
    }

    pub fn pick(&self, index: usize) {
        unsafe {
            let count: isize = msg_send![self.menu, numberOfItems];
            assert!((index as isize) < count, "tray has no item {index}");
            let _: () = msg_send![self.menu, performActionForItemAtIndex: index as isize];
        }
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            NSStatusBar::systemStatusBar(nil).removeStatusItem_(self.item);
            let _: () = msg_send![self.item, release];
            let _: () = msg_send![self.menu, release];
            let _: () = msg_send![self.target, release];
        }
        log::info!("tray: removed");
    }
}

fn tile() -> id {
    unsafe { msg_send![NSApp(), dockTile] }
}

pub fn set_badge(label: Option<&str>) {
    unsafe {
        let text = label.map_or(nil, string);
        let _: () = msg_send![tile(), setBadgeLabel: text];
    }
}

pub fn badge() -> Option<String> {
    unsafe {
        let label: id = msg_send![tile(), badgeLabel];
        (label != nil).then(|| {
            CStr::from_ptr(label.UTF8String())
                .to_string_lossy()
                .into_owned()
        })
    }
}
