use std::time::Duration;

use ely_gpui_component::forms::{
    Calendar, CronEditor, DatePicker, DateRangePicker, DateTimePicker, DurationPicker, MonthPicker,
    QuarterPicker, RelativeDatePicker, RelativeRange, TimePicker, TimezoneSelect, WeekPicker,
    YearPicker,
};
use gpui::{App, Div, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::civil::{Date, DateTime, Time, date, time};

use super::text::field;
use crate::{
    probe::probe,
    ui::{code, keep, section, set, specimen, specimens},
};

/// Today, fixed so captures stay the same from day to day.
const TODAY: Date = date(2026, 9, 25);

fn wide(element: impl IntoElement, width: f32) -> Div {
    div().w(px(width)).child(element)
}

pub fn calendar(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let day = keep("calendar-day", || date(2026, 9, 18), window, cx);
    let now = *day.read(cx);
    section(
        "Calendar",
        "Arrows move a cursor, Page keys turn the month, Enter picks. Today wears a ring.",
        cx,
    )
    .child(specimen(
        format!("picked {now}"),
        probe(
            "calendar",
            Calendar::new("calendar-demo")
                .selected(now)
                .today(TODAY)
                .on_pick(move |picked, _, cx| set(&day, picked, cx)),
        ),
        cx,
    ))
}

pub fn pickers(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let day = keep("picker-day", || Some(date(2026, 10, 2)), window, cx);
    let span = keep(
        "picker-span",
        || Some((date(2026, 9, 14), date(2026, 9, 18))),
        window,
        cx,
    );
    let at = keep("picker-at", || None::<DateTime>, window, cx);
    let clock = keep("picker-time", || Some(time(9, 30, 0, 0)), window, cx);
    let (day_now, span_now, at_now, clock_now) =
        (*day.read(cx), *span.read(cx), *at.read(cx), *clock.read(cx));
    section(
        "DatePicker / DateRangePicker / DateTimePicker / TimePicker",
        "Fields that open a calendar or time columns. Two clicks set a span.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "date",
                probe(
                    "date-picker",
                    wide(
                        DatePicker::new("picker-day", day_now)
                            .today(TODAY)
                            .on_change(move |picked, _, cx| set(&day, Some(picked), cx)),
                        260.0,
                    ),
                ),
                cx,
            ))
            .child(specimen(
                "range",
                probe(
                    "range-picker",
                    wide(
                        DateRangePicker::new("picker-span", span_now)
                            .today(TODAY)
                            .on_change(move |picked, _, cx| set(&span, Some(picked), cx)),
                        260.0,
                    ),
                ),
                cx,
            )),
    )
    .child(
        specimens()
            .child(specimen(
                "date and time",
                wide(
                    DateTimePicker::new("picker-at", at_now)
                        .today(TODAY)
                        .on_change(move |picked, _, cx| set(&at, Some(picked), cx)),
                    260.0,
                ),
                cx,
            ))
            .child(specimen(
                "time, 15-minute steps",
                probe(
                    "time-picker",
                    wide(
                        TimePicker::new("picker-time", clock_now)
                            .step(15)
                            .on_change(move |picked: Time, _, cx| set(&clock, Some(picked), cx)),
                        260.0,
                    ),
                ),
                cx,
            )),
    )
}

pub fn periods(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let month = keep("period-month", || Some((2026, 9)), window, cx);
    let quarter = keep("period-quarter", || Some((2026, 3)), window, cx);
    let year = keep("period-year", || Some(2026), window, cx);
    let week = keep("period-week", || Some((2026, 39)), window, cx);
    let now = (
        *month.read(cx),
        *quarter.read(cx),
        *year.read(cx),
        *week.read(cx),
    );
    section(
        "MonthPicker / YearPicker / QuarterPicker / WeekPicker",
        "Pages of periods; arrows move, Page keys turn the page. Weeks follow ISO 8601.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "month",
                probe(
                    "month-picker",
                    wide(
                        MonthPicker::new("period-month", now.0)
                            .today(TODAY)
                            .on_change(move |picked, _, cx| set(&month, Some(picked), cx)),
                        200.0,
                    ),
                ),
                cx,
            ))
            .child(specimen(
                "quarter",
                wide(
                    QuarterPicker::new("period-quarter", now.1)
                        .today(TODAY)
                        .on_change(move |picked, _, cx| set(&quarter, Some(picked), cx)),
                    200.0,
                ),
                cx,
            ))
            .child(specimen(
                "year",
                wide(
                    YearPicker::new("period-year", now.2)
                        .today(TODAY)
                        .on_change(move |picked, _, cx| set(&year, Some(picked), cx)),
                    200.0,
                ),
                cx,
            ))
            .child(specimen(
                "week",
                wide(
                    WeekPicker::new("period-week", now.3)
                        .today(TODAY)
                        .on_change(move |picked, _, cx| set(&week, Some(picked), cx)),
                    200.0,
                ),
                cx,
            )),
    )
}

pub fn more(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let range = keep(
        "relative-range",
        || Some(RelativeRange::Last7Days),
        window,
        cx,
    );
    let zone = keep(
        "zone-name",
        || SharedString::from("Europe/Lisbon"),
        window,
        cx,
    );
    let span = keep("duration-span", || Duration::from_secs(90 * 60), window, cx);
    let zone_field = field("input-zone", window, cx, |input| {
        input.placeholder("Search zones")
    });
    let (range_now, zone_now, span_now) = (*range.read(cx), zone.read(cx).clone(), *span.read(cx));
    section(
        "RelativeDatePicker / TimezoneSelect / DurationPicker",
        "Spans named from today, every zone the system knows, and hours and minutes.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "relative",
                div().w(px(260.0)).child(
                    RelativeDatePicker::new("relative-range", range_now)
                        .today(TODAY)
                        .on_change(move |picked, _, _, cx| set(&range, Some(picked), cx)),
                ),
                cx,
            ))
            .child(specimen(
                "time zone",
                div().w(px(260.0)).child(
                    TimezoneSelect::new("zone-name", &zone_field)
                        .selected(zone_now)
                        .on_change(move |picked, _, cx| set(&zone, picked.clone(), cx)),
                ),
                cx,
            ))
            .child(specimen(
                format!("duration {} min", span_now.as_secs() / 60),
                DurationPicker::new("duration-span", span_now)
                    .on_change(move |picked, _, cx| set(&span, picked, cx)),
                cx,
            )),
    )
}

pub fn cron(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let rule = field("input-cron", window, cx, |input| {
        input.placeholder("*/15 9-17 * * 1-5")
    });
    section(
        "CronEditor",
        "Five fields, read back in words with the next runs. Presets fill it in.",
        cx,
    )
    .child(probe(
        "cron",
        div()
            .w(px(520.0))
            .child(CronEditor::new("cron-rule", &rule).now(TODAY.at(12, 0, 0, 0))),
    ))
    .child(code("CronRule::parse(\"0 9 * * 1-5\")?.next(now, 3)", cx))
}
