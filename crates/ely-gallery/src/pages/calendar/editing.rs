use ely_gpui_component::{
    calendar::{AvailabilityPicker, EventDraft, EventEditor, Frequency, Recurrence, When},
    theme::{ActiveTheme, Radius},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{ToSpan, civil::Weekday};

use super::{here, plan, quiet};
use crate::{
    probe::probe,
    ui::{change, keep, section, set},
};

pub fn editor(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (zone, _) = here();
    let events = plan(window, cx);
    let review = events
        .read(cx)
        .iter()
        .find(|event| event.key.as_ref() == "review")
        .cloned();
    let said = keep("calendar-editor-said", || None::<SharedString>, window, cx);
    let shown = said.read(cx).clone();
    let theme = cx.theme();
    let body = match review {
        Some(review) => {
            let (saving, cancelling, deleting) = (said.clone(), said.clone(), said);
            let (kept, dropped) = (events.clone(), events);
            let draft = EventDraft {
                repeat: Some(Recurrence {
                    weekdays: vec![Weekday::Monday, Weekday::Thursday],
                    ..Recurrence::new(Frequency::Weekly)
                }),
                reminders: vec![10, 1440],
                ..EventDraft::of(&review)
            };
            EventEditor::new("calendar-editor", draft)
                .zone(zone)
                .on_save(move |draft, _, cx| {
                    let draft = draft.clone();
                    change(&kept, cx, |events| {
                        let event = events
                            .iter_mut()
                            .find(|event| event.key.as_ref() == "review")
                            .expect("the review is listed");
                        event.title = draft.title.clone();
                        event.when = draft.when;
                        event.hue = draft.hue;
                        event.place = (!draft.place.is_empty()).then(|| draft.place.clone());
                    });
                    set(&saving, Some(format!("Saved {}.", draft.title).into()), cx)
                })
                .on_cancel(move |_, cx| set(&cancelling, Some("Left as it was.".into()), cx))
                .on_delete(move |_, cx| {
                    change(&dropped, cx, |events| {
                        events.retain(|event| event.key.as_ref() != "review")
                    });
                    set(&deleting, Some("Deleted the review.".into()), cx)
                })
                .into_any_element()
        }
        None => quiet("The review was deleted.".into(), cx).into_any_element(),
    };
    section(
        "EventEditor / RecurrenceEditor / ReminderPicker",
        "An event's parts in a form: its title, whole days or hours, where, its color, how it repeats with the next days it falls on, its reminders and notes. Save waits for a title and an end after the start; saved, every view on the page shows the change.",
        cx,
    )
    .child(probe(
        "calendar-editor",
        div()
            .w(px(480.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .p_5()
                    .border_1()
                    .border_color(theme.colors.border)
                    .rounded(theme.radius(Radius::Lg))
                    .child(body),
            )
            .children(shown.map(|shown| quiet(shown, cx))),
    ))
}

pub fn availability(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (zone, today) = here();
    let busy: Vec<_> = plan(window, cx)
        .read(cx)
        .iter()
        .filter_map(|event| match event.when {
            When::Timed { start, end } => Some((start, end)),
            When::AllDay { .. } => None,
        })
        .collect();
    let days: Vec<_> = (1..=9)
        .map(|offset| {
            today
                .checked_add((offset as i64).days())
                .expect("a day ahead")
        })
        .filter(|day| !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday))
        .take(5)
        .collect();
    let picked = keep("calendar-slot", || None::<jiff::Timestamp>, window, cx);
    let chosen = *picked.read(cx);
    let said = chosen.map(|start| {
        let when = start
            .to_zoned(zone.clone())
            .strftime("%A at %H:%M")
            .to_string();
        SharedString::from(format!("Meet {when}."))
    });
    let picker = AvailabilityPicker::new("calendar-slots", days)
        .zone(zone)
        .busy(busy)
        .length(45)
        .on_pick(move |start, _, cx| set(&picked, Some(start), cx));
    let picker = match chosen {
        Some(start) => picker.selected(start),
        None => picker,
    };
    section(
        "AvailabilityPicker",
        "Times to meet over the working days ahead: each day with its count of open slots, then the slots of the one chosen. What the page's events take is closed.",
        cx,
    )
    .child(probe(
        "calendar-slots",
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_3()
            .child(picker)
            .children(said.map(|said| quiet(said, cx))),
    ))
}
