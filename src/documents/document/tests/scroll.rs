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
