use ely_gpui_component::{
    calendar::{CalendarMonthView, Event, EventCard, EventChip, YearView},
    forms::Calendar,
};
use gpui::{
    App, ElementId, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
};
use jiff::civil::Date;

use super::{events, here, plan, quiet};
use crate::{
    probe::probe,
    ui::{change, keep, section, set},
};

/// The day picked in the small calendar, the month shown, and what the last press said.
#[derive(Clone)]
struct Viewing {
    day: Date,
    month: Date,
    said: Option<SharedString>,
}

/// The month shown and the day picked in the small calendar, shared by both sections.
fn viewing(window: &mut Window, cx: &mut App) -> Entity<Viewing> {
    let (_, today) = here();
    keep(
        "calendar-viewing",
        || Viewing {
            day: today,
            month: today.first_of_month(),
            said: None,
        },
        window,
        cx,
    )
}

pub fn month(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (zone, _) = here();
    let viewing = viewing(window, cx);
    let now = viewing.read(cx).clone();
    let (turned, opened, pressed) = (viewing.clone(), viewing.clone(), viewing);
    let events = plan(window, cx).read(cx).clone();
    let view = CalendarMonthView::new("calendar-month", now.month, events)
        .zone(zone)
        .on_month(move |first, _, cx| change(&turned, cx, |viewing| viewing.month = first))
        .on_day(move |day, _, cx| {
            let said = format!("Opened {}.", day.strftime("%A, %B %-d"));
            change(&opened, cx, |viewing| viewing.said = Some(said.into()))
        })
        .on_event(move |key, _, cx| {
            let said = format!("Pressed {key}.");
            change(&pressed, cx, |viewing| viewing.said = Some(said.into()))
        });
    section(
        "CalendarMonthView",
        "A month in week rows. What spans days lies across them, the rest sits by the hour, and a crowded day keeps the rest behind N more. Arrows move a day, Page keys turn the month, and Enter or a double press opens the day.",
        cx,
    )
    .child(probe(
        "calendar-month",
        div()
            .w(px(800.))
            .flex()
            .flex_col()
            .gap_3()
            .child(view)
            .children(now.said.map(|said| quiet(said, cx))),
    ))
}

pub fn mini(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let viewing = viewing(window, cx);
    let day = viewing.read(cx).day;
    section(
        "MiniCalendar → forms::Calendar",
        "A month at a glance is forms::Calendar. Here a day picked in it turns the month view above to its month.",
        cx,
    )
    .child(probe(
        "calendar-mini",
        div().w(px(252.)).child(Calendar::new("calendar-mini").selected(day).on_pick(
            move |day, _, cx| {
                change(&viewing, cx, |viewing| {
                    viewing.day = day;
                    viewing.month = day.first_of_month();
                })
            },
        )),
    ))
}

pub fn year(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (zone, today) = here();
    let events = plan(window, cx).read(cx).clone();
    let said = keep("calendar-year-said", || None::<SharedString>, window, cx);
    let shown = said.read(cx).clone();
    section(
        "YearView",
        "Twelve small months. A day with something on it wears a dot in its first event's hue; arrows move a day, Page keys a month, and Enter or a double press opens the day.",
        cx,
    )
    .child(probe(
        "calendar-year",
        div()
            .w(px(900.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                YearView::new("calendar-year", today.year(), events)
                    .zone(zone)
                    .on_day(move |day, _, cx| {
                        let opened = format!("Opened {}.", day.strftime("%A, %B %-d"));
                        set(&said, Some(opened.into()), cx)
                    }),
            )
            .children(shown.map(|shown| quiet(shown, cx))),
    ))
}

pub fn marks(cx: &mut App) -> impl IntoElement + use<> {
    let (zone, today) = here();
    let events = events(today, &zone);
    let event = |key: &str| -> Event {
        events
            .iter()
            .find(|event| event.key.as_ref() == key)
            .cloned()
            .expect("the demo holds the event")
    };
    let chip = |key: &str| {
        EventChip::new(
            (ElementId::from("calendar-chip"), key.to_string()),
            event(key),
        )
        .zone(zone.clone())
    };
    let card = |key: &str| {
        div().w(px(220.)).h(px(58.)).child(
            EventCard::new(
                (ElementId::from("calendar-card"), key.to_string()),
                event(key),
            )
            .zone(zone.clone()),
        )
    };
    section(
        "EventCard / EventChip",
        "An event in a line, by its hour or across whole days, and as a block with its hours and place, the way a time grid holds it.",
        cx,
    )
    .child(probe(
        "calendar-marks",
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_8()
            .child(
                div()
                    .w(px(200.))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(chip("review"))
                    .child(chip("offsite"))
                    .child(chip("night")),
            )
            .child(card("review"))
            .child(card("night"))
            .child(card("offsite")),
    ))
}
