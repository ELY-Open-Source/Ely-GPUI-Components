mod messages;
mod states;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 10,
    slug: "feedback",
    title: "Feedback",
    summary: "What the app tells you: toasts, notifications, alerts and the states a view can be in.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Click("center-reset"),
    Step::Click("toast-saved"),
    Step::Wait(200),
    Step::Click("toast-error"),
    Step::Wait(200),
    Step::Click("toast-undo"),
    Step::Wait(800),
    Step::Shot("toasts"),
    Step::Wait(5600),
    Step::DownAt("center", 265.0, 20.0),
    Step::UpAt("center", 265.0, 20.0),
    Step::Wait(300),
    Step::Shot("center-read"),
    Step::DownAt("center", 350.0, 20.0),
    Step::UpAt("center", 350.0, 20.0),
    Step::Wait(700),
    Step::Shot("center-clear"),
    Step::Click("status"),
    Step::Wait(80),
    Step::Shot("status-rising"),
    Step::Click("boundary-break"),
    Step::Wait(700),
    Step::Shot("boundary"),
    Step::Click("boundary-break"),
    Step::Click("result-replay"),
    Step::Wait(120),
    Step::Shot("result-drawing"),
    Step::Wait(1000),
    Step::Shot("result"),
    Step::Click("saving-next"),
    Step::Click("sync-next"),
    Step::Click("connection-next"),
    Step::Wait(400),
    Step::Shot("statuses"),
    Step::Click("countdown-restart"),
    Step::Wait(1500),
    Step::Shot("clocks"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(messages::toasts(window, cx))
        .child(messages::center(window, cx))
        .child(messages::alerts(window, cx))
        .child(messages::banner(window, cx))
        .child(messages::callouts(cx))
        .child(messages::lines(window, cx))
        .child(messages::empties(cx))
        .child(states::errors(cx))
        .child(states::boundary(window, cx))
        .child(states::results(window, cx))
        .child(states::confirmations(window, cx))
        .child(states::statuses(window, cx))
        .child(states::clocks(window, cx))
        .into_any_element()
}
