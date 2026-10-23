mod anchored;
mod dialogs;
mod guides;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 9,
    slug: "overlays",
    title: "Overlays",
    summary: "Panels over the page: anchored to a control, over a scrim, or over everything.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Click("share"),
    Step::Wait(400),
    Step::Shot("popover"),
    Step::Key("escape"),
    Step::Hover("ada"),
    Step::Wait(900),
    Step::Shot("hovercard"),
    Step::Hover("share"),
    Step::Wait(400),
    Step::Click("edit-profile"),
    Step::Wait(500),
    Step::Shot("dialog"),
    Step::Key("escape"),
    Step::Click("alert"),
    Step::Wait(500),
    Step::Shot("alert"),
    Step::Key("escape"),
    Step::Click("confirm"),
    Step::Wait(500),
    Step::Shot("confirm"),
    Step::Key("escape"),
    Step::Click("prompt"),
    Step::Wait(400),
    Step::Type("drafts/2027"),
    Step::Wait(300),
    Step::Shot("prompt"),
    Step::Key("escape"),
    Step::Click("fullscreen"),
    Step::Wait(600),
    Step::Shot("fullscreen"),
    Step::Key("escape"),
    Step::Click("lightbox-0"),
    Step::Wait(700),
    Step::Shot("lightbox"),
    Step::Key("right"),
    Step::Wait(500),
    Step::Shot("lightbox-next"),
    Step::Key("escape"),
    Step::Click("tour"),
    Step::Wait(700),
    Step::Shot("tour"),
    Step::Key("right"),
    Step::Wait(700),
    Step::Shot("tour-next"),
    Step::Key("escape"),
    Step::Click("toolbar"),
    Step::Key("secondary-a"),
    Step::Wait(400),
    Step::Shot("toolbar"),
    Step::Click("peek"),
    Step::Wait(500),
    Step::Shot("peek"),
    Step::Click("peek"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(anchored::popovers(window, cx))
        .child(anchored::hover_card(cx))
        .child(dialogs::dialogs(window, cx))
        .child(guides::lightbox(window, cx))
        .child(guides::tour(window, cx))
        .child(guides::toolbar(window, cx))
        .child(guides::peek(window, cx))
        .child(dialogs::pointers(cx))
        .into_any_element()
}
