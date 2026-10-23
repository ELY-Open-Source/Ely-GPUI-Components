use ely_gpui_component::{
    Assets,
    buttons::Button,
    feedback::{Toast, ToastViewport, Toaster},
    overlays::{Dialog, Popover},
    primitives::{FocusScope, Severity},
    theme::ActiveTheme,
};
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, IntoElement, ParentElement, Render, Styled,
    Window, WindowOptions, div,
};

struct Layers {
    focus: FocusHandle,
    toaster: Entity<Toaster>,
    open: bool,
}

impl Render for Layers {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (opener, closer) = (cx.entity(), cx.entity());
        let toaster = self.toaster.clone();
        let dialog = self.open.then(|| {
            Dialog::new("delete", "Delete notes.txt?", move |_, cx| {
                closer.update(cx, |layers, cx| {
                    layers.open = false;
                    cx.notify();
                })
            })
            .detail("It leaves the trash after 30 days.")
            .action(|close| {
                Button::new("cancel", "Cancel").on_click(move |_, window, cx| close(window, cx))
            })
            .action(move |close| {
                Button::new("confirm", "Delete")
                    .primary()
                    .on_click(move |_, window, cx| {
                        close(window, cx);
                        toaster.update(cx, |toaster, cx| {
                            toaster.push(Toast::new("Deleted").severity(Severity::Success), cx);
                        });
                    })
            })
        });
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .bg(theme.colors.bg)
            .text_color(theme.colors.fg)
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(Popover::new("details", "Details", |_, _| {
                        div().p_3().child("Size 4 KB · Edited today")
                    }))
                    .child(
                        Button::new("delete-open", "Delete…").on_click(move |_, _, cx| {
                            opener.update(cx, |layers, cx| {
                                layers.open = true;
                                cx.notify();
                            })
                        }),
                    ),
            )
            .children(dialog)
            .child(ToastViewport::new("toasts", &self.toaster))
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
                    Layers {
                        focus,
                        toaster: cx.new(|_| Toaster::default()),
                        open: false,
                    }
                })
            })
            .expect("window failed to open");
            cx.activate(true);
        });
}
