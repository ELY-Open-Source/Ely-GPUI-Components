use std::rc::Rc;

use gpui::{AnyWindowHandle, App, AsyncApp, ElementId, Hsla, IntoElement, RenderOnce, Window};

use crate::{
    buttons::{ButtonVariant, IconButton},
    primitives::IconName,
};

type OnColor = Rc<dyn Fn(Hsla, &mut Window, &mut App)>;

/// A button that picks a color from anywhere on screen. Elsewhere it shows disabled.
///
/// Platforms: macOS.
///
/// ```
/// use ely_core::forms::EyeDropper;
///
/// let sample = EyeDropper::new("sample").on_pick(|_color, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct EyeDropper {
    id: ElementId,
    on_pick: Option<OnColor>,
}

impl EyeDropper {
    /// A dropper button under `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            on_pick: None,
        }
    }

    /// Runs with the color picked from the screen.
    pub fn on_pick(mut self, handler: impl Fn(Hsla, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for EyeDropper {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let on_pick = self.on_pick;
        let available = cfg!(target_os = "macos");
        IconButton::new(self.id, IconName::Pipette)
            .variant(ButtonVariant::Ghost)
            .tooltip(if available {
                "Pick a color from the screen"
            } else {
                "Picking from the screen needs macOS"
            })
            .disabled(!available)
            .on_click(move |_, window, cx| {
                sample(window.window_handle(), cx.to_async(), on_pick.clone())
            })
    }
}

/// Opens AppKit's color sampler; its answer comes back on the main thread.
#[cfg(target_os = "macos")]
fn sample(window: AnyWindowHandle, cx: AsyncApp, on_pick: Option<OnColor>) {
    log::info!("eye dropper: sampling");
    crate::platform::mac::sample(move |picked| {
        let Some([r, g, b, a]) = picked else {
            log::info!("eye dropper: no color");
            return;
        };
        let rgba = gpui::Rgba {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        };
        log::info!("eye dropper: picked {}", super::hex(rgba));
        let Some(on_pick) = on_pick else {
            return;
        };
        let mut cx = cx;
        if let Err(error) = window.update(&mut cx, |_, window, cx| on_pick(rgba.into(), window, cx))
        {
            log::error!("eye dropper: the window closed first: {error:#}");
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn sample(_: AnyWindowHandle, _: AsyncApp, _: Option<OnColor>) {
    unreachable!("the eye dropper stays disabled off macOS");
}
