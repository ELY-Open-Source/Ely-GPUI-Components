mod motion;
mod text;
mod values;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 2,
    slug: "typography",
    title: "Typography",
    summary: "Inter for words, tabular figures for numbers, one ink for everything.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Hover("fit-tooltip"),
    Step::Wait(700),
    Step::Shot("ellipsis-tooltip"),
    Step::Rest,
    Step::ClickEnd("copy-token"),
    Step::Shot("copied"),
    Step::Click("select-text"),
    Step::Key("cmd-a"),
    Step::Shot("select-all"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(text::headings(cx))
        .child(text::title(cx))
        .child(text::paragraph(cx))
        .child(text::label(cx))
        .child(text::caption(cx))
        .child(text::overline(cx))
        .child(text::inline_code(cx))
        .child(text::kbd(cx))
        .child(text::kbd_combo(cx))
        .child(text::blockquote(cx))
        .child(text::highlight(cx))
        .child(text::truncate(cx))
        .child(text::line_clamp(cx))
        .child(text::middle(cx))
        .child(text::ellipsis_tooltip(cx))
        .child(text::link(cx))
        .child(text::external_link(cx))
        .child(values::relative_time(cx))
        .child(values::date_time(cx))
        .child(values::numbers(cx))
        .child(values::currency(cx))
        .child(values::percent(cx))
        .child(values::file_size(cx))
        .child(values::duration(cx))
        .child(values::plural(cx))
        .child(values::copyable(cx))
        .child(values::selectable(cx))
        .child(values::emoji(cx))
        .child(values::latex(cx))
        .child(motion::animated_number(window, cx))
        .child(motion::typewriter(window, cx))
        .child(motion::gradient(cx))
        .child(motion::shimmer(cx))
        .into_any_element()
}
