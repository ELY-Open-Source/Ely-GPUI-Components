use ely_gpui_component::misc::{Calculator, Clock, Stopwatch, UnitConverter, WorldClock};
use gpui::{AnyElement, App, IntoElement, ParentElement, Styled, Window, div, px};
use jiff::tz::TimeZone;

use super::Page;
use crate::probe::probe;
use crate::step::Step;
use crate::ui::{section, specimen, specimens};

mod asking;
mod codes;
mod notices;
mod transfer;
mod web;

pub const PAGE: Page = Page {
    number: 42,
    slug: "misc",
    title: "Misc",
    summary: "Small tools that stand alone: clocks, a stopwatch, a calculator, a unit converter, a QR code scanner, a captcha, a poll, a survey, a quiz, flashcards, licenses, terms, a cookie banner, data out and in, and web views.",
    render,
    script: &[
        Step::DownAt("stopwatch-start", 30.0, 62.0),
        Step::UpAt("stopwatch-start", 30.0, 62.0),
        Step::Wait(1_600),
        Step::DownAt("stopwatch-start", 30.0, 62.0),
        Step::UpAt("stopwatch-start", 30.0, 62.0),
        Step::Wait(300),
        Step::Shot("stopwatch"),
        Step::DownAt("calculator", 24.0, 24.0),
        Step::UpAt("calculator", 24.0, 24.0),
        Step::Type("1234+5*6"),
        Step::Shot("calculator-typed"),
        Step::Key("enter"),
        Step::Shot("calculator-result"),
        Step::DownAt("converter", 24.0, 60.0),
        Step::UpAt("converter", 24.0, 60.0),
        Step::Key("cmd-a"),
        Step::Type("1000000000000000"),
        Step::Shot("converter-long"),
        Step::DownAt("captcha", 24.0, 104.0),
        Step::UpAt("captcha", 24.0, 104.0),
        Step::Type("abcd"),
        Step::Key("enter"),
        Step::Shot("captcha-wrong"),
        Step::Type("w7xk"),
        Step::Key("enter"),
        Step::Shot("captcha-passed"),
        Step::DownAt("poll", 50.0, 70.0),
        Step::UpAt("poll", 50.0, 70.0),
        Step::DownAt("poll", 27.0, 135.0),
        Step::UpAt("poll", 27.0, 135.0),
        Step::Shot("poll-voted"),
        Step::DownAt("quiz", 45.0, 103.0),
        Step::UpAt("quiz", 45.0, 103.0),
        Step::DownAt("quiz", 33.0, 168.0),
        Step::UpAt("quiz", 33.0, 168.0),
        Step::Shot("quiz-checked"),
        Step::DownAt("flashcards", 140.0, 80.0),
        Step::UpAt("flashcards", 140.0, 80.0),
        Step::Shot("flashcard-turned"),
        Step::DownAt("consent-open", 20.0, 16.0),
        Step::UpAt("consent-open", 20.0, 16.0),
        Step::Shot("consent"),
        Step::Key("escape"),
        Step::DownAt("cookies", 230.0, 84.0),
        Step::UpAt("cookies", 230.0, 84.0),
        Step::HoverAt("cookies", 20.0, 20.0),
        Step::Shot("cookies-choosing"),
        Step::DownAt("open-export", 20.0, 16.0),
        Step::UpAt("open-export", 20.0, 16.0),
        Step::Shot("export"),
        Step::Key("escape"),
        Step::DownAt("open-csv", 20.0, 16.0),
        Step::UpAt("open-csv", 20.0, 16.0),
        Step::Shot("csv-import"),
        Step::Key("escape"),
        Step::DownAt("web", 240.0, 100.0),
        Step::UpAt("web", 240.0, 100.0),
        Step::DownAt("converter", 24.0, 60.0),
        Step::UpAt("converter", 24.0, 60.0),
        Step::Key("cmd-a"),
        Step::NativeKey("5"),
        Step::Shot("keys-back"),
    ],
};

fn zone(name: &str) -> TimeZone {
    TimeZone::get(name).unwrap_or_else(|error| panic!("the zone {name}: {error}"))
}

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(clocks(cx))
        .child(stopwatch(cx))
        .child(calculator(cx))
        .child(converter(cx))
        .child(codes::render(window, cx))
        .child(asking::render(window, cx))
        .child(notices::render(window, cx))
        .child(transfer::render(window, cx))
        .child(web::render(cx))
        .into_any_element()
}

fn clocks(cx: &App) -> impl IntoElement + use<> {
    section(
        "Clock / WorldClock",
        "The time in a zone, a face over its digits, ticking each second. A world clock lays places in rows: how each one's day and hour stand against home.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("clock", Clock::new("clock").label("Here"), cx))
            .child(specimen(
                "world clock",
                div().w(px(320.0)).child(WorldClock::new(
                    "world-clock",
                    [
                        ("San Francisco", zone("America/Los_Angeles")),
                        ("New York", zone("America/New_York")),
                        ("London", zone("Europe/London")),
                        ("Mumbai", zone("Asia/Kolkata")),
                        ("Tokyo", zone("Asia/Tokyo")),
                    ],
                )),
                cx,
            )),
    )
}

fn stopwatch(cx: &App) -> impl IntoElement + use<> {
    section(
        "Stopwatch",
        "Time that runs while started and holds while paused, in tenths. Lap marks the time so far; once paused, it reads Reset and clears it.",
        cx,
    )
    .child(probe("stopwatch-start", div().w(px(320.0)).child(Stopwatch::new("stopwatch"))))
}

fn calculator(cx: &App) -> impl IntoElement + use<> {
    section(
        "Calculator",
        "Digits, the four operations, sign, delete and clear, worked in precedence; keys type too, and Enter gives the result.",
        cx,
    )
    .child(probe(
        "calculator",
        div().w(px(280.0)).child(Calculator::new("calculator")),
    ))
}

fn converter(cx: &App) -> impl IntoElement + use<> {
    section(
        "UnitConverter",
        "An amount read in another unit: length, area, volume, mass, temperature, speed and data, with a swap.",
        cx,
    )
    .child(probe(
        "converter",
        div().w(px(280.0)).child(UnitConverter::new("unit-converter")),
    ))
}
