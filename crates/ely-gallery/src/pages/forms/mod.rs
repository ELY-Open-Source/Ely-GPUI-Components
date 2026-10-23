mod choices;
mod colors;
mod dates;
mod files;
mod formats;
mod numbers;
mod other;
mod pickers;
mod rich;
mod structure;
mod text;
mod values;

use gpui::{AnyElement, App, IntoElement, ParentElement, Window, div};

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 6,
    slug: "forms",
    title: "Forms",
    summary: "Text, choices, dates, colors, files: every way to say what you mean.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::Rest,
    Step::Click("input-name"),
    Step::Key("secondary-a"),
    Step::Type("Ada Lovelace"),
    Step::Key("shift-left"),
    Step::Key("shift-left"),
    Step::Key("shift-left"),
    Step::Wait(300),
    Step::Shot("typed"),
    Step::Click("textarea"),
    Step::Key("secondary-a"),
    Step::Type("First line of notes"),
    Step::Key("enter"),
    Step::Type("Second line, longer, so the words wrap when they reach the edge of the box"),
    Step::Wait(300),
    Step::Shot("textarea"),
    Step::Click("password"),
    Step::Key("secondary-a"),
    Step::Type("hunter2"),
    Step::Wait(200),
    Step::Shot("password"),
    Step::Click("search"),
    Step::Wait(300),
    Step::Shot("search-history"),
    Step::Click("number"),
    Step::Key("up"),
    Step::Key("up"),
    Step::Wait(300),
    Step::Shot("number"),
    Step::DownAt("scrub", 12.0, 14.0),
    Step::DragTo("scrub", 72.0, 14.0),
    Step::UpAt("scrub", 72.0, 14.0),
    Step::Wait(300),
    Step::Shot("scrubbed"),
    Step::Click("email"),
    Step::Key("secondary-a"),
    Step::Type("ada@example"),
    Step::Click("phone"),
    Step::Key("secondary-a"),
    Step::Type("4155550132"),
    Step::Click("masked"),
    Step::Key("secondary-a"),
    Step::Type("ab1234"),
    Step::Wait(300),
    Step::Shot("formats"),
    Step::Click("pin"),
    Step::Key("secondary-a"),
    Step::Type("4821"),
    Step::Wait(300),
    Step::Shot("pin"),
    Step::Click("inline"),
    Step::Key("secondary-a"),
    Step::Type("Q3 planning"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("inline"),
    Step::Click("mention"),
    Step::Key("secondary-a"),
    Step::Type("Ask @gr"),
    Step::Wait(300),
    Step::Shot("mention"),
    Step::Key("enter"),
    Step::Type("about #rel"),
    Step::Key("down"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("mentioned"),
    Step::Click("tags"),
    Step::Type("motion,design"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("tags"),
    Step::Click("regex"),
    Step::Key("secondary-a"),
    Step::Type("^(ab+"),
    Step::Wait(300),
    Step::Shot("regex"),
    Step::Click("expression"),
    Step::Key("secondary-a"),
    Step::Type("price * qty + max(2, 4)"),
    Step::Wait(300),
    Step::Shot("expression"),
    Step::Click("hotkey"),
    Step::Key("cmd-shift-k"),
    Step::Wait(300),
    Step::Shot("hotkey"),
    Step::DownAt("path", 380.0, 16.0),
    Step::UpAt("path", 380.0, 16.0),
    Step::CancelPanel,
    Step::Wait(600),
    Step::DownAt("checks", 40.0, 66.0),
    Step::UpAt("checks", 40.0, 66.0),
    Step::Wait(300),
    Step::Shot("checks"),
    Step::DownAt("radios", 30.0, 66.0),
    Step::UpAt("radios", 30.0, 66.0),
    Step::Wait(400),
    Step::Shot("radios"),
    Step::DownAt("cards", 480.0, 30.0),
    Step::UpAt("cards", 480.0, 30.0),
    Step::Wait(400),
    Step::Shot("cards"),
    Step::DownAt("switches", 16.0, 39.0),
    Step::UpAt("switches", 16.0, 39.0),
    Step::Wait(600),
    Step::Shot("switches"),
    Step::Click("select"),
    Step::Wait(300),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("select-open"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Click("combobox"),
    Step::Key("secondary-a"),
    Step::Type("lis"),
    Step::Wait(300),
    Step::Shot("combobox"),
    Step::Key("enter"),
    Step::Wait(200),
    Step::Click("multi"),
    Step::Wait(300),
    Step::Key("down"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("multi"),
    Step::Key("escape"),
    Step::Click("cascader"),
    Step::Wait(300),
    Step::Key("down"),
    Step::Key("right"),
    Step::Wait(300),
    Step::Shot("cascader"),
    Step::Key("escape"),
    Step::DownAt("transfer", 60.0, 45.0),
    Step::UpAt("transfer", 60.0, 45.0),
    Step::Wait(200),
    Step::DownAt("transfer", 280.0, 134.0),
    Step::UpAt("transfer", 280.0, 134.0),
    Step::Wait(300),
    Step::Shot("transfer"),
    Step::DownAt("chips", 120.0, 14.0),
    Step::UpAt("chips", 120.0, 14.0),
    Step::DownAt("rating", 79.0, 13.0),
    Step::UpAt("rating", 79.0, 13.0),
    Step::Wait(300),
    Step::Shot("chips-rating"),
    Step::DownAt("slider", 130.0, 8.0),
    Step::DragTo("slider", 220.0, 8.0),
    Step::UpAt("slider", 220.0, 8.0),
    Step::Wait(200),
    Step::DownAt("knob", 24.0, 24.0),
    Step::DragTo("knob", 24.0, -36.0),
    Step::UpAt("knob", 24.0, -36.0),
    Step::DownAt("stepper", 76.0, 16.0),
    Step::UpAt("stepper", 76.0, 16.0),
    Step::Wait(300),
    Step::Shot("values"),
    Step::DownAt("calendar", 150.0, 150.0),
    Step::UpAt("calendar", 150.0, 150.0),
    Step::Key("right"),
    Step::Key("pagedown"),
    Step::Wait(400),
    Step::Shot("calendar"),
    Step::Click("date-picker"),
    Step::Wait(300),
    Step::Shot("date-open"),
    Step::Key("right"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Click("range-picker"),
    Step::Wait(300),
    Step::Key("enter"),
    Step::Key("right"),
    Step::Key("right"),
    Step::Key("right"),
    Step::Wait(200),
    Step::Shot("range-open"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Click("time-picker"),
    Step::Wait(300),
    Step::Shot("time-open"),
    Step::Key("escape"),
    Step::Click("month-picker"),
    Step::Wait(300),
    Step::Shot("month-open"),
    Step::Key("escape"),
    Step::Click("cron"),
    Step::Key("secondary-a"),
    Step::Type("*/15 9-17 * * 1-5"),
    Step::Wait(200),
    Step::Shot("cron"),
    Step::DownAt("color-picker", 60.0, 40.0),
    Step::DragTo("color-picker", 200.0, 60.0),
    Step::UpAt("color-picker", 200.0, 60.0),
    Step::Wait(300),
    Step::Shot("color"),
    Step::DownAt("color-picker", 100.0, 228.0),
    Step::UpAt("color-picker", 100.0, 228.0),
    Step::Wait(200),
    Step::Key("down"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("color-rgb"),
    Step::DownAt("palette", 50.0, 14.0),
    Step::UpAt("palette", 50.0, 14.0),
    Step::Wait(300),
    Step::Shot("swatches"),
    Step::DownAt("gradient", 108.0, 16.0),
    Step::UpAt("gradient", 108.0, 16.0),
    Step::Wait(300),
    Step::Shot("gradient"),
    Step::DownAt("file-one", 100.0, 14.0),
    Step::UpAt("file-one", 100.0, 14.0),
    Step::CancelPanel,
    Step::Wait(600),
    Step::Shot("files"),
    Step::DownAt("signature", 40.0, 100.0),
    Step::DragTo("signature", 70.0, 50.0),
    Step::DragTo("signature", 100.0, 110.0),
    Step::DragTo("signature", 130.0, 55.0),
    Step::DragTo("signature", 160.0, 105.0),
    Step::DragTo("signature", 210.0, 60.0),
    Step::DragTo("signature", 250.0, 100.0),
    Step::DragTo("signature", 320.0, 80.0),
    Step::UpAt("signature", 320.0, 80.0),
    Step::Wait(300),
    Step::Shot("signature"),
    Step::Click("code"),
    Step::Key("secondary-a"),
    Step::Type("let total = price(3) * 1.2; // tax"),
    Step::Click("json"),
    Step::Key("secondary-a"),
    Step::Type("{\"name\": \"Ely\", \"size\": 3,"),
    Step::Wait(300),
    Step::Shot("code-json"),
    Step::DownAt("font", 100.0, 14.0),
    Step::UpAt("font", 100.0, 14.0),
    Step::Key("secondary-a"),
    Step::Type("mono"),
    Step::Wait(300),
    Step::Shot("font"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::DownAt("icons", 100.0, 14.0),
    Step::UpAt("icons", 100.0, 14.0),
    Step::Key("secondary-a"),
    Step::Type("arrow"),
    Step::Key("down"),
    Step::Key("right"),
    Step::Key("down"),
    Step::Wait(300),
    Step::Shot("icons"),
    Step::Key("enter"),
    Step::DownAt("emoji", 100.0, 14.0),
    Step::UpAt("emoji", 100.0, 14.0),
    Step::Key("secondary-a"),
    Step::Type("heart"),
    Step::Key("down"),
    Step::Key("right"),
    Step::Wait(300),
    Step::Shot("emoji"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Click("country"),
    Step::Key("secondary-a"),
    Step::Type("jap"),
    Step::Wait(300),
    Step::Shot("country"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Click("form-name"),
    Step::Key("secondary-a"),
    Step::Type("Ada Lovelace"),
    Step::Click("form-email"),
    Step::Key("secondary-a"),
    Step::Type("ada.example.com"),
    Step::Wait(400),
    Step::Shot("form"),
    Step::Hover("form-actions"),
    Step::Wait(300),
    Step::Shot("form-dirty"),
    Step::Key("secondary-a"),
    Step::Type("ada@example.com"),
    Step::Key("cmd-enter"),
    Step::Wait(400),
    Step::Shot("form-saved"),
    Step::Rest,
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(text::input(window, cx))
        .child(text::text_area(window, cx))
        .child(text::password(window, cx))
        .child(text::search(window, cx))
        .child(numbers::number_input(window, cx))
        .child(numbers::scrub(window, cx))
        .child(numbers::money(window, cx))
        .child(formats::email_url(window, cx))
        .child(formats::phone_masked(window, cx))
        .child(formats::pin(window, cx))
        .child(formats::inline_edit(window, cx))
        .child(text::group(window, cx))
        .child(rich::mention(window, cx))
        .child(rich::tags(window, cx))
        .child(rich::rows(cx))
        .child(rich::path(window, cx))
        .child(rich::regex(window, cx))
        .child(rich::expression(window, cx))
        .child(rich::hotkey(window, cx))
        .child(choices::checkboxes(window, cx))
        .child(choices::radios(window, cx))
        .child(choices::cards(window, cx))
        .child(choices::switches(window, cx))
        .child(pickers::select(window, cx))
        .child(pickers::list_box(window, cx))
        .child(pickers::combobox(window, cx))
        .child(pickers::multi_select(window, cx))
        .child(pickers::cascader(window, cx))
        .child(pickers::transfer(window, cx))
        .child(values::chips(window, cx))
        .child(values::rating(window, cx))
        .child(values::sliders(window, cx))
        .child(values::dials(window, cx))
        .child(dates::calendar(window, cx))
        .child(dates::pickers(window, cx))
        .child(dates::periods(window, cx))
        .child(dates::more(window, cx))
        .child(dates::cron(window, cx))
        .child(colors::picker(window, cx))
        .child(colors::swatches(window, cx))
        .child(colors::gradient(window, cx))
        .child(files::files(window, cx))
        .child(other::signature(window, cx))
        .child(other::code(window, cx))
        .child(other::font(window, cx))
        .child(other::glyphs(window, cx))
        .child(other::regional(window, cx))
        .child(structure::form(window, cx))
        .child(structure::inline(window, cx))
        .into_any_element()
}
