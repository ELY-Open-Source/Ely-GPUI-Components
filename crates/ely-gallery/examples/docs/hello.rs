use ely_gpui_component::{Assets, buttons::Button, primitives::FocusScope, theme::ActiveTheme};
use gpui::{
    App, AppContext, Context, FocusHandle, IntoElement, ParentElement, Render, Styled, Window,
    WindowOptions,
};

struct Hello {
    focus: FocusHandle,
    presses: usize,
}

impl Render for Hello {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let this = cx.entity();
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .bg(colors.bg)
            .text_color(colors.fg)
            .child(format!("Pressed {} times", self.presses))
            .child(
                Button::new("press", "Press")
                    .primary()
                    .on_click(move |_, _, cx| {
                        this.update(cx, |hello, cx| {
                            hello.presses += 1;
                            cx.notify();
                        });
                    }),
            )
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            ely_gpui_component::init(cx).expect("Ely failed to start");
            cx.open_window(WindowOptions::default(), |window, cx| {
                cx.new(|cx| {
                    let focus = cx.focus_handle();
                    window.focus(&focus, cx);
                    Hello { focus, presses: 0 }
                })
            })
            .expect("window failed to open");
            cx.activate(true);
        });
}
