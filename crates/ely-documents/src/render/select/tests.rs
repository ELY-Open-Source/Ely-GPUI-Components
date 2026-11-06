use std::{cell::RefCell, rc::Rc};

use gpui::{
    ClipboardItem, Context, IntoElement, Modifiers, MouseButton, Render, TestAppContext,
    VisualTestContext, Window, div, point, prelude::*, px,
};

use super::RightPress;
use crate::{render::MarkdownRenderer, theme::Theme};

/// Selectable `source`, its right presses kept.
struct Body {
    source: &'static str,
    presses: Rc<RefCell<Vec<RightPress>>>,
}

impl Render for Body {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let presses = self.presses.clone();
        div().w(px(400.0)).child(
            div().debug_selector(|| "body".into()).child(
                MarkdownRenderer::new("body", self.source)
                    .selectable(move |press, _, _| presses.borrow_mut().push(press)),
            ),
        )
    }
}

fn opened<'a>(
    cx: &'a mut TestAppContext,
    source: &'static str,
) -> (gpui::Entity<Body>, &'a mut VisualTestContext) {
    cx.update(Theme::init);
    let presses = Rc::default();
    let (body, cx) = cx.add_window_view(|_, _| Body { source, presses });
    cx.run_until_parked();
    (body, cx)
}

fn copied(cx: &mut VisualTestContext) -> Option<String> {
    cx.write_to_clipboard(ClipboardItem::new_string("before".into()));
    cx.simulate_keystrokes("secondary-c");
    cx.read_from_clipboard()
        .and_then(|item| item.text())
        .filter(|text| text != "before")
}

fn drag(
    cx: &mut VisualTestContext,
    from: gpui::Point<gpui::Pixels>,
    to: gpui::Point<gpui::Pixels>,
) {
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::none());
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

const TWO: &str = "One two\n\n三 four\n";

#[gpui::test]
fn select_all_copies_every_block_on_its_line(cx: &mut TestAppContext) {
    let (_, cx) = opened(cx, TWO);
    let body = cx.debug_bounds("body").expect("drawn");
    cx.simulate_click(body.origin + point(px(2.0), px(4.0)), Modifiers::none());
    assert_eq!(copied(cx), None, "a click selects nothing");
    cx.simulate_keystrokes("secondary-a");
    assert_eq!(copied(cx).as_deref(), Some("One two\n三 four"));
}

#[gpui::test]
fn a_drag_selects_forward_and_back(cx: &mut TestAppContext) {
    let (_, cx) = opened(cx, TWO);
    let body = cx.debug_bounds("body").expect("drawn");
    let start = body.origin + point(px(0.0), px(4.0));
    drag(
        cx,
        start,
        point(body.right() - px(1.0), body.top() + px(4.0)),
    );
    assert_eq!(
        copied(cx).as_deref(),
        Some("One two"),
        "the first line's end"
    );
    drag(
        cx,
        point(body.right() - px(1.0), body.bottom() - px(1.0)),
        start,
    );
    assert_eq!(
        copied(cx).as_deref(),
        Some("One two\n三 four"),
        "back from the last line's end"
    );
}

#[gpui::test]
fn new_text_drops_the_selection(cx: &mut TestAppContext) {
    let (view, cx) = opened(cx, TWO);
    let body = cx.debug_bounds("body").expect("drawn");
    cx.simulate_click(body.origin + point(px(2.0), px(4.0)), Modifiers::none());
    cx.simulate_keystrokes("secondary-a");
    cx.update(|_, app| {
        view.update(app, |body, cx| {
            body.source = "Another mail\n";
            cx.notify();
        })
    });
    cx.run_until_parked();
    assert_eq!(copied(cx), None, "nothing of the old text");
}

#[gpui::test]
fn a_right_press_names_the_link_and_the_selection(cx: &mut TestAppContext) {
    let (view, cx) = opened(cx, "[site](https://example.com/) and more\n");
    let body = cx.debug_bounds("body").expect("drawn");
    let on_link = body.origin + point(px(2.0), px(4.0));
    cx.simulate_click(on_link, Modifiers::none());
    cx.simulate_keystrokes("secondary-a");
    cx.simulate_mouse_down(on_link, MouseButton::Right, Modifiers::none());
    cx.run_until_parked();
    let presses = cx.update(|_, app| view.read(app).presses.borrow().clone());
    let [press] = presses.as_slice() else {
        panic!("one press: {presses:?}")
    };
    assert_eq!(press.link.as_deref(), Some("https://example.com/"));
    assert_eq!(press.selected.as_deref(), Some("site and more"));
    assert_eq!(press.image, None);
}

#[gpui::test]
fn a_right_press_on_a_picture_names_it(cx: &mut TestAppContext) {
    let (view, cx) = opened(cx, "![a cat](https://example.com/cat.png)\n");
    let body = cx.debug_bounds("body").expect("drawn");
    cx.simulate_mouse_down(body.center(), MouseButton::Right, Modifiers::none());
    cx.run_until_parked();
    let presses = cx.update(|_, app| view.read(app).presses.borrow().clone());
    let [press] = presses.as_slice() else {
        panic!("one press: {presses:?}")
    };
    assert_eq!(press.image.as_deref(), Some("https://example.com/cat.png"));
    assert_eq!(press.selected, None);
}

#[gpui::test]
fn shorter_text_in_as_many_blocks_drops_the_selection(cx: &mut TestAppContext) {
    let long = "Long first paragraph here\n\n很长的第二段文字在这里\n";
    let (view, cx) = opened(cx, long);
    let body = cx.debug_bounds("body").expect("drawn");
    cx.simulate_click(body.origin + point(px(2.0), px(4.0)), Modifiers::none());
    cx.simulate_keystrokes("secondary-a");
    cx.update(|_, app| {
        view.update(app, |body, cx| {
            body.source = "x\n\n字\n";
            cx.notify();
        })
    });
    cx.run_until_parked();
    assert_eq!(copied(cx), None, "the old span fits no new text");
    cx.simulate_keystrokes("secondary-a");
    assert_eq!(copied(cx).as_deref(), Some("x\n字"));
}

#[gpui::test]
fn lists_copy_with_their_markers(cx: &mut TestAppContext) {
    let (_, cx) = opened(cx, "3. first\n4. second\n   - inner\n\n- [x] done\n");
    let body = cx.debug_bounds("body").expect("drawn");
    cx.simulate_click(body.origin + point(px(2.0), px(4.0)), Modifiers::none());
    cx.simulate_keystrokes("secondary-a");
    assert_eq!(
        copied(cx).as_deref(),
        Some("3. first\n4. second\n  ◦ inner\n[x] done")
    );
}

#[test]
fn a_span_ending_at_an_items_start_takes_no_marker() {
    let item = |lead: &str, text: &str| super::Drawn {
        lead: lead.into(),
        text: text.into(),
        layout: gpui::TextLayout::default(),
        links: Vec::new(),
    };
    let texts = [item("• ", "first"), item("• ", "second")];
    let at = |text, at| super::Spot { text, at };
    let span = |from, to| super::copied(&texts, (from, to));
    assert_eq!(
        span(at(0, 5), at(1, 0)).as_deref(),
        Some("\n"),
        "the break alone"
    );
    assert_eq!(span(at(0, 0), at(1, 3)).as_deref(), Some("• first\n• sec"));
    assert_eq!(span(at(0, 2), at(1, 6)).as_deref(), Some("rst\n• second"));
}
