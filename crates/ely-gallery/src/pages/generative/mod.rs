mod canvas;
mod data;
mod models;
mod panel;
mod results;
mod sound;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 25,
    slug: "generative",
    title: "Generative",
    summary: "Making pictures, sound and video with a model: the prompt and its settings, the queue and what comes back, the canvas, the models behind them, and the data and prompts they work from.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("gen-enhancer", 356.0, 124.0),
    Step::UpAt("gen-enhancer", 356.0, 124.0),
    Step::Wait(1400),
    Step::Shot("suggested"),
    Step::DownAt("gen-queue", 426.0, 280.0),
    Step::UpAt("gen-queue", 426.0, 280.0),
    Step::Wait(400),
    Step::Shot("retried"),
    Step::DownAt("gen-ab", 181.0, 147.0),
    Step::UpAt("gen-ab", 181.0, 147.0),
    Step::Wait(300),
    Step::Shot("verdict"),
    Step::DownAt("gen-canvas", 300.0, 70.0),
    Step::DragTo("gen-canvas", 390.0, 220.0),
    Step::UpAt("gen-canvas", 390.0, 220.0),
    Step::Wait(300),
    Step::Shot("masked"),
    Step::DownAt("gen-dataset", 14.0, 54.0),
    Step::UpAt("gen-dataset", 14.0, 54.0),
    Step::Wait(300),
    Step::Shot("sample"),
    Step::HoverAt("gen-embed", 49.6, 133.1),
    Step::Wait(200),
    Step::Shot("named"),
    Step::DownAt("gen-embed", 534.0, 160.0),
    Step::DragTo("gen-embed", 604.0, 140.0),
    Step::UpAt("gen-embed", 604.0, 140.0),
    Step::Wait(200),
    Step::Shot("turned"),
    Step::DownAt("gen-play", 313.0, 143.0),
    Step::UpAt("gen-play", 313.0, 143.0),
    Step::Type("dunes"),
    Step::DownAt("gen-play", 313.0, 179.0),
    Step::UpAt("gen-play", 313.0, 179.0),
    Step::Type("calm"),
    Step::DownAt("gen-play", 29.0, 302.0),
    Step::UpAt("gen-play", 29.0, 302.0),
    Step::Wait(1200),
    Step::Shot("played"),
    Step::DownAt("gen-versions", 46.0, 136.0),
    Step::UpAt("gen-versions", 46.0, 136.0),
    Step::Wait(200),
    Step::Shot("chosen"),
    Step::DownAt("gen-versions", 591.0, 28.0),
    Step::UpAt("gen-versions", 591.0, 28.0),
    Step::Wait(200),
    Step::Shot("restored"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(panel::panel(window, cx))
        .child(results::queue(window, cx))
        .child(results::results(window, cx))
        .child(results::variations(window, cx))
        .child(canvas::canvas(window, cx))
        .child(sound::audio(window, cx))
        .child(sound::voice(window, cx))
        .child(sound::video(window, cx))
        .child(models::cards(window, cx))
        .child(models::downloads_section(window, cx))
        .child(models::machine(window, cx))
        .child(models::tuning(window, cx))
        .child(data::dataset(cx))
        .child(data::embeddings(cx))
        .child(data::playground(window, cx))
        .child(data::versions(window, cx))
        .into_any_element()
}
