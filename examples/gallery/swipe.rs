use anyhow::Result;
use gpui::{AsyncApp, Pixels, Point, WindowHandle};

use crate::{capture::number, shell::Gallery};

/// A scroll's phase, as AppKit numbers them.
pub const BEGAN: i64 = 1;
pub const CHANGED: i64 = 2;
pub const ENDED: i64 = 4;

pub fn swipe(
    window: WindowHandle<Gallery>,
    at: Point<Pixels>,
    across: f32,
    phase: i64,
    cx: &mut AsyncApp,
) -> Result<()> {
    let (height, number) = window.update(cx, |_, window, _| {
        (window.viewport_size().height, number(window))
    })?;
    post_scroll(
        f64::from(at.x),
        f64::from(height - at.y),
        across,
        phase,
        number?,
    )
}

/// Sends a real trackpad scroll to the window, sideways by `across` points, with its phase. Quartz scrolls carry no window, so AppKit reads their location as window points and the window takes them directly.
#[cfg(target_os = "macos")]
fn post_scroll(x: f64, y: f64, across: f32, phase: i64, number: u32) -> Result<()> {
    use cocoa::{
        appkit::NSApp,
        base::{id, nil},
        foundation::NSRect,
    };
    use objc::{class, msg_send, sel, sel_impl};
    type Event = *mut std::ffi::c_void;
    #[repr(C)]
    struct Spot {
        x: f64,
        y: f64,
    }
    unsafe extern "C" {
        fn CGEventCreateScrollWheelEvent2(
            source: *const std::ffi::c_void,
            units: u32,
            count: u32,
            wheel1: i32,
            wheel2: i32,
            wheel3: i32,
        ) -> Event;
        fn CGEventSetIntegerValueField(event: Event, field: u32, value: i64);
        fn CGEventSetLocation(event: Event, at: Spot);
        fn CFRelease(object: *const std::ffi::c_void);
    }
    const PIXELS: u32 = 0;
    const CONTINUOUS: u32 = 88;
    const PHASE: u32 = 99;
    // SAFETY: main thread; window and events are checked for nil, the CGEvent freed once.
    unsafe {
        let window: id = msg_send![NSApp(), windowWithWindowNumber: i64::from(number)];
        anyhow::ensure!(window != nil, "no window numbered {number}");
        let screens: id = msg_send![class!(NSScreen), screens];
        let primary: id = msg_send![screens, objectAtIndex: 0usize];
        let frame: NSRect = msg_send![primary, frame];
        let event = CGEventCreateScrollWheelEvent2(
            std::ptr::null(),
            PIXELS,
            2,
            0,
            across.round() as i32,
            0,
        );
        anyhow::ensure!(!event.is_null(), "Quartz refused a scroll event");
        CGEventSetLocation(
            event,
            Spot {
                x,
                y: frame.size.height - y,
            },
        );
        CGEventSetIntegerValueField(event, CONTINUOUS, 1);
        CGEventSetIntegerValueField(event, PHASE, phase);
        let scroll: id = msg_send![class!(NSEvent), eventWithCGEvent: event];
        CFRelease(event);
        anyhow::ensure!(scroll != nil, "AppKit refused a scroll event");
        let _: () = msg_send![window, sendEvent: scroll];
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn post_scroll(_: f64, _: f64, _: f32, _: i64, _: u32) -> Result<()> {
    anyhow::bail!("scripted scrolls need macOS")
}
