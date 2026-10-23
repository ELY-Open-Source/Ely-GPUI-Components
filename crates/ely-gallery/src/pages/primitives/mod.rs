mod native;
mod parts;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 1,
    slug: "primitives",
    title: "Primitives",
    summary: "The smallest parts. Most are gpui itself; Ely adds the rest.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Hover("tip-plain"),
    Step::Wait(700),
    Step::Shot("tooltip"),
    Step::Hover("tip-meta"),
    Step::Wait(700),
    Step::Shot("tooltip-meta"),
    Step::Hover("tip-rich"),
    Step::Wait(700),
    Step::Shot("tooltip-rich"),
    Step::Hover("tip-follow"),
    Step::Wait(200),
    Step::Shot("tooltip-follow"),
    Step::Rest,
    Step::Down("prim-press"),
    Step::Wait(700),
    Step::Shot("pressed"),
    Step::Up("prim-press"),
    Step::Rest,
    Step::Click("portal-toggle"),
    Step::Shot("portal"),
    Step::Click("portal-toggle"),
    Step::Click("backdrop-open"),
    Step::Wait(300),
    Step::Shot("backdrop"),
    Step::Click("backdrop-open"),
    Step::Click("trap-c"),
    Step::Key("tab"),
    Step::Shot("trap-wrap"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(native::boxes(cx))
        .child(native::text(cx))
        .child(parts::icon(cx))
        .child(parts::icon_set(cx))
        .child(native::svg_demo(cx))
        .child(parts::image(cx))
        .child(native::canvas_demo(cx))
        .child(parts::divider(cx))
        .child(native::spacer(cx))
        .child(native::portal(window, cx))
        .child(parts::backdrop(window, cx))
        .child(native::slot(cx))
        .child(native::show(window, cx))
        .child(native::each(cx))
        .child(native::fragment(cx))
        .child(parts::visually_hidden(cx))
        .child(parts::focus_ring(cx))
        .child(parts::focus_trap(window, cx))
        .child(native::click_outside(window, cx))
        .child(native::hover_area(window, cx))
        .child(parts::pressable(window, cx))
        .child(native::keyboard(window, cx))
        .child(parts::measure(window, cx))
        .child(parts::intersection(window, cx))
        .child(native::clipboard(window, cx))
        .child(parts::tooltip(cx))
        .into_any_element()
}
