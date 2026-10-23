mod files;
mod long;
mod rows;
mod trees;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 13,
    slug: "lists",
    title: "Lists & Trees",
    summary: "Rows to read, pick, reorder and swipe; lists long enough to need care.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("pick", 20.0, 52.0),
    Step::UpAt("pick", 20.0, 52.0),
    Step::Key("shift-down"),
    Step::Wait(200),
    Step::Shot("selected-range"),
    Step::Swipe("swipe-row", -120.0),
    Step::Wait(500),
    Step::Shot("swiped-open"),
    Step::DownAt("tree", 40.0, 102.0),
    Step::UpAt("tree", 40.0, 102.0),
    Step::Key("right"),
    Step::Wait(200),
    Step::Shot("tree-open"),
    Step::DownAt("tree", 60.0, 214.0),
    Step::DragTo("tree", 60.0, 46.0),
    Step::Wait(100),
    Step::Shot("tree-dragging"),
    Step::UpAt("tree", 60.0, 46.0),
    Step::Wait(200),
    Step::Shot("tree-moved"),
    Step::DownAt("outline", 40.0, 150.0),
    Step::UpAt("outline", 40.0, 150.0),
    Step::Wait(500),
    Step::Shot("outline-moved"),
    Step::DownAt("file-tree", 120.0, 22.0),
    Step::UpAt("file-tree", 120.0, 22.0),
    Step::Key("cmd-a"),
    Step::Type("side"),
    Step::Wait(300),
    Step::Shot("files-filtered"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(rows::items(cx))
        .child(rows::selectable(window, cx))
        .child(rows::sortable(window, cx))
        .child(rows::swipeable(window, cx))
        .child(long::virtual_list(cx))
        .child(long::infinite(window, cx))
        .child(long::grouped(cx))
        .child(trees::tree(window, cx))
        .child(trees::checkbox(window, cx))
        .child(trees::select(window, cx))
        .child(trees::outline(window, cx))
        .child(files::file_tree(window, cx))
        .child(files::directory(window, cx))
        .into_any_element()
}
