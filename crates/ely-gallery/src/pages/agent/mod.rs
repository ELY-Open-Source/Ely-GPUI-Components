mod changes;
mod control;
mod environment;
mod previews;
mod progress;
mod tools;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 24,
    slug: "agent",
    title: "Agent",
    summary: "An assistant at work: the tools it calls, what it asks before it acts, how far it has come, what it would change, what it sees and what it makes.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("agent-call", 60.0, 18.0),
    Step::UpAt("agent-call", 60.0, 18.0),
    Step::Wait(300),
    Step::Shot("tool-call"),
    Step::Click("agent-approve-open"),
    Step::Wait(300),
    Step::Shot("approval"),
    Step::Key("escape"),
    Step::Wait(200),
    Step::DownAt("agent-call", 60.0, 18.0),
    Step::UpAt("agent-call", 60.0, 18.0),
    Step::Wait(200),
    Step::Click("agent-advance"),
    Step::Wait(300),
    Step::Shot("plan"),
    Step::DownAt("agent-changes", 30.0, 51.0),
    Step::UpAt("agent-changes", 30.0, 51.0),
    Step::Wait(300),
    Step::Shot("changes"),
    Step::DownAt("agent-changes", 30.0, 51.0),
    Step::UpAt("agent-changes", 30.0, 51.0),
    Step::Wait(200),
    Step::Click("agent-pointer-next"),
    Step::Wait(400),
    Step::Shot("pointer"),
    Step::Click("agent-stop"),
    Step::Wait(300),
    Step::Shot("interrupted"),
    Step::DownAt("agent-ask", 57.0, 79.0),
    Step::UpAt("agent-ask", 57.0, 79.0),
    Step::Wait(300),
    Step::Shot("answered"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(tools::calls(window, cx))
        .child(tools::approval(window, cx))
        .child(progress::steps(window, cx))
        .child(progress::agents(window, cx))
        .child(changes::changes_section(window, cx))
        .child(previews::watching(window, cx))
        .child(previews::artifacts(window, cx))
        .child(environment::sandbox_section(window, cx))
        .child(environment::connectors_section(window, cx))
        .child(environment::memory_section(window, cx))
        .child(control::interrupt_section(window, cx))
        .child(control::cost_section(window, cx))
        .child(control::trace_section(window, cx))
        .into_any_element()
}
