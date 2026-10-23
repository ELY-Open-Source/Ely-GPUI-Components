use ely_gpui_component::calendar::{
    CalendarDayView, CalendarWeekView, Event, TimezoneOverlay, When,
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, tz::TimeZone};

use super::{here, plan, quiet};
use crate::{
    probe::probe,
    ui::{change, keep, section, set},
};

/// Adds an event from a drag and says so.
fn creating(
    events: &Entity<Vec<Event>>,
    said: &Entity<Option<SharedString>>,
) -> impl Fn(Timestamp, Timestamp, &mut Window, &mut App) + 'static {
    let (events, said) = (events.clone(), said.clone());
    move |start, end, _, cx| {
        change(&events, cx, |events| {
            let key = format!("made-{}", events.len());
            events.push(Event::timed(key, "New event", start, end, 4));
        });
        set(&said, Some("Made a new event.".into()), cx)
    }
}

/// Moves an event's end to where its edge was dragged.
fn resizing(
    events: &Entity<Vec<Event>>,
) -> impl Fn(&SharedString, Timestamp, &mut Window, &mut App) + 'static {
    let events = events.clone();
    move |key, end, _, cx| {
        change(&events, cx, |events| {
            let event = events
                .iter_mut()
                .find(|event| event.key == *key)
                .expect("a resized event is listed");
            let When::Timed { start, .. } = event.when else {
                panic!("event {key}: only timed events resize");
            };
            event.when = When::Timed { start, end };
        })
    }
}

pub fn week(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (zone, today) = here();
    let events = plan(window, cx);
    let said = keep("calendar-week-said", || None::<SharedString>, window, cx);
    let (shown, listed) = (said.read(cx).clone(), events.read(cx).clone());
    let (create, resize) = (creating(&events, &said), resizing(&events));
    let pressed = said;
    section(
        "CalendarWeekView / TimeGrid / AllDayRow / CurrentTimeIndicator",
        "A week under its days' names: whole days lie above, the rest by their hours, and events that overlap share the width. A drag on empty time makes an event of a quarter hour or more, and a drag on an event's lower edge moves its end. Today carries a line at the minute now.",
        cx,
    )
    .child(probe(
        "calendar-week",
        div()
            .w(px(800.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div().h(px(560.)).child(
                    CalendarWeekView::new("calendar-week", today, listed)
                        .zone(zone)
                        .on_create(create)
                        .on_resize(resize)
                        .on_event(move |key, _, cx| {
                            set(&pressed, Some(format!("Pressed {key}.").into()), cx)
                        }),
                ),
            )
            .children(shown.map(|shown| quiet(shown, cx))),
    ))
}

pub fn day(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (zone, today) = here();
    let events = plan(window, cx);
    let said = keep("calendar-day-said", || None::<SharedString>, window, cx);
    let listed = events.read(cx).clone();
    let (create, resize) = (creating(&events, &said), resizing(&events));
    let tokyo = TimeZone::get("Asia/Tokyo").expect("the system knows Tokyo");
    section(
        "CalendarDayView / TimezoneOverlay",
        "One day, with another zone's hours beside its own, so a call across zones reads in both.",
        cx,
    )
    .child(probe(
        "calendar-day",
        div().w(px(420.)).h(px(480.)).child(
            CalendarDayView::new("calendar-day", today, listed)
                .zone(zone)
                .overlay(TimezoneOverlay::new(tokyo))
                .on_create(create)
                .on_resize(resize),
        ),
    ))
}
