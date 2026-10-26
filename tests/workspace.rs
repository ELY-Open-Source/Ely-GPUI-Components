//! A workspace with a sidebar, a confirm dialog and toasts, built from Ely's public API alone.

use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    feedback::{Toast, ToastViewport, Toaster},
    init_for_tests,
    layout::Sidebar,
    overlays::Dialog,
    primitives::FocusScope,
};
use gpui::{
    AppContext as _, Context, Entity, FocusHandle, IntoElement, KeyUpEvent, Keystroke,
    ParentElement, Render, SharedString, Styled, TestAppContext, VisualTestContext, Window, div,
};

struct Workspace {
    root: FocusHandle,
    delete: FocusHandle,
    toaster: Entity<Toaster>,
    asking: bool,
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (ask, shut, done) = (cx.entity(), cx.entity(), cx.entity());
        let toaster = self.toaster.clone();
        let dialog = self.asking.then(|| {
            Dialog::new("confirm", "Delete the draft?", move |_, cx| {
                shut.update(cx, |view, cx| {
                    view.asking = false;
                    cx.notify();
                })
            })
            .action(move |close| {
                Button::new("confirm-delete", "Delete")
                    .variant(ButtonVariant::Danger)
                    .on_click(move |_, window, cx| {
                        close(window, cx);
                        toaster.update(cx, |toaster, cx| {
                            toaster.push(Toast::new("Draft deleted"), cx);
                        });
                        done.update(cx, |_, cx| cx.notify());
                    })
            })
        });
        FocusScope::new(&self.root)
            .root()
            .size_full()
            .flex()
            .child(Sidebar::new("nav", false).child("Drafts"))
            .child(
                div().flex_1().child(
                    Button::new("delete", "Delete draft")
                        .focus_handle(&self.delete)
                        .on_click(move |_, _, cx| {
                            ask.update(cx, |view, cx| {
                                view.asking = true;
                                cx.notify();
                            })
                        }),
                ),
            )
            .children(dialog)
            .child(ToastViewport::new("toasts", &self.toaster))
    }
}

fn workspace(cx: &mut TestAppContext) -> (Entity<Workspace>, &mut VisualTestContext) {
    cx.update(init_for_tests);
    let (view, cx) = cx.add_window_view(|_, cx| Workspace {
        root: cx.focus_handle(),
        delete: cx.focus_handle().tab_stop(true),
        toaster: cx.new(|_| Toaster::default()),
        asking: false,
    });
    let delete = view.read_with(cx, |view, _| view.delete.clone());
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&delete, cx);
    });
    settle(cx);
    (view, cx)
}

fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    settle(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke:
            Keystroke::parse_with_os(
                key,
                ely_gpui_component::typography::keys::os(
                    ely_gpui_component::theme::Platform::current(),
                ),
            )
            .expect("a key"),
    });
    settle(cx);
}

fn asking(view: &Entity<Workspace>, cx: &mut VisualTestContext) -> bool {
    view.read_with(cx, |view, _| view.asking)
}

#[gpui::test]
fn escape_closes_the_dialog_and_hands_focus_back(cx: &mut TestAppContext) {
    let (view, cx) = workspace(cx);
    press("enter", cx);
    assert!(asking(&view, cx), "Enter on the button opens the dialog");
    let delete = view.read_with(cx, |view, _| view.delete.clone());
    assert!(
        !cx.update(|window, _| delete.is_focused(window)),
        "the dialog holds focus"
    );
    press("escape", cx);
    assert!(!asking(&view, cx));
    assert!(
        cx.update(|window, _| delete.is_focused(window)),
        "focus is back on the button"
    );
}

#[gpui::test]
fn confirming_closes_the_dialog_and_shows_a_toast(cx: &mut TestAppContext) {
    let (view, cx) = workspace(cx);
    press("enter", cx);
    press("tab", cx);
    press("tab", cx);
    press("enter", cx);
    assert!(!asking(&view, cx), "Delete closes the dialog");
    let toaster = view.read_with(cx, |view, _| view.toaster.clone());
    let shown: Vec<SharedString> = toaster.read_with(cx, |toaster, _| {
        toaster
            .shown()
            .into_iter()
            .map(|(title, _)| title)
            .collect()
    });
    assert_eq!(shown, ["Draft deleted"]);
}
