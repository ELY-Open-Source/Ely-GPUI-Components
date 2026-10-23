mod machine;
mod profiling;
mod session;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 20,
    slug: "debug",
    title: "Debug",
    summary: "A paused program: its breakpoints, stack, threads and values, memory and machine code; profiles and network.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[Step::Rest];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(session::session(window, cx))
        .child(session::breakpoints_and_stack(window, cx))
        .child(session::values(window, cx))
        .child(machine::machine(window, cx))
        .child(profiling::profiles(window, cx))
        .child(profiling::network(window, cx))
        .into_any_element()
}
