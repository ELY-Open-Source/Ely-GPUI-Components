mod core;
mod features;
mod grids;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 14,
    slug: "tables",
    title: "Tables",
    summary: "Rows and columns that sort, filter, page and select, and stay quick at a hundred thousand rows.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("orders", 440.0, 16.0),
    Step::UpAt("orders", 440.0, 16.0),
    Step::Wait(200),
    Step::Shot("sorted"),
    Step::DownAt("orders", 18.0, 54.0),
    Step::UpAt("orders", 18.0, 54.0),
    Step::DownAt("orders", 18.0, 126.0),
    Step::UpAt("orders", 18.0, 126.0),
    Step::Wait(200),
    Step::Shot("selected"),
    Step::DownAt("orders-tools", 300.0, 14.0),
    Step::UpAt("orders-tools", 300.0, 14.0),
    Step::Wait(300),
    Step::Shot("builders"),
    Step::Hover("tasks"),
    Step::Wait(300),
    Step::DownAt("tasks", 14.0, 54.0),
    Step::UpAt("tasks", 14.0, 54.0),
    Step::Wait(300),
    Step::Shot("expanded"),
    Step::Hover("grouped"),
    Step::Wait(300),
    Step::DownAt("grouped", 60.0, 54.0),
    Step::UpAt("grouped", 60.0, 54.0),
    Step::Wait(300),
    Step::Shot("folded"),
    Step::Hover("sheet"),
    Step::Wait(300),
    Step::DownAt("sheet", 184.0, 294.0),
    Step::UpAt("sheet", 184.0, 294.0),
    Step::Type("=F8/4"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("sheet-formula"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(core::plain(cx))
        .child(core::orders(window, cx))
        .child(features::wide(cx))
        .child(features::rows(window, cx))
        .child(features::grouped(cx))
        .child(grids::data_grid(window, cx))
        .child(grids::spreadsheet(window, cx))
        .child(grids::tree_table(cx))
        .child(grids::pivot(cx))
        .child(core::long(window, cx))
        .child(core::heatmap(cx))
        .into_any_element()
}
