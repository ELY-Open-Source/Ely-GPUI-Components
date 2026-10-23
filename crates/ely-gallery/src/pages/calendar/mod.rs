use ely_gpui_component::{
    calendar::Event,
    theme::{ActiveTheme, TextSize},
};
use gpui::{
    AnyElement, App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div,
};
use jiff::{Timestamp, ToSpan, civil::Date, tz::TimeZone};

mod editing;
mod months;
mod people;
mod times;

use super::Page;
use crate::{step::Step, ui::keep};

pub const PAGE: Page = Page {
    number: 30,
    slug: "calendar",
    title: "Calendar",
    summary: "Days, weeks, months and years, and what happens in them.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("calendar-month", 285.0, 230.0),
    Step::UpAt("calendar-month", 285.0, 230.0),
    Step::Key("right"),
    Step::Key("down"),
    Step::Wait(150),
    Step::Shot("month-cursor"),
    Step::Key("pagedown"),
    Step::Wait(200),
    Step::Shot("month-next"),
    Step::Key("pageup"),
    Step::DownAt("calendar-year", 150.0, 150.0),
    Step::UpAt("calendar-year", 150.0, 150.0),
    Step::Key("right"),
    Step::Wait(150),
    Step::Shot("year-cursor"),
    Step::DownAt("calendar-week", 560.0, 300.0),
    Step::DragTo("calendar-week", 560.0, 380.0),
    Step::Wait(150),
    Step::Shot("week-drafting"),
    Step::UpAt("calendar-week", 560.0, 380.0),
    Step::Wait(200),
    Step::Shot("week-made"),
    Step::DownAt("calendar-popover", 60.0, 10.0),
    Step::UpAt("calendar-popover", 60.0, 10.0),
    Step::Wait(300),
    Step::Shot("popover-open"),
    Step::Key("escape"),
];

/// What a demo says it did, under it.
fn quiet(text: SharedString, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .text_size(theme.text_size(TextSize::Sm))
        .text_color(theme.colors.fg_muted)
        .child(text)
}

/// The gallery's zone and today in it.
fn here() -> (TimeZone, Date) {
    let zone = TimeZone::try_system().expect("the gallery reads the system time zone");
    let today = Timestamp::now().to_zoned(zone.clone()).date();
    (zone, today)
}

/// The page's events, which a drag on the week or day adds to.
fn plan(window: &mut Window, cx: &mut App) -> Entity<Vec<Event>> {
    let (zone, today) = here();
    keep("calendar-events", || events(today, &zone), window, cx)
}

/// A studio's weeks around today: meetings, a crowded day, an offsite, a trip past a weekend, a night shoot past midnight.
fn events(today: Date, zone: &TimeZone) -> Vec<Event> {
    let day = |offset: i64| today.checked_add(offset.days()).expect("a day nearby");
    let at = |offset: i64, hour: i8, minute: i8| {
        day(offset)
            .at(hour, minute, 0, 0)
            .to_zoned(zone.clone())
            .expect("the zone holds the hour")
            .timestamp()
    };
    vec![
        Event::timed("standup", "Studio stand-up", at(0, 9, 30), at(0, 9, 45), 0),
        Event::timed("review", "Design review", at(0, 11, 0), at(0, 12, 0), 3).place("Model room"),
        Event::timed("lunch", "Lunch with Ana", at(0, 12, 30), at(0, 13, 30), 5),
        Event::timed("samples", "Plaster samples", at(0, 15, 0), at(0, 16, 0), 2),
        Event::timed("client", "Client call", at(0, 17, 0), at(0, 17, 30), 1),
        Event::timed("crit", "Crit", at(-3, 14, 0), at(-3, 15, 30), 3).place("Studio 2"),
        Event::timed("print", "Print run", at(-9, 10, 0), at(-9, 11, 0), 2),
        Event::all_day("offsite", "Studio offsite", day(2), day(4), 4),
        Event::timed("night", "Night shoot", at(5, 21, 0), at(6, 2, 0), 6).place("Atrium"),
        Event::all_day("trip", "Site visit · Lisbon", day(9), day(13), 7),
        Event::timed("models", "Model photos", at(11, 10, 0), at(11, 12, 0), 0),
        Event::all_day("launch", "Launch", day(18), day(18), 1),
    ]
}

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(months::month(window, cx))
        .child(months::mini(window, cx))
        .child(times::week(window, cx))
        .child(times::day(window, cx))
        .child(people::agenda(window, cx))
        .child(people::details(window, cx))
        .child(editing::editor(window, cx))
        .child(editing::availability(window, cx))
        .child(months::year(window, cx))
        .child(months::marks(cx))
        .into_any_element()
}
