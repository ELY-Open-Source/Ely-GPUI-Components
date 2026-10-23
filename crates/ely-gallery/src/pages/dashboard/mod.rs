use gpui::{AnyElement, App, Window, div, prelude::*};

mod board;
mod monitor;

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 34,
    slug: "dashboard",
    title: "Dashboard & Monitoring",
    summary: "Widgets on a grid you arrange, what is firing and what is up, and the stream of what happens.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(board::grid(window, cx))
        .child(monitor::health(window, cx))
        .child(monitor::incidents(window, cx))
        .child(monitor::stream(window, cx))
        .into_any_element()
}
