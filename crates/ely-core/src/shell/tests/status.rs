use gpui::{
    Context, FocusHandle, IntoElement, KeyBinding, KeyUpEvent, Keystroke, ParentElement, Render,
    Styled, TestAppContext, VisualTestContext, Window,
};

use crate::{
    primitives::{FocusNext, FocusScope},
    shell::{StatusBar, StatusBarItem},
    theme::Theme,
};

/// A status bar with one clickable item.
struct Bench {
    root: FocusHandle,
    presses: usize,
}

impl Render for Bench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        FocusScope::new(&self.root).root().size_full().child(
            StatusBar::new()
                .left(StatusBarItem::new("plain").label("Saved"))
                .left(
                    StatusBarItem::new("risk")
                        .label("Unsaved")
                        .on_click(move |_, _, cx| view.update(cx, |bench, _| bench.presses += 1)),
                ),
        )
    }
}

fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse_with_os(
            key,
            crate::typography::keys::os(crate::theme::Platform::current()),
        )
        .expect("a key"),
    });
    cx.run_until_parked();
}

#[gpui::test]
fn a_clickable_status_item_is_a_tab_stop_that_keys_press(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        cx.bind_keys([KeyBinding::new("tab", FocusNext, None)]);
    });
    let (view, cx) = cx.add_window_view(|_, cx| Bench {
        root: cx.focus_handle(),
        presses: 0,
    });
    let root = view.read_with(cx, |bench, _| bench.root.clone());
    cx.update(|window, cx| window.focus(&root, cx));
    press("tab", cx);
    press("enter", cx);
    press("space", cx);
    assert_eq!(
        view.read_with(cx, |bench, _| bench.presses),
        2,
        "Tab reaches only the clickable item"
    );
}
