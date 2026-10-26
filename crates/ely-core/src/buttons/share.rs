use gpui::{
    App, Bounds, ElementId, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled,
    Window, canvas, div,
};

use super::IconButton;
use crate::primitives::IconName;

/// What to share: text, a link, or both.
#[derive(Clone, Debug, Default)]
struct Items {
    text: Option<SharedString>,
    url: Option<SharedString>,
}

struct Anchor {
    bounds: Bounds<Pixels>,
    #[cfg(target_os = "macos")]
    picker: Option<crate::platform::mac::Picker>,
}

/// Opens the system share picker beside itself. Elsewhere it shows disabled.
///
/// Platforms: macOS.
///
/// ```
/// use ely_core::buttons::ShareButton;
///
/// let share = ShareButton::new("share")
///     .text("Ely ships 0.2")
///     .url("https://example.com/blog/0-2");
/// ```
#[derive(IntoElement)]
pub struct ShareButton {
    id: ElementId,
    items: Items,
}

impl ShareButton {
    /// A share button under `id`, with nothing to share yet.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Items::default(),
        }
    }

    /// Text to share.
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.items.text = Some(text.into());
        self
    }

    /// An address to share.
    pub fn url(mut self, url: impl Into<SharedString>) -> Self {
        self.items.url = Some(url.into());
        self
    }
}

impl RenderOnce for ShareButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        assert!(
            self.items.text.is_some() || self.items.url.is_some(),
            "share button {:?} has nothing to share",
            self.id
        );
        let anchor = window.use_keyed_state(self.id.clone(), cx, |_, _| Anchor {
            bounds: Bounds::default(),
            #[cfg(target_os = "macos")]
            picker: None,
        });
        let (measure, open) = (anchor.clone(), anchor);
        let items = self.items;
        let button = IconButton::new(self.id, IconName::Share2).tooltip("Share");
        #[cfg(target_os = "macos")]
        let button = button.on_click(move |_, window, cx| {
            let bounds = open.read(cx).bounds;
            match show(&items, bounds, window) {
                Ok(picker) => open.update(cx, |anchor, _| anchor.picker = Some(picker)),
                Err(error) => log::error!("share button: {error:#}"),
            }
        });
        #[cfg(not(target_os = "macos"))]
        let button = {
            let _ = (open, items);
            button
                .disabled(true)
                .tooltip("Sharing needs macOS in this build")
        };
        div().relative().child(button).child(
            canvas(
                move |bounds, _, cx| {
                    if measure.read(cx).bounds != bounds {
                        measure.update(cx, |anchor, _| anchor.bounds = bounds);
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
        )
    }
}

/// Shows the picker under `bounds`, in the window's AppKit view.
#[cfg(target_os = "macos")]
fn show(
    items: &Items,
    bounds: Bounds<Pixels>,
    window: &Window,
) -> anyhow::Result<crate::platform::mac::Picker> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = HasWindowHandle::window_handle(window)
        .map_err(|error| anyhow::anyhow!("window has no native handle: {error:?}"))?;
    let RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        anyhow::bail!("share needs an AppKit window");
    };
    // AppKit's view counts y from its bottom.
    let height = f64::from(window.viewport_size().height);
    let rect = [
        f64::from(bounds.left()),
        height - f64::from(bounds.bottom()),
        f64::from(bounds.size.width),
        f64::from(bounds.size.height),
    ];
    crate::platform::mac::share(
        items.text.as_deref(),
        items.url.as_deref(),
        appkit.ns_view.as_ptr(),
        rect,
    )
}
