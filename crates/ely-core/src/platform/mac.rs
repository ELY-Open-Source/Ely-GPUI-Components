//! AppKit through objc2: window buttons, sharing and color sampling.

use std::{cell::RefCell, ffi::c_void};

use anyhow::Context as _;
use block2::RcBlock;
use objc2::{MainThreadMarker, rc::Retained, runtime::AnyObject};
use objc2_app_kit::{
    NSColor, NSColorSampler, NSColorSpace, NSSharingServicePicker, NSView, NSWindow,
    NSWindowButton, NSWindowStyleMask,
};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSRectEdge, NSSize, NSString, NSURL};

/// AppKit's views and menus live on the main thread; gpui calls from there.
fn main_thread(what: &str) -> MainThreadMarker {
    MainThreadMarker::new().unwrap_or_else(|| panic!("{what} runs on the main thread"))
}

/// The height AppKit gives a window's buttons, in points.
pub(crate) fn window_button_height() -> f64 {
    let mtm = main_thread("sizing the traffic lights");
    // The mask gpui opens windows with.
    let mask = NSWindowStyleMask::Titled
        | NSWindowStyleMask::Closable
        | NSWindowStyleMask::Miniaturizable
        | NSWindowStyleMask::Resizable
        | NSWindowStyleMask::FullSizeContentView;
    let close = NSWindow::standardWindowButton_forStyleMask(NSWindowButton::CloseButton, mask, mtm)
        .expect("AppKit gives a close button to size the traffic lights by");
    close.frame().size.height
}

/// A shown share picker, kept until the next share.
pub(crate) struct Picker {
    _shown: Retained<NSSharingServicePicker>,
}

/// Shows AppKit's share picker under `rect` of the view at `view`, in its flipped points.
pub(crate) fn share(
    text: Option<&str>,
    url: Option<&str>,
    view: *mut c_void,
    rect: [f64; 4],
) -> anyhow::Result<Picker> {
    let mtm = main_thread("sharing");
    let mut items: Vec<Retained<AnyObject>> = Vec::new();
    if let Some(text) = text {
        items.push(Retained::into_super(Retained::into_super(
            NSString::from_str(text),
        )));
    }
    if let Some(url) = url {
        let link = NSURL::URLWithString(&NSString::from_str(url))
            .with_context(|| format!("share url {url} does not parse"))?;
        items.push(Retained::into_super(Retained::into_super(link)));
    }
    let items = NSArray::from_retained_slice(&items);
    // SAFETY: the items are strings and URLs, objects a share picker takes.
    let picker = unsafe { NSSharingServicePicker::initWithItems(mtm.alloc(), &items) };
    // SAFETY: gpui's raw window handle holds the window's live content view.
    let view = unsafe { &*view.cast::<NSView>() };
    let [x, y, width, height] = rect;
    let rect = NSRect::new(NSPoint::new(x, y), NSSize::new(width, height));
    picker.showRelativeToRect_ofView_preferredEdge(rect, view, NSRectEdge::MinY);
    log::info!("share button: picker shown for {} items", items.len());
    Ok(Picker { _shown: picker })
}

/// An `NSColor` as sRGB components, `None` when it has no sRGB form.
fn srgb(color: &NSColor) -> Option<[f64; 4]> {
    let srgb = color.colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())?;
    Some([
        srgb.redComponent(),
        srgb.greenComponent(),
        srgb.blueComponent(),
        srgb.alphaComponent(),
    ])
}

/// Opens AppKit's color sampler; `on_color` gets the sRGB pick, or `None` when cancelled or unconvertible.
pub(crate) fn sample(on_color: impl FnOnce(Option<[f64; 4]>) + 'static) {
    let _ = main_thread("sampling a color");
    let once = RefCell::new(Some(on_color));
    let handler = RcBlock::new(move |color: *mut NSColor| {
        // SAFETY: AppKit hands a live color, or nil when cancelled, for this call.
        let color = unsafe { color.as_ref() };
        let picked = color.and_then(|color| {
            let read = srgb(color);
            if read.is_none() {
                log::error!("eye dropper: the color has no sRGB form");
            }
            read
        });
        match once.borrow_mut().take() {
            Some(on_color) => on_color(picked),
            None => log::error!("eye dropper: AppKit answered twice"),
        }
    });
    // AppKit keeps the sampler until its session ends.
    let sampler = NSColorSampler::new();
    // SAFETY: the block takes the color AppKit passes, as the selector declares.
    unsafe { sampler.showSamplerWithSelectionHandler(&handler) };
}

#[cfg(test)]
mod tests {
    use objc2_app_kit::NSColor;

    use super::srgb;

    #[test]
    fn a_display_p3_color_reads_back_as_srgb() {
        let color = NSColor::colorWithDisplayP3Red_green_blue_alpha(1.0, 0.0, 0.0, 0.5);
        let [r, g, b, a] = srgb(&color).expect("P3 red converts to sRGB");
        assert!(r > 0.99 && g < 0.01 && b < 0.01, "{r} {g} {b}");
        assert!((a - 0.5).abs() < 1e-6);
    }
}
