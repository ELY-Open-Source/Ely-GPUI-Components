mod code;
mod intel;
mod panels;
mod search;
mod settings;
mod start;
mod status;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 17,
    slug: "editor",
    title: "Editor",
    summary: "A code editor with the gutter, folds and cursors it needs, the help a language server gives, search, the panels around it and its status row.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("editor-main", 118.0, 104.0),
    Step::UpAt("editor-main", 118.0, 104.0),
    Step::Key("cmd-d"),
    Step::Key("cmd-d"),
    Step::Wait(300),
    Step::Shot("cursors"),
    Step::Key("escape"),
    Step::Key("cmd-/"),
    Step::Wait(300),
    Step::Shot("commented"),
    Step::Key("cmd-z"),
    Step::Wait(300),
    Step::DownAt("status", 249.0, 11.0),
    Step::UpAt("status", 249.0, 11.0),
    Step::Wait(900),
    Step::Shot("status"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(code::editor(window, cx))
        .child(code::looks(window, cx))
        .child(code::changes(window, cx))
        .child(intel::hints(window, cx))
        .child(intel::navigate(window, cx))
        .child(search::find(window, cx))
        .child(search::panel(window, cx))
        .child(panels::problems_and_output(window, cx))
        .child(panels::console_and_extensions(window, cx))
        .child(settings::settings_and_keys(window, cx))
        .child(start::start(window, cx))
        .child(status::status(window, cx))
        .into_any_element()
}
