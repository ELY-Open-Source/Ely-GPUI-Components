mod cartesian;
mod flows;
mod spread;
mod wholes;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 15,
    slug: "charts",
    title: "Charts",
    summary: "Lines, bars, parts of a whole, spreads, flows and schedules on quiet axes, with legends, tooltips, zoom and export.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::HoverAt("revenue", 400.0, 140.0),
    Step::Wait(300),
    Step::Shot("crosshair"),
    Step::DownAt("revenue", 150.0, 140.0),
    Step::DragTo("revenue", 520.0, 140.0),
    Step::UpAt("revenue", 520.0, 140.0),
    Step::Wait(300),
    Step::HoverAt("revenue", 360.0, 120.0),
    Step::Wait(300),
    Step::Shot("zoomed"),
    Step::DownAt("revenue", 100.0, 8.0),
    Step::UpAt("revenue", 100.0, 8.0),
    Step::Wait(300),
    Step::Shot("hidden"),
    Step::HoverAt("bars", 160.0, 150.0),
    Step::Wait(300),
    Step::Shot("band"),
    Step::HoverAt("pie", 236.0, 95.0),
    Step::Wait(300),
    Step::Shot("slice"),
    Step::HoverAt("donut", 109.0, 223.0),
    Step::Wait(300),
    Step::Shot("donut"),
    Step::HoverAt("bubbles", 291.0, 232.0),
    Step::Wait(300),
    Step::Shot("bubble"),
    Step::HoverAt("radar", 309.0, 112.0),
    Step::Wait(300),
    Step::Shot("spoke"),
    Step::HoverAt("treemap", 60.0, 60.0),
    Step::Wait(300),
    Step::Shot("tile"),
    Step::HoverAt("sankey", 490.0, 177.0),
    Step::Wait(300),
    Step::Shot("ribbon"),
    Step::HoverAt("gantt", 400.0, 98.0),
    Step::Wait(300),
    Step::Shot("task"),
    Step::HoverAt("chord", 246.0, 58.0),
    Step::Wait(300),
    Step::Shot("chord"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(cartesian::lines(window, cx))
        .child(cartesian::areas_and_bars(cx))
        .child(wholes::pies(cx))
        .child(cartesian::scatter(cx))
        .child(wholes::radar(cx))
        .child(wholes::heatmaps(cx))
        .child(wholes::parts(cx))
        .child(flows::flows(cx))
        .child(spread::waterfall(cx))
        .child(spread::distributions(cx))
        .child(flows::schedule(cx))
        .child(flows::relations(cx))
        .child(flows::targets(cx))
        .child(cartesian::realtime(window, cx))
        .into_any_element()
}
