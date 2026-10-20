use gpui::{Half, Pixels, Point, point, px};

use super::Theme;

/// Whose conventions the components follow: window buttons, the title bar's layout and key caps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Mac,
    Windows,
    Linux,
}

impl Platform {
    /// The system this build targets, where the theme starts.
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::Mac
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Linux
        }
    }
}

/// The system traffic lights' origin, centered in the theme's title bar.
pub(crate) fn traffic_light_origin(theme: &Theme) -> Point<Pixels> {
    let rem = theme.base_rem();
    let drawn = theme.traffic_light().to_pixels(rem);
    point(
        px(14.0),
        (theme.titlebar_height().to_pixels(rem) - light_height(drawn)).half(),
    )
}

/// AppKit sizes its window buttons per release: 16 tall on macOS 15, 14 on 26.
#[cfg(target_os = "macos")]
fn light_height(_drawn: Pixels) -> Pixels {
    use cocoa::{
        base::{id, nil},
        foundation::NSRect,
    };
    use objc::{class, msg_send, sel, sel_impl};
    // Titled, closable, miniaturizable, resizable, full-size content: the mask gpui opens windows with.
    let mask: u64 = 1 | 2 | 4 | 8 | 1 << 15;
    let frame: NSRect = unsafe {
        let close: id = msg_send![class!(NSWindow), standardWindowButton: 0u64 forStyleMask: mask];
        assert!(
            close != nil,
            "theme: AppKit gave no close button to size the traffic lights by"
        );
        msg_send![close, frame]
    };
    px(frame.size.height as f32)
}

#[cfg(not(target_os = "macos"))]
fn light_height(drawn: Pixels) -> Pixels {
    drawn
}
