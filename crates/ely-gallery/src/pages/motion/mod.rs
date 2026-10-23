mod effects;
mod loading;
mod moves;
mod progress;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 11,
    slug: "motion",
    title: "Loading & Motion",
    summary: "Waiting, shown honestly, and the motion everything else moves with.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Click("save"),
    Step::Wait(250),
    Step::Shot("loading-button"),
    Step::Click("load-more"),
    Step::Wait(300),
    Step::Shot("load-more-busy"),
    Step::Wait(1200),
    Step::Shot("load-more-done"),
    Step::Click("shimmer-load"),
    Step::Wait(500),
    Step::Shot("shimmer-loaded"),
    Step::Click("shimmer-load"),
    Step::Click("progress-step"),
    Step::Wait(400),
    Step::Shot("progress"),
    Step::Click("overlay-reload"),
    Step::Wait(300),
    Step::Shot("overlay"),
    Step::Wait(1500),
    Step::Click("refresh-run"),
    Step::Wait(300),
    Step::Shot("refreshing"),
    Step::Click("uploads-tick"),
    Step::Wait(400),
    Step::Shot("uploads"),
    Step::Click("transition-toggle"),
    Step::Wait(80),
    Step::Shot("transition-leaving"),
    Step::Click("transition-toggle"),
    Step::Click("presence-add"),
    Step::Wait(80),
    Step::Shot("presence-arriving"),
    Step::Click("flip-shuffle"),
    Step::Wait(100),
    Step::Shot("flip-gliding"),
    Step::Click("ripple"),
    Step::Wait(120),
    Step::Shot("ripple"),
    Step::Click("flash-tick"),
    Step::Click("shake-try"),
    Step::Wait(100),
    Step::Shot("cues"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(loading::spinners(cx))
        .child(loading::skeletons(cx))
        .child(loading::shimmer(window, cx))
        .child(loading::buttons(window, cx))
        .child(loading::more(window, cx))
        .child(progress::bars(window, cx))
        .child(progress::overlay(window, cx))
        .child(progress::lazy(cx))
        .child(progress::suspense(window, cx))
        .child(progress::refresh(window, cx))
        .child(progress::uploads(window, cx))
        .child(moves::transitions(window, cx))
        .child(moves::stagger(window, cx))
        .child(moves::presence(window, cx))
        .child(moves::flip(window, cx))
        .child(moves::reorder(window, cx))
        .child(effects::marquee(cx))
        .child(effects::ripple(window, cx))
        .child(effects::light(cx))
        .child(effects::cues(window, cx))
        .child(effects::backdrops(cx))
        .into_any_element()
}
