mod people;
mod sharing;
mod talk;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 22,
    slug: "collab",
    title: "Collaboration",
    summary: "People in the same document: who is here and where, comments and reactions, suggested changes, and sharing.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::HoverAt("collab-peers", 35.0, 13.0),
    Step::Wait(700),
    Step::Shot("presence"),
    Step::DownAt("collab-peers", 35.0, 13.0),
    Step::UpAt("collab-peers", 35.0, 13.0),
    Step::Wait(300),
    Step::Shot("follow"),
    Step::DownAt("collab-peers", 35.0, 13.0),
    Step::UpAt("collab-peers", 35.0, 13.0),
    Step::DownAt("collab-marker", 12.0, 12.0),
    Step::UpAt("collab-marker", 12.0, 12.0),
    Step::Wait(300),
    Step::Shot("bubble"),
    Step::Key("escape"),
    Step::DownAt("collab-sidebar", 238.0, 24.0),
    Step::UpAt("collab-sidebar", 238.0, 24.0),
    Step::Wait(300),
    Step::Shot("resolved"),
    Step::DownAt("collab-sidebar", 90.0, 24.0),
    Step::UpAt("collab-sidebar", 90.0, 24.0),
    Step::DownAt("collab-share", 30.0, 14.0),
    Step::UpAt("collab-share", 30.0, 14.0),
    Step::Wait(400),
    Step::Shot("share"),
    Step::Key("escape"),
    Step::Wait(300),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(people::presence(window, cx))
        .child(talk::comments(window, cx))
        .child(talk::annotations(window, cx))
        .child(talk::changes(window, cx))
        .child(sharing::sharing(window, cx))
        .child(sharing::activity(cx))
        .into_any_element()
}
