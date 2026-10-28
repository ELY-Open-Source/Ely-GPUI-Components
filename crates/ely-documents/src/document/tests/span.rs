//! Selecting across blocks in the visual view (Ely R14.29).

use std::{cell::RefCell, ops::Range, rc::Rc};

use gpui::{
    Bounds, Entity, Modifiers, MouseButton, Pixels, TestAppContext, VisualTestContext, point, px,
};

use super::{open, opened, text};
use crate::{DocumentEvent, InlineFormat, MarkdownDocument, MarkdownMode, document::visual};

const TEXT: &str = "First line.\n\nSecond one.\n\nThird here.\n";

/// The open field's range, selection and caret.
fn field(
    document: &Entity<MarkdownDocument>,
    cx: &mut VisualTestContext,
) -> (Range<usize>, Range<usize>, usize) {
    document.read_with(cx, |document, cx| {
        let open = document.open.as_ref().expect("a field is open");
        let input = open.input.read(cx);
        (open.range.clone(), input.selection(), input.cursor())
    })
}

fn drawn(name: &str, cx: &mut VisualTestContext) -> Option<Bounds<Pixels>> {
    cx.debug_bounds(Box::leak(name.to_string().into_boxed_str()))
}

/// Holds the left button on block `from`, then moves to block `to`.
fn drag(from: usize, to: usize, cx: &mut VisualTestContext) {
    let start = drawn(&format!("markdown-block-{from}"), cx).expect("drawn");
    let at = point(start.left() + px(4.0), start.top() + px(4.0));
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let target = drawn(&format!("markdown-block-{to}"), cx).expect("drawn");
    cx.simulate_mouse_move(target.center(), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

/// Lifts the button on the field's first or last line, past its text.
fn lift(last: bool, cx: &mut VisualTestContext) {
    let field = drawn("markdown-open", cx).expect("the field is drawn");
    let at = match last {
        true => point(field.right() - px(2.0), field.bottom() - px(8.0)),
        false => point(field.left() + px(2.0), field.top() + px(8.0)),
    };
    cx.simulate_mouse_move(at, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

#[gpui::test]
fn a_drag_down_opens_the_blocks_it_crosses_as_one_selection(cx: &mut TestAppContext) {
    let (document, cx) = open(TEXT, MarkdownMode::Visual, cx);
    let changes = Rc::new(RefCell::new(0));
    let counted = changes.clone();
    cx.update(|_, cx| {
        cx.subscribe(&document, move |_, event: &DocumentEvent, _| {
            if *event == DocumentEvent::Changed {
                *counted.borrow_mut() += 1;
            }
        })
        .detach()
    });
    drag(0, 2, cx);
    let (range, ..) = field(&document, cx);
    assert_eq!(range, 0..TEXT.len() - 1, "three blocks open as one");
    assert!(
        drawn("markdown-block-1", cx).is_none(),
        "a block in the field is drawn once"
    );
    lift(true, cx);
    let end = "First line.\n\nSecond one.\n\nThird here.".len();
    assert_eq!(
        field(&document, cx).1,
        11..end,
        "from the press to the third's end"
    );
    assert_eq!(*changes.borrow(), 0, "a selection changes no text");
    assert!(document.read_with(cx, |document, _| document.drag.is_none()));
    cx.simulate_keystrokes("secondary-c");
    let copied = cx.read_from_clipboard().and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some("\n\nSecond one.\n\nThird here."));
    cx.simulate_keystrokes("backspace escape");
    assert_eq!(text(&document, cx), "First line.\n");
    cx.dispatch_action(visual::Undo);
    assert_eq!(text(&document, cx), TEXT, "one step undoes it");
}

#[gpui::test]
fn a_drag_up_keeps_its_caret_at_the_top(cx: &mut TestAppContext) {
    let (document, cx) = open(TEXT, MarkdownMode::Visual, cx);
    drag(2, 0, cx);
    lift(false, cx);
    let (range, selection, caret) = field(&document, cx);
    assert_eq!(range, 0..TEXT.len() - 1);
    assert_eq!((selection.end, caret), (TEXT.len() - 1, selection.start));
    assert!(caret < 11, "the head is in the first block");
    cx.simulate_input("X");
    cx.simulate_keystrokes("escape");
    assert!(
        text(&document, cx).ends_with("X\n"),
        "{}",
        text(&document, cx)
    );
}

#[gpui::test]
fn shift_and_arrows_select_on_past_a_blocks_edge(cx: &mut TestAppContext) {
    let (document, cx) = open(TEXT, MarkdownMode::Visual, cx);
    super::press(0, Modifiers::none(), cx);
    cx.simulate_keystrokes("shift-down");
    assert_eq!(
        field(&document, cx),
        (0..24, 11..24, 24),
        "the column holds"
    );
    assert_eq!(opened(&document, cx), "First line.\n\nSecond one.");
    cx.simulate_keystrokes("shift-down");
    let (range, selection, caret) = field(&document, cx);
    assert_eq!((range, selection, caret), (0..37, 11..37, 37));
    cx.simulate_keystrokes("shift-up");
    let (range, _, caret) = field(&document, cx);
    assert_eq!(range, 0..37, "going back keeps the field");
    assert!(caret < 37);
    cx.simulate_keystrokes("down down down");
    assert_eq!(
        field(&document, cx).0,
        0..37,
        "plain down stays in its last block"
    );
}

#[gpui::test]
fn shift_up_from_a_blocks_top_reaches_the_one_above(cx: &mut TestAppContext) {
    let (document, cx) = open(TEXT, MarkdownMode::Visual, cx);
    super::press(1, Modifiers::none(), cx);
    cx.simulate_keystrokes("shift-up");
    assert_eq!(field(&document, cx), (0..24, 11..24, 11));
    cx.simulate_keystrokes("shift-up");
    assert_eq!(
        field(&document, cx),
        (0..24, 0..24, 0),
        "the first block ends it"
    );
}

#[gpui::test]
fn a_drag_past_the_view_scrolls_and_selects_on(cx: &mut TestAppContext) {
    let long: String = (0..80).map(|n| format!("Line {n}.\n\n")).collect();
    let (document, cx) = open(&long, MarkdownMode::Visual, cx);
    let start = drawn("markdown-block-0", cx).expect("drawn");
    let at = point(start.left() + px(4.0), start.top() + px(4.0));
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let view = document.read_with(cx, |document, _| document.list.viewport_bounds());
    let below = point(view.center().x, view.bottom() + px(40.0));
    for _ in 0..3 {
        cx.simulate_mouse_move(below, MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
    }
    let top = document.read_with(cx, |document, _| document.list.logical_scroll_top());
    assert!(top.item_ix > 0 || top.offset_in_item > px(0.0), "{top:?}");
    let (range, ..) = field(&document, cx);
    assert!(range.end > "Line 0.".len(), "{range:?}");
    cx.simulate_mouse_up(below, MouseButton::Left, Modifiers::none());
}

#[gpui::test]
fn a_format_takes_each_selected_block(cx: &mut TestAppContext) {
    let (document, cx) = open(TEXT, MarkdownMode::Visual, cx);
    super::press(1, Modifiers::none(), cx);
    cx.simulate_keystrokes("shift-up shift-up");
    document
        .update(cx, |document, cx| document.format(InlineFormat::Strong, cx))
        .expect("taken");
    cx.run_until_parked();
    cx.simulate_keystrokes("escape");
    assert_eq!(
        text(&document, cx),
        "**First line.**\n\n**Second one.**\n\nThird here.\n"
    );
}
