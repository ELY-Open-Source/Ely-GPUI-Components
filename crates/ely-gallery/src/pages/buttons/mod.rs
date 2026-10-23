mod actions;
mod basics;
mod choices;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 5,
    slug: "buttons",
    title: "Buttons & Actions",
    summary: "Seven variants, three sizes, one focus ring, and the verbs around them.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("segmented", 150.0, 16.0),
    Step::UpAt("segmented", 150.0, 16.0),
    Step::Wait(500),
    Step::Shot("segmented"),
    Step::DownAt("confirm-hold", 40.0, 14.0),
    Step::Wait(500),
    Step::Shot("holding"),
    Step::Wait(900),
    Step::Shot("held"),
    Step::UpAt("confirm-hold", 40.0, 14.0),
    Step::Click("confirm-twice"),
    Step::Wait(200),
    Step::Shot("armed"),
    Step::Click("confirm-twice"),
    Step::Wait(200),
    Step::Shot("confirmed"),
    Step::Wait(3200),
    Step::Click("confirm-twice"),
    Step::Click("confirm-twice"),
    Step::Click("confirm-twice"),
    Step::Wait(1600),
    Step::Shot("twice-reset"),
    Step::Click("copy-button"),
    Step::Wait(300),
    Step::Shot("copied"),
    Step::Click("share-button"),
    Step::Wait(900),
    Step::ShotPopup("share-picker"),
    Step::NativeKey("escape"),
    Step::Wait(400),
    Step::Click("bulk-select"),
    Step::Click("bulk-select"),
    Step::Click("bulk-select"),
    Step::Wait(500),
    Step::Shot("bulk"),
    Step::Click("action-sheet-open"),
    Step::Wait(400),
    Step::Key("tab"),
    Step::Key("tab"),
    Step::Key("tab"),
    Step::Key("tab"),
    Step::Key("tab"),
    Step::Wait(200),
    Step::Shot("action-sheet"),
    Step::Key("escape"),
    Step::Click("quick"),
    Step::Key("down"),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("quick"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(basics::basics(cx))
        .child(actions::later(cx))
        .child(choices::toggle_button(window, cx))
        .child(choices::toggle_group(window, cx))
        .child(choices::segmented(window, cx))
        .child(actions::fab(cx))
        .child(actions::confirm(cx))
        .child(actions::small_buttons(cx))
        .child(actions::action_bar(cx))
        .child(actions::bulk(window, cx))
        .child(actions::action_sheet(window, cx))
        .child(actions::quick_actions(cx))
        .into_any_element()
}
