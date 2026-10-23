use ely_gpui_component::{
    mail::{Label, LabelPicker, SnoozePicker},
    theme::{ActiveTheme, Radius},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::{day_and_hour, quiet};
use crate::{
    probe::probe,
    ui::{change, keep, section, set},
};

/// The demo's labels and the keys on.
#[derive(Clone)]
struct Sorting {
    labels: Vec<Label>,
    on: Vec<SharedString>,
}

pub fn labels(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let sorting = keep(
        "mail-labels",
        || Sorting {
            labels: vec![
                Label::new("clients", "Clients", 0),
                Label::new("studio", "Studio", 5),
                Label::new("press", "Press", 3),
                Label::new("finance", "Finance", 1),
                Label::new("site", "Site visits", 2),
                Label::new("travel", "Travel", 7),
            ],
            on: vec!["clients".into(), "press".into()],
        },
        window,
        cx,
    );
    let now = sorting.read(cx).clone();
    let names: Vec<&str> = now
        .labels
        .iter()
        .filter(|label| now.on.contains(&label.key))
        .map(|label| label.name.as_ref())
        .collect();
    let said = match names.len() {
        0 => SharedString::from("No labels on."),
        _ => format!("On: {}.", names.join(", ")).into(),
    };
    let (changing, making) = (sorting.clone(), sorting);
    let theme = cx.theme();
    section(
        "LabelPicker",
        "The labels a message wears: the field finds them by name, arrows move, and a press or Enter turns one on or off. A name no label has can be made into one, on at once.",
        cx,
    )
    .child(probe(
        "mail-labels",
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .w(px(300.))
                    .p_2()
                    .border_1()
                    .border_color(theme.colors.border)
                    .rounded(theme.radius(Radius::Lg))
                    .child(
                        LabelPicker::new("mail-labels", now.labels.clone())
                            .selected(now.on.clone())
                            .on_change(move |keys, _, cx| {
                                change(&changing, cx, |sorting| sorting.on = keys.to_vec())
                            })
                            .on_create(move |name, _, cx| {
                                let hues = cx.theme().colors.chart.len();
                                change(&making, cx, |sorting| {
                                    let made = sorting.labels.len();
                                    let key = SharedString::from(format!("made-{made}"));
                                    let free = (0..hues).find(|hue| {
                                        sorting.labels.iter().all(|label| label.hue != *hue)
                                    });
                                    let hue = free.unwrap_or(made % hues);
                                    sorting.labels.push(Label::new(key.clone(), name.clone(), hue));
                                    sorting.on.push(key);
                                })
                            }),
                    ),
            )
            .child(quiet(said, cx)),
    ))
}

pub fn snooze(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let said = keep("mail-snoozed", || None::<SharedString>, window, cx);
    let shown = said.read(cx).clone();
    section(
        "SnoozePicker",
        "Put a message off until later today, tomorrow, this weekend or next week, each with its day and hour, or until a time of your own.",
        cx,
    )
    .child(probe(
        "mail-snooze",
        div()
            .flex()
            .items_center()
            .gap_4()
            .child(SnoozePicker::new("mail-snooze").on_snooze(move |at, _, cx| {
                set(&said, Some(format!("Back {}.", day_and_hour(at)).into()), cx)
            }))
            .children(shown.map(|shown| quiet(shown, cx))),
    ))
}
