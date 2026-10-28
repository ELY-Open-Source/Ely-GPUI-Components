//! Selecting across blocks in the visual view (Ely R14.29).

use std::{cell::RefCell, ops::Range, rc::Rc, time::Duration};

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

/// Presses block `from`, then moves to block `to`.
fn drag(from: usize, to: usize, cx: &mut VisualTestContext) {
    let start = drawn(&format!("markdown-block-{from}"), cx).expect("drawn");
    let at = point(start.left() + px(4.0), start.top() + px(4.0));
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let target = drawn(&format!("markdown-block-{to}"), cx).expect("drawn");
    cx.simulate_mouse_move(target.center(), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

/// Lifts on the field's first or last line, past it.
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
    super::frames(cx);
    let (range, selection, caret) = field(&document, cx);
    assert_eq!((range, selection.start), (0..24, 11));
    assert!((20..=24).contains(&caret), "near the line's end: {caret}");
    assert_eq!(opened(&document, cx), "First line.\n\nSecond one.");
    cx.simulate_keystrokes("shift-down");
    super::frames(cx);
    let (range, selection, caret) = field(&document, cx);
    assert_eq!((range, selection.start), (0..37, 11));
    assert!((33..=37).contains(&caret), "near the line's end: {caret}");
    cx.simulate_keystrokes("shift-up");
    let (range, _, caret) = field(&document, cx);
    assert_eq!(range, 0..37, "going back keeps the field");
    assert!(caret < 33);
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
    super::frames(cx);
    let (range, selection, caret) = field(&document, cx);
    assert_eq!((range, selection.end, caret), (0..24, 24, selection.start));
    assert!((7..=11).contains(&caret), "near the line's end: {caret}");
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
    let below = point(view.center().x, view.bottom() + px(20.0));
    cx.simulate_mouse_move(below, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let scrolled = |cx: &mut VisualTestContext| {
        let top = document.read_with(cx, |document, _| document.list.logical_scroll_top());
        let (_, _, head) = field(&document, cx);
        (top.item_ix, top.offset_in_item, head)
    };
    let tick = |cx: &mut VisualTestContext| {
        cx.executor().advance_clock(Duration::from_millis(16));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    };
    tick(cx);
    let first = scrolled(cx);
    // The pointer holds still past the edge.
    for _ in 0..30 {
        tick(cx);
    }
    let held = scrolled(cx);
    assert!(
        (held.0, held.1) > (first.0, first.1),
        "{first:?} then {held:?}"
    );
    assert!(
        held.2 > first.2,
        "the head follows what scrolls in: {first:?} {held:?}"
    );
    cx.simulate_mouse_up(below, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.run_until_parked();
    assert_eq!(scrolled(cx), held, "a release stops it");
}

/// Repeats `word` `n` times, a line too long to fit.
fn wrapped(word: &str, n: usize) -> String {
    vec![word; n].join(" ")
}

/// Shift past a wrapped block's edge keeps the head's x.
fn aims_at_the_old_x(down: bool, cx: &mut TestAppContext) {
    let first = wrapped("one", 120);
    let source = format!("{first}\n\n{}\n", wrapped("two", 160));
    let (document, cx) = open(&source, MarkdownMode::Visual, cx);
    super::press(if down { 0 } else { 1 }, Modifiers::none(), cx);
    let opened = |cx: &mut VisualTestContext| {
        document.read_with(cx, |document, _| {
            document.open.as_ref().expect("open").input.clone()
        })
    };
    let x = opened(cx).update(cx, |input, cx| {
        let end = input.text().len();
        assert!(input.bounds_for_range(0..end).len() > 1, "the block wraps");
        let line = input
            .bounds_for(if down { end } else { 0 })
            .expect("laid out");
        let left = input.bounds().expect("laid out").left();
        let at = input.offset_for_point(point(left + px(300.0), line.center().y));
        input.select(at..at, cx);
        input.bounds_for(at).expect("laid out").left()
    });
    cx.simulate_keystrokes(if down { "shift-down" } else { "shift-up" });
    super::frames(cx);
    let (_, _, caret) = field(&document, cx);
    let edge = if down { first.len() + 2 } else { first.len() };
    let want = opened(cx).read_with(cx, |input, _| {
        let line = input.bounds_for(edge).expect("laid out");
        input.offset_for_point(point(x, line.center().y))
    });
    assert_eq!(caret, want, "on the other block's edge line, at the old x");
    assert!(
        want.abs_diff(edge) > 20,
        "mid-line, not at its end: {want} {edge}"
    );
}

#[gpui::test]
fn shift_down_lands_on_the_next_blocks_first_shown_line(cx: &mut TestAppContext) {
    aims_at_the_old_x(true, cx);
}

#[gpui::test]
fn shift_up_lands_on_the_last_shown_line_above(cx: &mut TestAppContext) {
    aims_at_the_old_x(false, cx);
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

#[gpui::test]
fn shift_up_from_a_new_end_takes_the_block_above(cx: &mut TestAppContext) {
    let (document, cx) = open("alpha\n", MarkdownMode::Visual, cx);
    document.update_in(cx, |document, window, cx| {
        document.open(None, crate::document::edit::Landing::End, window, cx)
    });
    cx.simulate_input("beta");
    cx.simulate_keystrokes("shift-up");
    super::frames(cx);
    let (range, selection, caret) = field(&document, cx);
    assert_eq!(range, 0..6, "the block above and the new end");
    assert_eq!(opened(&document, cx), "alpha\n\nbeta");
    assert_eq!((selection.end, caret), (11, selection.start));
    assert!(caret < 5, "the head is in the block above: {caret}");
    cx.simulate_keystrokes("escape");
    assert_eq!(text(&document, cx), "alpha\n\nbeta");
    cx.dispatch_action(visual::Undo);
    assert_eq!(text(&document, cx), "alpha\n", "one step undoes it");
}

/// A press and release at `at`, drawn between.
fn click(at: gpui::Point<Pixels>, cx: &mut VisualTestContext) {
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::none());
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
    cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

#[gpui::test]
fn clicks_in_a_new_end_select_nothing_above(cx: &mut TestAppContext) {
    let (document, cx) = open("alpha\n", MarkdownMode::Visual, cx);
    let more = document.read_with(cx, |document, _| {
        document.list.bounds_for_item(document.parsed.blocks.len())
    });
    click(more.expect("the end is drawn").center(), cx);
    cx.simulate_input("beta");
    assert_eq!(
        text(&document, cx),
        "alpha\n\nbeta",
        "the click took no block"
    );
    let input = document.read_with(cx, |document, _| {
        document.open.as_ref().expect("open").input.clone()
    });
    let at = input.read_with(cx, |input, _| input.bounds_for(2).expect("laid out"));
    click(at.center(), cx);
    cx.simulate_input("X");
    cx.simulate_keystrokes("escape");
    assert_eq!(text(&document, cx), "alpha\n\nbeXta");
}

#[gpui::test]
fn a_held_drag_ends_with_its_field(cx: &mut TestAppContext) {
    let long: String = (0..80).map(|n| format!("Line {n}.\n\n")).collect();
    let (document, cx) = open(&long, MarkdownMode::Visual, cx);
    let start = drawn("markdown-block-0", cx).expect("drawn");
    let at = point(start.left() + px(4.0), start.top() + px(4.0));
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let view = document.read_with(cx, |document, _| document.list.viewport_bounds());
    let below = point(view.center().x, view.bottom() + px(20.0));
    cx.simulate_mouse_move(below, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    document.update_in(cx, |document, window, cx| {
        document.set_mode(MarkdownMode::Source, window, cx)
    });
    cx.simulate_mouse_up(below, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let notified = Rc::new(RefCell::new(0));
    let counted = notified.clone();
    cx.update(|_, cx| {
        cx.observe(&document, move |_, _| *counted.borrow_mut() += 1)
            .detach()
    });
    let top = document.read_with(cx, |document, _| document.list.logical_scroll_top());
    for _ in 0..10 {
        cx.executor().advance_clock(Duration::from_millis(16));
        cx.run_until_parked();
    }
    assert!(document.read_with(cx, |document, _| document.drag.is_none()));
    assert_eq!(*notified.borrow(), 0, "idle once the field is gone");
    let still = document.read_with(cx, |document, _| document.list.logical_scroll_top());
    assert_eq!(
        (still.item_ix, still.offset_in_item),
        (top.item_ix, top.offset_in_item)
    );
}

#[gpui::test]
fn typing_or_a_move_before_the_aim_lands_keeps_its_place(cx: &mut TestAppContext) {
    let (document, cx) = open(TEXT, MarkdownMode::Visual, cx);
    super::press(0, Modifiers::none(), cx);
    cx.simulate_keystrokes("shift-down");
    cx.simulate_input("X");
    super::frames(cx);
    cx.simulate_input("Y");
    assert_eq!(opened(&document, cx), "First line.XYSecond one.");
    cx.simulate_keystrokes("escape");
    super::press(0, Modifiers::none(), cx);
    cx.simulate_keystrokes("shift-down left");
    super::frames(cx);
    let (_, selection, _) = field(&document, cx);
    assert!(selection.is_empty(), "the move stands: {selection:?}");
}
