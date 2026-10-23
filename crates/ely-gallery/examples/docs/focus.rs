use ely_gpui_component::{
    Assets,
    buttons::Button,
    primitives::{FocusRing, FocusScope},
    theme::{ActiveTheme, Radius},
};
use gpui::{
    App, AppContext, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowOptions, div, transparent_black,
};

struct Panel {
    root: FocusHandle,
    tile: FocusHandle,
    on: bool,
}

impl Render for Panel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let this = cx.entity();
        let tile = div()
            .id("tile")
            .track_focus(&self.tile)
            .p_4()
            .border_1()
            .border_color(transparent_black())
            .rounded(theme.radius(Radius::Md))
            .bg(theme.colors.surface)
            .focus_ring(cx)
            .child(if self.on { "On" } else { "Off" })
            .on_click(move |_, _, cx| {
                this.update(cx, |panel, cx| {
                    panel.on = !panel.on;
                    cx.notify();
                })
            });
        FocusScope::new(&self.root)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .bg(theme.colors.bg)
            .text_color(theme.colors.fg)
            .child(Button::new("first", "First"))
            .child(tile)
            .child(Button::new("last", "Last"))
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            ely_gpui_component::init(cx).expect("Ely failed to start");
            cx.open_window(WindowOptions::default(), |window, cx| {
                cx.new(|cx| {
                    let root = cx.focus_handle();
                    window.focus(&root, cx);
                    Panel {
                        root,
                        tile: cx.focus_handle().tab_stop(true),
                        on: false,
                    }
                })
            })
            .expect("window failed to open");
            cx.activate(true);
        });
}
