use ely_gpui_component::{
    buttons::{SegmentedControl, ToggleButton, ToggleGroup, ToggleItem},
    primitives::IconName,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{row, section, specimen, specimens},
};

pub fn toggle_button(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let pinned = window.use_keyed_state("toggle-pinned", cx, |_, _| true);
    let muted = window.use_keyed_state("toggle-muted", cx, |_, _| false);
    let (pin_on, mute_on) = (*pinned.read(cx), *muted.read(cx));
    section("ToggleButton", "Stays pressed until pressed again.", cx).child(
        row()
            .child(
                ToggleButton::new(
                    "toggle-pin",
                    ToggleItem::new("pin").label("Pinned").icon(IconName::Pin),
                    pin_on,
                )
                .on_toggle(move |on, _, cx| {
                    pinned.update(cx, |pinned, cx| {
                        *pinned = on;
                        cx.notify();
                    })
                }),
            )
            .child(
                ToggleButton::new(
                    "toggle-mute",
                    ToggleItem::new("mute")
                        .icon(IconName::VolumeX)
                        .tooltip("Mute"),
                    mute_on,
                )
                .on_toggle(move |on, _, cx| {
                    muted.update(cx, |muted, cx| {
                        *muted = on;
                        cx.notify();
                    })
                }),
            ),
    )
}

fn choice(
    key: &'static str,
    initial: &'static [&'static str],
    window: &mut Window,
    cx: &mut App,
) -> gpui::Entity<Vec<SharedString>> {
    window.use_keyed_state(key, cx, |_, _| {
        initial.iter().map(|v| SharedString::from(*v)).collect()
    })
}

pub fn toggle_group(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let marks = choice("toggle-marks", &["bold"], window, cx);
    let align = choice("toggle-align", &["left"], window, cx);
    let (marks_now, align_now) = (marks.read(cx).clone(), align.read(cx).clone());
    let store = |state: gpui::Entity<Vec<SharedString>>| {
        move |values: &[SharedString], _: &mut Window, cx: &mut App| {
            let values = values.to_vec();
            state.update(cx, |state, cx| {
                *state = values;
                cx.notify();
            })
        }
    };
    section(
        "ToggleGroup",
        "Several at once for marks; one at a time for alignment.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "multiple",
                ToggleGroup::new("marks")
                    .multiple()
                    .item(ToggleItem::new("bold").icon(IconName::Bold).tooltip("Bold"))
                    .item(
                        ToggleItem::new("italic")
                            .icon(IconName::Italic)
                            .tooltip("Italic"),
                    )
                    .item(
                        ToggleItem::new("underline")
                            .icon(IconName::Underline)
                            .tooltip("Underline"),
                    )
                    .selected(marks_now)
                    .on_change(store(marks)),
                cx,
            ))
            .child(specimen(
                "single",
                ToggleGroup::new("align")
                    .item(
                        ToggleItem::new("left")
                            .label("Left")
                            .icon(IconName::AlignLeft),
                    )
                    .item(
                        ToggleItem::new("center")
                            .label("Center")
                            .icon(IconName::AlignCenter),
                    )
                    .item(
                        ToggleItem::new("right")
                            .label("Right")
                            .icon(IconName::AlignRight),
                    )
                    .selected(align_now)
                    .on_change(store(align)),
                cx,
            )),
    )
}

pub fn segmented(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let range = window.use_keyed_state("segment-range", cx, |_, _| SharedString::from("day"));
    let view = window.use_keyed_state("segment-view", cx, |_, _| SharedString::from("list"));
    let (range_now, view_now) = (range.read(cx).clone(), view.read(cx).clone());
    let store = |state: gpui::Entity<SharedString>| {
        move |value: &SharedString, _: &mut Window, cx: &mut App| {
            let value = value.clone();
            state.update(cx, |state, cx| {
                *state = value;
                cx.notify();
            })
        }
    };
    section(
        "SegmentedControl",
        "One of a few. The thumb springs to the choice.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "text",
                probe(
                    "segmented",
                    div().w(px(300.0)).child(
                        SegmentedControl::new("range", range_now)
                            .segment("day", "Day", None)
                            .segment("week", "Week", None)
                            .segment("month", "Month", None)
                            .on_change(store(range)),
                    ),
                ),
                cx,
            ))
            .child(specimen(
                "with icons",
                SegmentedControl::new("view", view_now)
                    .segment("list", "List", Some(IconName::List))
                    .segment("grid", "Grid", Some(IconName::LayoutGrid))
                    .on_change(store(view)),
                cx,
            )),
    )
}
