use gpui::{AnyElement, App, Window, div, prelude::*};

mod boards;
mod graphs;
mod looks;
mod panels;
mod plane;
mod tools;

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 32,
    slug: "canvas",
    title: "Canvas & Design",
    summary: "An endless plane to lay work on, the tools and panels around it, and graphs of nodes.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("canvas-plane", 420.0, 300.0),
    Step::DragTo("canvas-plane", 300.0, 240.0),
    Step::UpAt("canvas-plane", 300.0, 240.0),
    Step::Wait(200),
    Step::Shot("panned"),
    Step::DownAt("canvas-tools", 150.0, 110.0),
    Step::DragTo("canvas-tools", 170.0, 112.0),
    Step::DragTo("canvas-tools", 187.0, 112.0),
    Step::Wait(200),
    Step::Shot("snapping"),
    Step::UpAt("canvas-tools", 187.0, 112.0),
    Step::Wait(200),
    Step::DownAt("canvas-palette", 83.0, 16.0),
    Step::UpAt("canvas-palette", 83.0, 16.0),
    Step::Wait(150),
    Step::DownAt("canvas-tools", 60.0, 230.0),
    Step::DragTo("canvas-tools", 200.0, 300.0),
    Step::UpAt("canvas-tools", 200.0, 300.0),
    Step::Wait(250),
    Step::Shot("drawn"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(plane::plane(window, cx))
        .child(tools::tools(window, cx))
        .child(panels::arrange(window, cx))
        .child(looks::styles(window, cx))
        .child(looks::rules(window, cx))
        .child(graphs::nodes(window, cx))
        .child(boards::flow(window, cx))
        .child(boards::mind(window, cx))
        .child(graphs::board(window, cx))
        .into_any_element()
}
