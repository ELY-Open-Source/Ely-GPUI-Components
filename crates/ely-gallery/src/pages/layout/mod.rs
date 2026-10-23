mod basics;
mod regions;
mod scrolling;
mod shells;
mod systems;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 3,
    slug: "layout",
    title: "Layout",
    summary: "Stacks, grids, splits, docks. Structure that stays out of the way.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Hover("scroll-area"),
    Step::Wait(300),
    Step::Shot("scroll-hover"),
    Step::DownAt("scrollbar", 310.0, 12.0),
    Step::DragTo("scrollbar", 310.0, 110.0),
    Step::UpAt("scrollbar", 310.0, 110.0),
    Step::Shot("scrollbar-drag"),
    Step::DownAt("scrollbar", 310.0, 110.0),
    Step::DragTo("scrollbar", 310.0, 12.0),
    Step::UpAt("scrollbar", 310.0, 12.0),
    Step::DownAt("sticky", 310.0, 12.0),
    Step::DragTo("sticky", 310.0, 76.0),
    Step::UpAt("sticky", 310.0, 76.0),
    Step::Shot("sticky-pushed"),
    Step::DownAt("sticky", 310.0, 76.0),
    Step::DragTo("sticky", 310.0, 12.0),
    Step::UpAt("sticky", 310.0, 12.0),
    Step::Click("collapse-toggle"),
    Step::Wait(400),
    Step::Shot("collapsed"),
    Step::Click("collapse-toggle"),
    Step::Click("sidebar-toggle"),
    Step::Wait(400),
    Step::Shot("sidebar-collapsed"),
    Step::Click("sidebar-toggle"),
    Step::Rest,
    Step::Click("sheet-open"),
    Step::Wait(400),
    Step::Shot("sheet"),
    Step::Click("sheet-open"),
    Step::Click("drawer-open"),
    Step::Wait(400),
    Step::Shot("drawer"),
    Step::DownAt("drawer-body", 10.0, -28.0),
    Step::DragTo("drawer-body", 10.0, 172.0),
    Step::UpAt("drawer-body", 10.0, 172.0),
    Step::Wait(400),
    Step::Shot("drawer-pulled"),
    Step::Click("floating-open"),
    Step::Wait(300),
    Step::Shot("floating"),
    Step::Click("floating-open"),
    Step::DownAt("dock", 300.0, 240.0),
    Step::DragTo("dock", 380.0, 60.0),
    Step::Wait(200),
    Step::Shot("dock-zones"),
    Step::UpAt("dock", 380.0, 60.0),
    Step::Wait(200),
    Step::Shot("dock-top"),
    Step::DownAt("dock", 300.0, 20.0),
    Step::DragTo("dock", 380.0, 330.0),
    Step::UpAt("dock", 380.0, 330.0),
    Step::Click("workspace-save"),
    Step::DownAt("panes", 380.0, 150.0),
    Step::DragTo("panes", 560.0, 150.0),
    Step::UpAt("panes", 560.0, 150.0),
    Step::Shot("panes-dragged"),
    Step::Click("workspace-restore"),
    Step::Wait(300),
    Step::Shot("panes-restored"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(basics::stacks(cx))
        .child(basics::flex(cx))
        .child(basics::grid(cx))
        .child(basics::simple_grid(cx))
        .child(basics::masonry(cx))
        .child(basics::center(cx))
        .child(basics::container(cx))
        .child(basics::aspect_ratio(cx))
        .child(basics::inset(cx))
        .child(basics::wrap(cx))
        .child(basics::absolute(cx))
        .child(basics::z_stack(cx))
        .child(scrolling::scroll_area(cx))
        .child(scrolling::scrollbar(window, cx))
        .child(scrolling::scroll_shadow(window, cx))
        .child(scrolling::scroll_to_top(window, cx))
        .child(scrolling::sticky_header(window, cx))
        .child(regions::split_pane(cx))
        .child(regions::resize_handles(cx))
        .child(regions::collapsible(window, cx))
        .child(regions::accordion(cx))
        .child(regions::card(cx))
        .child(regions::panel(cx))
        .child(regions::section_demo(cx))
        .child(regions::fieldset(cx))
        .child(regions::frame_demo(cx))
        .child(regions::well(cx))
        .child(shells::sidebar(window, cx))
        .child(shells::drawer(window, cx))
        .child(shells::sheet(window, cx))
        .child(shells::page(cx))
        .child(shells::app_shell(cx))
        .child(shells::master_detail(cx))
        .child(systems::dock(window, cx))
        .child(systems::dock_zones(cx))
        .child(systems::floating(window, cx))
        .child(systems::panes(window, cx))
        .child(systems::workspace(window, cx))
        .child(systems::viewport(cx))
        .into_any_element()
}
