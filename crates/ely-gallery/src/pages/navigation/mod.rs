mod palettes;
mod paths;
mod places;
mod tabs;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 7,
    slug: "navigation",
    title: "Navigation",
    summary: "Tabs, paths, places and palettes: where you are, and the ways out.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::DownAt("tabs-top", 128.0, 14.0),
    Step::UpAt("tabs-top", 128.0, 14.0),
    Step::Wait(500),
    Step::Shot("tabs"),
    Step::Hover("editor-tabs"),
    Step::Wait(300),
    Step::Shot("editor-hover"),
    Step::DownAt("editor-tabs", 544.0, 16.0),
    Step::UpAt("editor-tabs", 544.0, 16.0),
    Step::Wait(300),
    Step::Shot("editor-overflow"),
    Step::Key("escape"),
    Step::DownAt("breadcrumb", 173.0, 10.0),
    Step::UpAt("breadcrumb", 173.0, 10.0),
    Step::Wait(300),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("breadcrumb-open"),
    Step::Key("enter"),
    Step::DownAt("pagination", 150.0, 10.0),
    Step::UpAt("pagination", 150.0, 10.0),
    Step::Wait(300),
    Step::Shot("pagination"),
    Step::DownAt("wizard", 531.0, 173.0),
    Step::UpAt("wizard", 531.0, 173.0),
    Step::Wait(500),
    Step::Shot("wizard"),
    Step::DownAt("nav-sidebar", 60.0, 58.0),
    Step::UpAt("nav-sidebar", 60.0, 58.0),
    Step::DownAt("nav-sidebar", 266.0, 26.0),
    Step::UpAt("nav-sidebar", 266.0, 26.0),
    Step::Wait(500),
    Step::Shot("nav-folded"),
    Step::DownAt("nav-sidebar", 28.0, 26.0),
    Step::UpAt("nav-sidebar", 28.0, 26.0),
    Step::DownAt("nav-menu", 42.0, 14.0),
    Step::UpAt("nav-menu", 42.0, 14.0),
    Step::Wait(300),
    Step::Shot("nav-menu-open"),
    Step::Key("right"),
    Step::Key("down"),
    Step::Wait(300),
    Step::Shot("nav-menu-docs"),
    Step::Key("escape"),
    Step::DownAt("history", 63.0, 12.0),
    Step::UpAt("history", 63.0, 12.0),
    Step::Wait(300),
    Step::Shot("history-list"),
    Step::Key("escape"),
    Step::DownAt("toc", 60.0, 90.0),
    Step::UpAt("toc", 60.0, 90.0),
    Step::Wait(500),
    Step::Shot("toc"),
    Step::DownAt("goto", 100.0, 14.0),
    Step::UpAt("goto", 100.0, 14.0),
    Step::Key("secondary-a"),
    Step::Type("120:8"),
    Step::Wait(200),
    Step::Shot("goto"),
    Step::Key("enter"),
    Step::Click("commands"),
    Step::Wait(500),
    Step::Shot("command-palette"),
    Step::Type("tog"),
    Step::Wait(300),
    Step::Shot("command-fuzzy"),
    Step::Key("escape"),
    Step::Click("files"),
    Step::Wait(400),
    Step::Type("navmenu"),
    Step::Wait(300),
    Step::Shot("quick-open"),
    Step::Key("escape"),
    Step::Click("switcher"),
    Step::Wait(500),
    Step::Shot("quick-switcher"),
    Step::Key("enter"),
    Step::Click("search"),
    Step::Wait(400),
    Step::Type("the"),
    Step::Wait(300),
    Step::Shot("search"),
    Step::Key("escape"),
    Step::Click("launcher"),
    Step::Wait(800),
    Step::KeyWindow("launcher", "t"),
    Step::KeyWindow("launcher", "o"),
    Step::KeyWindow("launcher", "k"),
    Step::Wait(300),
    Step::ShotWindow("launcher", "launcher"),
    Step::KeyWindow("launcher", "escape"),
    Step::Wait(300),
    Step::ExpectClosed("launcher"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(tabs::tabs(window, cx))
        .child(tabs::editor_tabs(window, cx))
        .child(paths::breadcrumb(window, cx))
        .child(paths::pagination(window, cx))
        .child(paths::steps(window, cx))
        .child(places::sidebar(window, cx))
        .child(places::menu(window, cx))
        .child(places::history(window, cx))
        .child(places::contents(window, cx))
        .child(places::go_to_line(window, cx))
        .child(palettes::command_palette(window, cx))
        .child(palettes::quick_open(window, cx))
        .child(palettes::quick_switcher(window, cx))
        .child(palettes::search_palette(window, cx))
        .into_any_element()
}
