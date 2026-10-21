use gpui::{
    Context, FocusHandle, IntoElement, KeyBinding, KeyUpEvent, Keystroke, Modifiers, ParentElement,
    Render, Styled, TestAppContext, VisualTestContext, Window, div, point, px,
};

use super::{Button, SegmentedControl, ToggleButton, ToggleGroup, ToggleItem};
use crate::{
    primitives::{FocusNext, FocusScope, IconName},
    theme::Theme,
};

/// One button that counts its presses.
struct Bench {
    root: FocusHandle,
    loading: bool,
    presses: usize,
}

impl Render for Bench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        FocusScope::new(&self.root).root().size_full().child(
            Button::new("save", "Save")
                .loading(self.loading)
                .on_click(move |_, _, cx| view.update(cx, |bench, _| bench.presses += 1)),
        )
    }
}

fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).expect("a key"),
    });
    cx.run_until_parked();
}

#[gpui::test]
fn a_loading_button_ignores_presses_and_keeps_its_tab_stop(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        cx.bind_keys([KeyBinding::new("tab", FocusNext, None)]);
    });
    let (view, cx) = cx.add_window_view(|_, cx| Bench {
        root: cx.focus_handle(),
        loading: true,
        presses: 0,
    });
    let root = view.read_with(cx, |bench, _| bench.root.clone());
    cx.update(|window, cx| window.focus(&root, cx));
    press("tab", cx);
    press("enter", cx);
    assert_eq!(view.read_with(cx, |bench, _| bench.presses), 0);
    view.update(cx, |bench, cx| {
        bench.loading = false;
        cx.notify();
    });
    cx.run_until_parked();
    press("enter", cx);
    assert_eq!(
        view.read_with(cx, |bench, _| bench.presses),
        1,
        "focus stayed on it"
    );
}

/// One toggle that counts its presses.
struct Toggles {
    disabled: bool,
    presses: usize,
}

impl Render for Toggles {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        div().size_full().child(
            ToggleButton::new("bold", ToggleItem::new("bold").label("B"), false)
                .disabled(self.disabled)
                .on_toggle(move |_, _, cx| view.update(cx, |toggles, _| toggles.presses += 1)),
        )
    }
}

#[gpui::test]
fn a_disabled_toggle_ignores_clicks(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (view, cx) = cx.add_window_view(|_, _| Toggles {
        disabled: true,
        presses: 0,
    });
    cx.simulate_click(point(px(4.0), px(4.0)), Modifiers::none());
    assert_eq!(view.read_with(cx, |toggles, _| toggles.presses), 0);
    view.update(cx, |toggles, cx| {
        toggles.disabled = false;
        cx.notify();
    });
    cx.run_until_parked();
    cx.simulate_click(point(px(4.0), px(4.0)), Modifiers::none());
    assert_eq!(view.read_with(cx, |toggles, _| toggles.presses), 1);
}

/// Segments whose chosen value left with a change of list.
struct Stale;

impl Render for Stale {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(300.0)).child(
            SegmentedControl::new("views", "gone")
                .segment("list", "List", None)
                .segment("grid", "Grid", None),
        )
    }
}

#[gpui::test]
fn a_choice_missing_from_its_segments_marks_none(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (_, cx) = cx.add_window_view(|_, _| Stale);
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
    }
    assert!(cx.debug_bounds("segment list").is_some());
    assert!(
        cx.debug_bounds("segment-thumb").is_none(),
        "no thumb marks a choice"
    );
}

/// Three icon-and-word toggles in a column `width` wide.
struct Filters {
    width: f32,
}

impl Render for Filters {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let item = |value: &'static str, icon| ToggleItem::new(value).icon(icon).label(value);
        div().w(px(self.width)).child(
            ToggleGroup::new("filters")
                .multiple()
                .item(item("Unread", IconName::Mail))
                .item(item("Flagged", IconName::Star))
                .item(item("Attachments", IconName::Paperclip)),
        )
    }
}

#[gpui::test]
fn a_narrow_toggle_group_keeps_one_row_of_icons(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (view, cx) = cx.add_window_view(|_, _| Filters { width: 600.0 });
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
    }
    let wide = cx.debug_bounds("toggle Attachments").expect("a toggle");
    assert!(
        wide.size.width > wide.size.height * 2.0,
        "{wide:?} shows its word"
    );
    view.update(cx, |filters, cx| {
        filters.width = 150.0;
        cx.notify();
    });
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
    }
    let mut tops = Vec::new();
    for selector in ["toggle Unread", "toggle Flagged", "toggle Attachments"] {
        tops.push(cx.debug_bounds(selector).expect("a toggle").top());
    }
    assert!(tops.iter().all(|top| *top == tops[0]), "wrapped: {tops:?}");
}
