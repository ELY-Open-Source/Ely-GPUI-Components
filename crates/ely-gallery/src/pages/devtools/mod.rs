use gpui::{AnyElement, App, Window, div, prelude::*};

mod apis;
mod data;
mod db;
mod formats;
mod tools;

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 33,
    slug: "devtools",
    title: "DB & Dev Tools",
    summary: "Connections, schemas and queries, data to read, requests to send, and the machines that run them.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(db::connections(window, cx))
        .child(db::schema(window, cx))
        .child(data::queries(window, cx))
        .child(db::structure(window, cx))
        .child(data::redis(window, cx))
        .child(formats::documents(window, cx))
        .child(apis::requests(window, cx))
        .child(apis::socket(window, cx))
        .child(apis::graphql(window, cx))
        .child(tools::text(window, cx))
        .child(tools::machines(window, cx))
        .into_any_element()
}
