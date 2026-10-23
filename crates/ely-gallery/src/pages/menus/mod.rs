mod more;
mod rows;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 8,
    slug: "menus",
    title: "Menus",
    summary: "Rows of actions that open from a button, a right click or an arrow, and nest.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Click("file-menu"),
    Step::Wait(300),
    Step::Key("down"),
    Step::Key("down"),
    Step::Key("down"),
    Step::Key("right"),
    Step::Wait(400),
    Step::Shot("submenu"),
    Step::Key("escape"),
    Step::Key("escape"),
    Step::Click("view-menu"),
    Step::Wait(400),
    Step::Shot("checks"),
    Step::Key("escape"),
    Step::ClickEnd("merge"),
    Step::Wait(400),
    Step::Shot("split"),
    Step::Key("escape"),
    Step::Click("more"),
    Step::Wait(400),
    Step::Shot("overflow"),
    Step::Key("escape"),
    Step::RightAt("canvas", 180.0, 70.0),
    Step::Wait(400),
    Step::Shot("context"),
    Step::Key("escape"),
    Step::DownAt("menubar", 16.0, 12.0),
    Step::UpAt("menubar", 16.0, 12.0),
    Step::Wait(400),
    Step::Shot("menubar"),
    Step::Key("right"),
    Step::Wait(300),
    Step::Shot("menubar-view"),
    Step::Key("escape"),
    Step::Click("move"),
    Step::Wait(400),
    Step::Type("re"),
    Step::Wait(300),
    Step::Shot("searchable"),
    Step::Key("escape"),
    Step::RightAt("pie-area", 210.0, 130.0),
    Step::Wait(600),
    Step::Key("right"),
    Step::Key("right"),
    Step::Wait(300),
    Step::Shot("pie"),
    Step::Key("escape"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(rows::file_menu(window, cx))
        .child(rows::view_menu(window, cx))
        .child(rows::hosts(window, cx))
        .child(rows::context(window, cx))
        .child(more::menu_bar(cx))
        .child(more::mega_menu(cx))
        .child(more::searchable(window, cx))
        .child(more::pie(window, cx))
        .into_any_element()
}
