//! A long document's bar shows its place and moves it.

use gpui::{Modifiers, MouseButton, ScrollDelta, ScrollWheelEvent, TestAppContext, point, px};

use super::{MarkdownMode, frames, open};

fn long() -> String {
    (0..300).map(|n| format!("Paragraph {n}.\n\n")).collect()
}

fn the_thumb_follows_and_drags(mode: MarkdownMode, cx: &mut TestAppContext) {
    let (document, cx) = open(&long(), mode, cx);
    frames(cx);
    let view = cx.update(|window, _| window.viewport_size());
    let before = cx.debug_bounds("scrollbar-thumb").expect("a thumb");
    assert!(
        before.size.height < view.height / 2.0,
        "{before:?} in {view:?}"
    );
    cx.simulate_event(ScrollWheelEvent {
        position: point(view.width / 2.0, view.height / 2.0),
        delta: ScrollDelta::Pixels(point(px(0.), px(-800.))),
        ..Default::default()
    });
    frames(cx);
    let wheeled = cx.debug_bounds("scrollbar-thumb").expect("a thumb");
    assert!(wheeled.top() > before.top(), "{wheeled:?} after {before:?}");
    let top = |cx: &mut gpui::VisualTestContext| {
        document.read_with(cx, |document, cx| match mode {
            MarkdownMode::Source => {
                document
                    .editor
                    .read(cx)
                    .scroll
                    .0
                    .borrow()
                    .base_handle
                    .offset()
                    .y
            }
            _ => document.list.scroll_px_offset_for_scrollbar().y,
        })
    };
    let at = top(cx);
    let grip = wheeled.center();
    cx.simulate_mouse_down(grip, MouseButton::Left, Modifiers::none());
    // The first move starts the drag, the next anchor it.
    for down in [10., 20., 140.] {
        let to = grip + point(px(0.), px(down));
        cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
    }
    let to = grip + point(px(0.), px(140.));
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    frames(cx);
    assert!(
        top(cx) < at - px(400.),
        "dragging the thumb down scrolls: {:?} then {at:?}",
        top(cx)
    );
}

#[gpui::test]
fn visual_has_a_bar(cx: &mut TestAppContext) {
    the_thumb_follows_and_drags(MarkdownMode::Visual, cx);
}

#[gpui::test]
fn source_has_a_bar(cx: &mut TestAppContext) {
    the_thumb_follows_and_drags(MarkdownMode::Source, cx);
}

/// Presses the thumb and drags it `downs` below.
fn drag_thumb(downs: &[f32], cx: &mut gpui::VisualTestContext) {
    let grip = cx
        .debug_bounds("scrollbar-thumb")
        .expect("a thumb")
        .center();
    cx.simulate_mouse_move(grip, None, Modifiers::none());
    frames(cx);
    cx.simulate_mouse_down(grip, MouseButton::Left, Modifiers::none());
    for down in downs {
        let to = grip + point(px(0.), px(*down));
        cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
        frames(cx);
    }
    let last = downs.last().copied().unwrap_or(0.);
    cx.simulate_mouse_up(
        grip + point(px(0.), px(last)),
        MouseButton::Left,
        Modifiers::none(),
    );
    frames(cx);
}

#[gpui::test]
fn a_short_document_has_no_bar(cx: &mut TestAppContext) {
    let (_, cx) = open("Short paragraph.", MarkdownMode::Preview, cx);
    frames(cx);
    assert!(cx.debug_bounds("scrollbar-thumb").is_none());
}

#[gpui::test]
fn dragging_past_the_end_shows_a_tall_last_block_whole(cx: &mut TestAppContext) {
    let lines: String = (0..300).map(|n| format!("Line {n}\n")).collect();
    let text = format!("{}\n```text\n{lines}```\n", long());
    let (document, cx) = open(&text, MarkdownMode::Preview, cx);
    frames(cx);
    drag_thumb(&[10., 20., 60., 240., 900., 1600.], cx);
    document.read_with(cx, |document, _| {
        let list = &document.list;
        let tail = (0..list.item_count())
            .rev()
            .find_map(|ix| list.bounds_for_item(ix).filter(|b| b.size.height > px(0.)))
            .expect("the code block measured");
        let view = list.viewport_bounds();
        assert!(
            tail.bottom() <= view.bottom() + px(1.),
            "{tail:?} past {view:?}"
        );
    });
}

#[gpui::test]
fn the_thumb_leaves_the_source_selection(cx: &mut TestAppContext) {
    let (document, cx) = open(&long(), MarkdownMode::Source, cx);
    frames(cx);
    let before = document.read_with(cx, |document, cx| document.editor.read(cx).primary());
    drag_thumb(&[10., 20., 140.], cx);
    let after = document.read_with(cx, |document, cx| document.editor.read(cx).primary());
    assert_eq!(after, before);
}

#[gpui::test]
fn pressing_the_thumb_opens_no_block(cx: &mut TestAppContext) {
    let (document, cx) = open(&long(), MarkdownMode::Visual, cx);
    frames(cx);
    drag_thumb(&[], cx);
    document.read_with(cx, |document, _| assert!(document.open.is_none()));
}

/// How strongly the thumb under the pointer is painted.
fn hovered_alpha(cx: &mut gpui::VisualTestContext) -> f32 {
    let thumb = cx.debug_bounds("scrollbar-thumb").expect("a thumb");
    cx.simulate_mouse_move(thumb.center(), None, Modifiers::none());
    frames(cx);
    cx.update(|window, _| {
        let bounds = thumb.scale(window.scale_factor());
        let quads = window.painted_quads().into_iter();
        let ours = quads.filter(|quad| quad.bounds == bounds);
        let colors = ours.filter_map(|quad| quad.background.as_solid());
        colors.map(|color| color.a).fold(0.0, f32::max)
    })
}

#[gpui::test]
fn the_thumb_shows_under_the_pointer(cx: &mut TestAppContext) {
    let (_, cx) = open(&long(), MarkdownMode::Source, cx);
    frames(cx);
    let alpha = hovered_alpha(cx);
    assert!(alpha > 0.1, "alpha {alpha}");
}

#[gpui::test]
fn a_drag_up_from_a_tall_blocks_end_moves(cx: &mut TestAppContext) {
    let lines: String = (0..300).map(|n| format!("Line {n}\n")).collect();
    let (document, cx) = open(&format!("```text\n{lines}```\n"), MarkdownMode::Preview, cx);
    frames(cx);
    drag_thumb(&[10., 20., 60., 240., 900., 1600.], cx);
    let top = |cx: &mut gpui::VisualTestContext| {
        document.read_with(cx, |document, _| {
            document.list.scroll_px_offset_for_scrollbar().y
        })
    };
    let bottom = top(cx);
    drag_thumb(&[-10., -20., -120.], cx);
    assert!(top(cx) > bottom + px(200.), "{:?} from {bottom:?}", top(cx));
}
