use ely_gpui_component::calendar::{AgendaView, Answer, Attendee, AttendeeList, EventPopover};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::{here, plan, quiet};
use crate::{
    probe::probe,
    ui::{change, keep, section, set},
};

fn guests() -> [Attendee; 5] {
    [
        Attendee::new("Chloé Martin", "chloe@atrium.studio", Answer::Going).organizer(),
        Attendee::new("Ana Lima", "ana@atrium.studio", Answer::Going),
        Attendee::new("Ben Ito", "ben@atrium.studio", Answer::Maybe),
        Attendee::new("Dev Rao", "dev@atrium.studio", Answer::Waiting),
        Attendee::new("Eli Park", "eli@atrium.studio", Answer::Declined),
    ]
}

pub fn agenda(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (zone, today) = here();
    let listed = plan(window, cx).read(cx).clone();
    let said = keep("calendar-agenda-said", || None::<SharedString>, window, cx);
    let shown = said.read(cx).clone();
    section(
        "AgendaView",
        "The days ahead with their events under each day's name, whole days first, then by the hour; days with nothing are left out. A press or an arrow selects, Enter or a double press opens.",
        cx,
    )
    .child(probe(
        "calendar-agenda",
        div()
            .w(px(420.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                AgendaView::new("calendar-agenda", today, 7, listed)
                    .zone(zone)
                    .on_event(move |key, _, cx| set(&said, Some(format!("Opened {key}.").into()), cx)),
            )
            .children(shown.map(|shown| quiet(shown, cx))),
    ))
}

pub fn details(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (zone, _) = here();
    let events = plan(window, cx);
    let review = events
        .read(cx)
        .iter()
        .find(|event| event.key.as_ref() == "review")
        .cloned()
        .expect("the demo holds the review");
    let said = keep("calendar-popover-said", || None::<SharedString>, window, cx);
    let shown = said.read(cx).clone();
    let (edited, dropped) = (said.clone(), said);
    section(
        "EventPopover / AttendeeList",
        "An event's chip opens its details: when and where, who is asked and what each said, and what can be done with it. The list of guests stands on its own too.",
        cx,
    )
    .child(probe(
        "calendar-popover",
        div()
            .flex()
            .items_start()
            .gap_8()
            .child(
                div()
                    .w(px(200.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        EventPopover::new("calendar-popover", review)
                            .zone(zone)
                            .attendees(guests())
                            .on_edit(move |key, _, cx| {
                                set(&edited, Some(format!("Edit {key}.").into()), cx)
                            })
                            .on_delete(move |key, _, cx| {
                                change(&events, cx, |events| events.retain(|event| event.key != *key));
                                set(&dropped, Some(format!("Deleted {key}.").into()), cx)
                            }),
                    )
                    .children(shown.map(|shown| quiet(shown, cx))),
            )
            .child(div().w(px(360.)).child(AttendeeList::new("calendar-guests", guests()))),
    ))
}
