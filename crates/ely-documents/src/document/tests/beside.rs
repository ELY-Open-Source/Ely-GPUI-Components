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
    let at = text(document, cx).find(find).expect("a match");
    let edits = vec![(at..at + find.len(), with.to_string())];
    assert!(document.update(cx, |document, cx| document.replace(edits, cx)));
    cx.run_until_parked();
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
