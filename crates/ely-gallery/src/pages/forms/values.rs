use ely_gpui_component::forms::{Choice, ChoiceChips, Knob, RangeSlider, Rating, Slider, Stepper};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set, specimen, specimens},
};

pub fn chips(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let sort = keep("chips-sort", || vec![SharedString::from("new")], window, cx);
    let teams = keep(
        "chips-teams",
        || vec![SharedString::from("design")],
        window,
        cx,
    );
    let (sort_now, teams_now) = (sort.read(cx).clone(), teams.read(cx).clone());
    section(
        "ChoiceChips / FilterChips",
        "Pills for a few options. Filter chips toggle and show a check.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(specimen(
                "choice",
                probe(
                    "chips",
                    ChoiceChips::new(
                        "chips-sort",
                        [
                            Choice::new("new", "Newest"),
                            Choice::new("top", "Popular"),
                            Choice::new("old", "Oldest"),
                        ],
                    )
                    .selected(sort_now)
                    .on_change(move |next, _, cx| set(&sort, next.to_vec(), cx)),
                ),
                cx,
            ))
            .child(specimen(
                "filter",
                ChoiceChips::new(
                    "chips-teams",
                    [
                        Choice::new("design", "Design"),
                        Choice::new("eng", "Engineering"),
                        Choice::new("research", "Research"),
                        Choice::new("ops", "Operations").disabled(),
                    ],
                )
                .multiple()
                .selected(teams_now)
                .on_change(move |next, _, cx| set(&teams, next.to_vec(), cx)),
                cx,
            )),
    )
}

pub fn rating(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let stars = keep("rating-stars", || 3u8, window, cx);
    let now = *stars.read(cx);
    section(
        "Rating",
        "Hover previews, a click sets it, and clicking the same star clears it.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                format!("{now} of 5"),
                probe(
                    "rating",
                    Rating::new("rating-stars", now)
                        .on_change(move |next, _, cx| set(&stars, next, cx)),
                ),
                cx,
            ))
            .child(specimen(
                "read only",
                Rating::new("rating-fixed", 4).disabled(true),
                cx,
            )),
    )
}

pub fn sliders(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let volume = keep("slider-volume", || 40.0, window, cx);
    let level = keep("slider-level", || 60.0, window, cx);
    let price = keep("slider-price", || (20.0, 80.0), window, cx);
    let (volume_now, level_now, (low, high)) = (*volume.read(cx), *level.read(cx), *price.read(cx));
    section(
        "Slider / RangeSlider / VerticalSlider",
        "Drag a thumb, click the track, or use the arrows; Page keys jump ten steps.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                format!("volume {volume_now:.0}"),
                probe(
                    "slider",
                    div().w(px(320.0)).child(
                        Slider::new("slider-volume", volume_now)
                            .on_change(move |next, _, cx| set(&volume, next, cx)),
                    ),
                ),
                cx,
            ))
            .child(specimen(
                format!("price {low:.0} to {high:.0}"),
                div().w(px(320.0)).child(
                    RangeSlider::new("slider-price", low, high)
                        .step(5.0)
                        .on_change(move |next, _, cx| set(&price, next, cx)),
                ),
                cx,
            ))
            .child(specimen(
                format!("level {level_now:.0}"),
                div().h(px(140.0)).child(
                    Slider::new("slider-level", level_now)
                        .vertical()
                        .step(5.0)
                        .on_change(move |next, _, cx| set(&level, next, cx)),
                ),
                cx,
            )),
    )
}

pub fn dials(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let gain = keep("knob-gain", || 65.0, window, cx);
    let pan = keep("knob-pan", || 0.0, window, cx);
    let guests = keep("stepper-guests", || 2.0, window, cx);
    let (gain_now, pan_now, guests_now) = (*gain.read(cx), *pan.read(cx), *guests.read(cx));
    section(
        "Knob / Dial / Stepper",
        "Drag a knob up or down, or use the arrows. The stepper counts without typing.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                format!("gain {gain_now:.0}"),
                probe(
                    "knob",
                    Knob::new("knob-gain", gain_now)
                        .on_change(move |next, _, cx| set(&gain, next, cx)),
                ),
                cx,
            ))
            .child(specimen(
                format!("pan {pan_now:.0}"),
                Knob::new("knob-pan", pan_now)
                    .range(-50.0, 50.0)
                    .on_change(move |next, _, cx| set(&pan, next, cx)),
                cx,
            ))
            .child(specimen(
                "guests",
                probe(
                    "stepper",
                    Stepper::new("stepper-guests", guests_now)
                        .range(1.0, 10.0)
                        .on_change(move |next, _, cx| set(&guests, next, cx)),
                ),
                cx,
            )),
    )
}
