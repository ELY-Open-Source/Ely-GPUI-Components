use gpui::{AnyElement, App, Window, div, prelude::*};

mod keys;
mod pointer;

pub use keys::bind_keys;

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 38,
    slug: "interaction",
    title: "Interaction",
    summary: "Drag and drop, resizing and turning, band selection, presses and gestures, keys, scrolling and focus.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("interaction-tiles", 296.0, 6.0),
    Step::DragTo("interaction-tiles", 120.0, 120.0),
    Step::Wait(100),
    Step::Shot("band"),
    Step::UpAt("interaction-tiles", 120.0, 120.0),
    Step::DownAt("interaction-rotatable", 98.0, 22.0),
    Step::DragTo("interaction-rotatable", 116.0, 70.0),
    Step::UpAt("interaction-rotatable", 116.0, 70.0),
    Step::Wait(100),
    Step::Shot("turned"),
    Step::DownAt("interaction-keys", 12.0, 12.0),
    Step::UpAt("interaction-keys", 12.0, 12.0),
    Step::Key("cmd-k"),
    Step::Wait(100),
    Step::Shot("chord-waiting"),
    Step::Key("cmd-s"),
    Step::Wait(100),
    Step::Shot("chord-done"),
    Step::DownAt("interaction-fruit", 40.0, 16.0),
    Step::UpAt("interaction-fruit", 40.0, 16.0),
    Step::Type("bl"),
    Step::Wait(100),
    Step::Shot("type-ahead"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(pointer::drag_and_drop(window, cx))
        .child(pointer::shapes(window, cx))
        .child(keys::keys(window, cx))
        .child(keys::undo(window, cx))
        .child(keys::scrolling(cx))
        .child(keys::focus(window, cx))
        .into_any_element()
}
