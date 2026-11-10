//! A replacement beside the open block leaves it open.

use gpui::{Modifiers, TestAppContext};

use super::{open, press, text};
use crate::MarkdownMode;

const TEXT: &str = "one\n\ntwo\n\nthree\n";

/// Replaces the first `find` in the text as read.
fn replace(
    document: &gpui::Entity<super::MarkdownDocument>,
    find: &str,
    with: &str,
    cx: &mut gpui::VisualTestContext,
) {
    replace_at(document, find, with, cx);
    cx.run_until_parked();
}

/// The same, read before anything else runs.
fn replace_at(
    document: &gpui::Entity<super::MarkdownDocument>,
    find: &str,
    with: &str,
    cx: &mut gpui::VisualTestContext,
) {
    let at = text(document, cx).find(find).expect("a match");
    let edits = vec![(at..at + find.len(), with.to_string())];
    assert!(document.update(cx, |document, cx| document.replace(edits, cx)));
}

#[gpui::test]
fn typing_goes_on_after_a_replacement_before_or_after_the_block(cx: &mut TestAppContext) {
    let (document, cx) = open(TEXT, MarkdownMode::Visual, cx);
    press(1, Modifiers::none(), cx);
    cx.simulate_input("x");
    // The typing is not written back: later bytes differ.
    for (find, with, typed) in [("one", "first", "xy"), ("three", "3", "xyy")] {
        replace(&document, find, with, cx);
        cx.simulate_input("y");
        let read = text(&document, cx);
        assert!(
            read.contains(typed) && read.contains(with),
            "{find}: {read:?}"
        );
        assert!(
            document.read_with(cx, |document, _| document.open.is_some()),
            "{find}: the block closed"
        );
    }
    let read = text(&document, cx);
    assert!(
        read.starts_with("first\n\n") && read.ends_with("\n\n3\n"),
        "{read:?}"
    );
    // One step back: the last replacement alone.
    document.update(cx, |document, cx| {
        document.editor().update(cx, |editor, cx| editor.undo(cx))
    });
    cx.run_until_parked();
    assert!(text(&document, cx).ends_with("\n\nthree\n"));
}

#[gpui::test]
fn a_replacement_inside_the_open_block_writes_it_back(cx: &mut TestAppContext) {
    let (document, cx) = open(TEXT, MarkdownMode::Visual, cx);
    press(1, Modifiers::none(), cx);
    cx.simulate_input("x");
    replace(&document, "two", "2", cx);
    let read = text(&document, cx);
    assert!(
        read.contains('x') && read.contains('2') && !read.contains("two"),
        "{read:?}"
    );
    assert!(
        document.read_with(cx, |document, _| document.open.is_none()),
        "the block stayed open over a changed source"
    );
}

/// Whether a block is open.
fn is_open(
    document: &gpui::Entity<super::MarkdownDocument>,
    cx: &mut gpui::VisualTestContext,
) -> bool {
    document.read_with(cx, |document, _| document.open.is_some())
}

#[gpui::test]
fn a_replacement_that_joins_the_paragraph_before_writes_the_block_back(cx: &mut TestAppContext) {
    let (document, cx) = open(TEXT, MarkdownMode::Visual, cx);
    press(1, Modifiers::none(), cx);
    cx.simulate_input("x");
    // No edit touches "two", yet it stops being a block.
    replace_at(&document, "one\n\n", "one\n", cx);
    // At once: no field is open and undrawn meanwhile.
    assert!(
        !is_open(&document, cx),
        "a field stayed open over half a paragraph"
    );
    cx.run_until_parked();
    let read = text(&document, cx);
    assert!(
        read.starts_with("one\n") && read.contains('x') && read.ends_with("\n\nthree\n"),
        "{read:?}"
    );
}

#[gpui::test]
fn a_fence_opened_above_writes_the_block_back(cx: &mut TestAppContext) {
    let (document, cx) = open(TEXT, MarkdownMode::Visual, cx);
    press(1, Modifiers::none(), cx);
    cx.simulate_input("x");
    replace_at(&document, "one", "```", cx);
    assert!(
        !is_open(&document, cx),
        "a field stayed open inside a code block"
    );
    cx.run_until_parked();
    assert!(text(&document, cx).contains('x'));
}

const TABLE: &str = "| a | b |\n| --- | --- |\n| c | d |\n";

#[gpui::test]
fn a_row_added_above_the_open_cell_writes_the_cell_back(cx: &mut TestAppContext) {
    let (document, cx) = open(TABLE, MarkdownMode::Visual, cx);
    let cell = cx
        .debug_bounds("markdown-cell-0-1-0")
        .expect("the cell is drawn");
    cx.simulate_click(
        cell.origin + gpui::point(gpui::px(2.0), gpui::px(2.0)),
        Modifiers::none(),
    );
    cx.run_until_parked();
    assert!(is_open(&document, cx), "the cell opens");
    cx.simulate_input("x");
    let at = text(&document, cx).find("| c").expect("the row");
    let edits = vec![(at..at, "| new | row |\n".to_string())];
    assert!(document.update(cx, |document, cx| document.replace(edits, cx)));
    cx.run_until_parked();
    // The cell is row 2 now: the field would sit on row 1.
    assert!(!is_open(&document, cx), "the field stayed on another row");
    let read = text(&document, cx);
    let row = read.lines().nth(3).expect("the typed row");
    assert!(
        read.contains("| new | row |") && row.contains('x') && row.contains('d'),
        "{read:?}"
    );
}
