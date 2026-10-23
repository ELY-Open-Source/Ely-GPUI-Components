mod assets;
mod book;
mod markets;
mod overview;
mod trading;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 16,
    slug: "finance",
    title: "Finance",
    summary: "Market charts with the studies traders read, books, quotes, order entry and portfolios.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::HoverAt("market", 520.0, 160.0),
    Step::Wait(300),
    Step::Shot("crosshair"),
    Step::DownAt("drawings", 240.0, 210.0),
    Step::DragTo("drawings", 620.0, 90.0),
    Step::UpAt("drawings", 620.0, 90.0),
    Step::Wait(300),
    Step::Shot("drawn"),
    Step::HoverAt("depth", 300.0, 150.0),
    Step::Wait(300),
    Step::Shot("depth"),
    Step::HoverAt("payoff", 560.0, 120.0),
    Step::Wait(300),
    Step::Shot("payoff"),
    Step::HoverAt("heatmap", 150.0, 150.0),
    Step::Wait(300),
    Step::Shot("heatmap"),
    Step::DownAt("ticket", 170.0, 256.0),
    Step::UpAt("ticket", 170.0, 256.0),
    Step::Wait(400),
    Step::Shot("confirm"),
    Step::Key("escape"),
    Step::Wait(300),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(markets::market(window, cx))
        .child(markets::indicators(window, cx))
        .child(markets::drawings(window, cx))
        .child(markets::depth(cx))
        .child(markets::profile(cx))
        .child(markets::bricks(cx))
        .child(markets::together(window, cx))
        .child(book::quotes(cx))
        .child(book::book(cx))
        .child(book::tape(window, cx))
        .child(trading::ticket(window, cx))
        .child(trading::blotter(cx))
        .child(trading::risk(window, cx))
        .child(trading::options(window, cx))
        .child(overview::watch(window, cx))
        .child(overview::heatmap(cx))
        .child(overview::screener(window, cx))
        .child(overview::overview(cx))
        .child(overview::calendars(window, cx))
        .child(assets::portfolio(cx))
        .child(assets::statements(cx))
        .child(assets::money(window, cx))
        .into_any_element()
}
