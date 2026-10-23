mod blocks;
mod cite;
mod compose;
mod controls;
mod history;
mod talk;
mod welcome;

use ely_gpui_component::feedback::{ToastViewport, Toaster};
use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::{step::Step, ui::keep};

pub const PAGE: Page = Page {
    number: 23,
    slug: "chat",
    title: "AI Chat",
    summary: "A conversation with an assistant: messages that stream in, code and media inside them, and what can be done with each.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("chat-thinking", 12.0, 10.0),
    Step::UpAt("chat-thinking", 12.0, 10.0),
    Step::Wait(300),
    Step::Shot("reasoning"),
    Step::DownAt("chat-thinking", 12.0, 10.0),
    Step::UpAt("chat-thinking", 12.0, 10.0),
    Step::DownAt("chat-edit-open", 20.0, 12.0),
    Step::UpAt("chat-edit-open", 20.0, 12.0),
    Step::Wait(300),
    Step::Shot("editing"),
    Step::Key("escape"),
    Step::Wait(200),
    Step::HoverAt("chat-cite-1", 8.0, 8.0),
    Step::Wait(700),
    Step::Shot("citation"),
    Step::DownAt("chat-sources", 20.0, 10.0),
    Step::UpAt("chat-sources", 20.0, 10.0),
    Step::Wait(300),
    Step::Shot("sources"),
    Step::DownAt("chat-templates", 30.0, 12.0),
    Step::UpAt("chat-templates", 30.0, 12.0),
    Step::Wait(200),
    Step::Key("down"),
    Step::Key("enter"),
    Step::Wait(200),
    Step::DownAt("chat-system", 160.0, 48.0),
    Step::UpAt("chat-system", 160.0, 48.0),
    Step::Key("cmd-a"),
    Step::Type("Answer in one line."),
    Step::Wait(200),
    Step::Shot("prompts"),
    Step::DownAt("chat-system", 548.0, 12.0),
    Step::UpAt("chat-system", 548.0, 12.0),
    Step::Wait(200),
    Step::Shot("reset"),
    Step::HoverAt("chat-history", 150.0, 124.0),
    Step::Wait(300),
    Step::Shot("history"),
    Step::Click("chat-export-open"),
    Step::Wait(300),
    Step::Shot("export"),
    Step::Key("escape"),
    Step::Wait(200),
    Step::Click("chat-follow-ups"),
    Step::Wait(300),
    Step::Shot("follow-up"),
    Step::DownAt("chat-sources", 20.0, 10.0),
    Step::UpAt("chat-sources", 20.0, 10.0),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    let draft = compose::draft(window, cx);
    let history = history::history(window, cx);
    let toaster = keep("chat-toaster", Toaster::default, window, cx);
    let opening = welcome::welcome_draft(window, cx);
    div()
        .child(talk::conversation(window, cx))
        .child(talk::streaming(window, cx))
        .child(blocks::code(window, cx))
        .child(blocks::rich(window, cx))
        .child(blocks::media(window, cx))
        .child(controls::actions(window, cx))
        .child(controls::status(window, cx))
        .child(controls::thinking(window, cx))
        .child(cite::citations(window, cx))
        .child(cite::search(window, cx))
        .child(compose::composer(&draft, window, cx))
        .child(compose::pickers(&draft, window, cx))
        .child(compose::tuning(window, cx))
        .child(history::conversations_section(&history, window, cx))
        .child(history::projects_section(window, cx))
        .child(history::shared_section(&history, &toaster, window, cx))
        .child(welcome::welcome_section(&opening, &toaster, window, cx))
        .child(welcome::follow_ups_section(window, cx))
        .child(welcome::library_section(&opening, window, cx))
        .child(ToastViewport::new("chat-toasts", &toaster))
        .into_any_element()
}
