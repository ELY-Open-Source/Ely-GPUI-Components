mod diffs;
mod people;
mod work;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 19,
    slug: "git",
    title: "Git",
    summary: "Diffs and merges, changes and commits, the commit graph, branches, blame and review.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("git-diff", 800.0, 21.0),
    Step::UpAt("git-diff", 800.0, 21.0),
    Step::Wait(300),
    Step::Shot("split"),
    Step::DownAt("git-commit", 40.0, 20.0),
    Step::UpAt("git-commit", 40.0, 20.0),
    Step::Wait(200),
    Step::Key("cmd-a"),
    Step::Type("Blend every accent toward white by lift, clamped in gamma"),
    Step::Wait(300),
    Step::Shot("subject"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(diffs::viewer(window, cx))
        .child(diffs::merges(window, cx))
        .child(work::changes(window, cx))
        .child(work::commits(window, cx))
        .child(work::refs(window, cx))
        .child(people::blame(window, cx))
        .child(people::review(window, cx))
        .into_any_element()
}
