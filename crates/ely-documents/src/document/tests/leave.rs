use ely_core::primitives::FocusScope;
use gpui::{
    AppContext as _, Context, Entity, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, Styled as _, TestAppContext, VisualTestContext, Window, div, px,
};

use super::MarkdownMode;
use crate::MarkdownDocument;

/// A stop, a document of three blocks, a stop.
struct Host {
    root: FocusHandle,
    before: FocusHandle,
    after: FocusHandle,
    document: Entity<MarkdownDocument>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        FocusScope::new(&self.root)
            .root()
            .size_full()
            .child(div().track_focus(&self.before).h(px(20.0)))
            .child(self.document.clone())
            .child(div().track_focus(&self.after).h(px(20.0)))
    }
}

fn host<'a>(text: &str, cx: &'a mut TestAppContext) -> (Entity<Host>, &'a mut VisualTestContext) {
    cx.update(crate::init_for_tests);
    let text = text.to_string();
    let (host, cx) = cx.add_window_view(move |window, cx| Host {
        root: cx.focus_handle(),
        before: cx.focus_handle().tab_stop(true),
        after: cx.focus_handle().tab_stop(true),
        document: cx.new(|cx| MarkdownDocument::new(&text, MarkdownMode::Visual, window, cx)),
    });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    (host, cx)
}

fn inside(host: &Entity<Host>, cx: &mut VisualTestContext) -> bool {
    cx.update(|window, cx| {
        let document = host.read(cx).document.read(cx).focus.clone();
        document.contains_focused(window, cx)
    })
}

fn focused(
    handle: impl Fn(&Host) -> FocusHandle,
    host: &Entity<Host>,
    cx: &mut VisualTestContext,
) -> bool {
    cx.update(|window, cx| handle(host.read(cx)).is_focused(window))
}

#[gpui::test]
fn f6_leaves_the_document_not_for_its_next_block(cx: &mut TestAppContext) {
    let (host, cx) = host("one\n\ntwo\n\nthree\n", cx);
    cx.update(|window, cx| window.focus(&host.read(cx).before.clone(), cx));
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    assert!(inside(&host, cx), "Tab enters the first block");
    cx.simulate_keystrokes("f6");
    cx.run_until_parked();
    assert!(
        focused(|host| host.after.clone(), &host, cx),
        "F6 passes the other blocks"
    );
    cx.simulate_keystrokes("shift-f6");
    cx.run_until_parked();
    assert!(inside(&host, cx), "Shift-F6 comes back in");
    cx.simulate_keystrokes("shift-f6");
    cx.run_until_parked();
    assert!(
        focused(|host| host.before.clone(), &host, cx),
        "Shift-F6 leaves before it"
    );
}

/// Nothing outside: F6 stops where it began.
struct Alone {
    root: FocusHandle,
    document: Entity<MarkdownDocument>,
}

impl Render for Alone {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        FocusScope::new(&self.root)
            .root()
            .size_full()
            .child(self.document.clone())
    }
}

#[gpui::test]
fn f6_with_nothing_outside_returns(cx: &mut TestAppContext) {
    cx.update(crate::init_for_tests);
    let (alone, cx) = cx.add_window_view(|window, cx| Alone {
        root: cx.focus_handle(),
        document: cx
            .new(|cx| MarkdownDocument::new("one\n\ntwo\n", MarkdownMode::Visual, window, cx)),
    });
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&alone.read(cx).root.clone(), cx);
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    let document = alone.read_with(cx, |alone, cx| alone.document.read(cx).focus.clone());
    assert!(
        cx.update(|window, cx| document.contains_focused(window, cx)),
        "Tab enters"
    );
    cx.simulate_keystrokes("f6");
    cx.run_until_parked();
    assert!(
        cx.update(|window, cx| document.contains_focused(window, cx)),
        "focus stays put"
    );
}
