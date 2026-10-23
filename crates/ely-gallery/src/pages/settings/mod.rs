use gpui::{AnyElement, App, Window, div, prelude::*};

mod pages;
mod window;

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 35,
    slug: "settings",
    title: "Settings",
    summary: "A settings window: its sections, a search over them, and the controls each one holds.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("settings-layout", 60.0, 16.0),
    Step::UpAt("settings-layout", 60.0, 16.0),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("keyboard"),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("network"),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("privacy"),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("notifications"),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("startup"),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("storage"),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("advanced"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div().child(window::settings(window, cx)).into_any_element()
}
