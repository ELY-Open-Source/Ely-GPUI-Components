use std::{path::Path, time::Duration};

use ely_gpui_component::{
    buttons::{Button, IconButton},
    primitives::{
        Backdrop, Divider, FocusRing, FocusScope, Icon, IconName, Image, IntersectionObserver,
        Measure, Pressable, Tooltip, TooltipTrigger,
    },
    theme::{ActiveTheme, IconSize, Radius, TextSize},
};
use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels, SharedString, Size,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px,
};

use crate::{
    assets::CUSTOM_ICON,
    probe::probe,
    ui::{UNREAD, blocked, code, picture, row, section, specimen, specimens},
};

const ATRIUM: &str = asset!("atrium.jpg");

fn counter(window: &mut Window, cx: &mut App, key: &'static str) -> gpui::Entity<u32> {
    window.use_keyed_state(key, cx, |_, _| 0u32)
}

pub fn icon(cx: &App) -> impl IntoElement + use<> {
    let colors = &cx.theme().colors;
    let sizes = [
        (IconSize::Xs, "12"),
        (IconSize::Sm, "14"),
        (IconSize::Md, "16"),
        (IconSize::Lg, "20"),
        (IconSize::Xl, "24"),
        (IconSize::Xxl, "32"),
    ];
    let tones = [
        (colors.fg, "fg"),
        (colors.fg_muted, "muted"),
        (colors.focus, "focus"),
        (colors.success, "success"),
        (colors.warning, "warning"),
        (colors.danger, "danger"),
    ];
    section(
        "Icon",
        "Bundled Lucide or an application's SVG asset, drawn as a themed mask.",
        cx,
    )
    .child(specimens().children(
        sizes.map(|(size, name)| specimen(name, Icon::new(IconName::Sparkles).size(size), cx)),
    ))
    .child(
        specimens().children(tones.map(|(color, name)| {
            specimen(name, Icon::new(IconName::CircleCheck).color(color), cx)
        })),
    )
    .child(
        specimens().child(specimen(
            "custom SVG",
            Icon::from_path(CUSTOM_ICON)
                .size(IconSize::Xl)
                .color(colors.accent),
            cx,
        )),
    )
}

pub fn icon_set(cx: &App) -> impl IntoElement + use<> {
    let colors = &cx.theme().colors;
    section(
        "IconSet / IconRegistry",
        "IconName lists every bundled icon. Hover for its name.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_1()
            .children(IconName::ALL.iter().map(|&name| {
                div()
                    .id(name.name())
                    .size_9()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .hover(|style| style.bg(colors.hover))
                    .tooltip(Tooltip::text(name.name()))
                    .child(Icon::new(name).color(colors.fg_muted))
            })),
    )
}

fn tile(image: Image, cx: &App) -> Image {
    image
        .w(px(198.0))
        .h(px(132.0))
        .rounded(cx.theme().radius(Radius::Lg))
}

pub fn image(cx: &App) -> impl IntoElement + use<> {
    section(
        "Image",
        "Loading pulses. Failure and emptiness show a quiet mark.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "loaded",
                tile(Image::new("img-atrium", picture(ATRIUM)), cx),
                cx,
            ))
            .child(specimen(
                "failed",
                tile(Image::new("img-missing", Path::new("/missing/ely.png")), cx),
                cx,
            ))
            .child(specimen(
                "empty",
                tile(Image::placeholder("img-empty"), cx),
                cx,
            )),
    )
}

pub fn divider(cx: &App) -> impl IntoElement + use<> {
    section(
        "Divider / Separator",
        "One hairline. Across, down, or holding a word.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_5()
            .w(px(420.0))
            .child(Divider::horizontal())
            .child(Divider::horizontal().label("or"))
            .child(
                row()
                    .h_6()
                    .child("Left")
                    .child(Divider::vertical())
                    .child("Right"),
            ),
    )
}

pub fn backdrop(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = window.use_keyed_state("prim-backdrop", cx, |_, _| false);
    let shown = *open.read(cx);
    let theme = cx.theme();
    let (show, dismiss, close) = (open.clone(), open.clone(), open);
    let set = |state: &gpui::Entity<bool>, value: bool, cx: &mut App| {
        state.update(cx, |open, cx| {
            *open = value;
            cx.notify();
        })
    };
    let card = div()
        .w(px(320.0))
        .p_5()
        .flex()
        .flex_col()
        .gap_3()
        .rounded(theme.radius(Radius::Xl))
        .bg(theme.colors.overlay)
        .shadow(theme.elevation(ely_gpui_component::theme::Elevation::Modal))
        .child(div().font_weight(FontWeight::SEMIBOLD).child("Backdrop"))
        .child(
            div()
                .text_color(theme.colors.fg_muted)
                .child("Click outside to close."),
        )
        .child(div().flex().justify_end().child(
            Button::new("backdrop-close", "Close").on_click(move |_, _, cx| set(&close, false, cx)),
        ));
    section(
        "Overlay / Backdrop",
        "A scrim that fades in, blocks the page, and hears outside clicks.",
        cx,
    )
    .child(
        row()
            .child(probe(
                "backdrop-open",
                Button::new("backdrop-open", "Show backdrop")
                    .on_click(move |_, _, cx| set(&show, true, cx)),
            ))
            .when(shown, |row| {
                row.child(
                    Backdrop::new("prim-backdrop-layer")
                        .on_dismiss(move |_, cx| set(&dismiss, false, cx))
                        .child(card),
                )
            }),
    )
}

pub fn visually_hidden(cx: &App) -> impl IntoElement + use<> {
    section("VisuallyHidden", "Text for screen readers only.", cx)
        .child(blocked(format!("Not yet shown: {UNREAD}."), cx))
}

fn chip(id: &'static str, label: &'static str, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .id(id)
        .tab_index(0)
        .px_3()
        .py_1p5()
        .rounded(theme.radius(Radius::Md))
        .border_1()
        .border_color(theme.colors.border)
        .focus_ring(cx)
        .child(label)
}

pub fn focus_ring(cx: &App) -> impl IntoElement + use<> {
    section(
        "FocusRing",
        "Any focusable element. Press Tab to walk these.",
        cx,
    )
    .child(
        row()
            .child(chip("ring-a", "First", cx))
            .child(chip("ring-b", "Second", cx))
            .child(chip("ring-c", "Third", cx))
            .child(code(".focus_ring(cx)", cx)),
    )
}

pub fn focus_trap(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let handle = window
        .use_keyed_state("prim-trap", cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let theme = cx.theme();
    section(
        "FocusScope / FocusTrap",
        "Tab into the box. Tab then circles inside it.",
        cx,
    )
    .child(
        FocusScope::new(&handle).trap().flex().child(
            row()
                .p_3()
                .rounded(theme.radius(Radius::Lg))
                .border_1()
                .border_color(theme.colors.border_strong)
                .child(chip("trap-a", "One", cx))
                .child(chip("trap-b", "Two", cx))
                .child(probe("trap-c", chip("trap-c", "Three", cx))),
        ),
    )
}

pub fn pressable(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let taps = counter(window, cx, "prim-taps");
    let holds = counter(window, cx, "prim-holds");
    let (tap_count, hold_count) = (*taps.read(cx), *holds.read(cx));
    let bump = |count: &gpui::Entity<u32>, cx: &mut App| {
        count.update(cx, |count, cx| {
            *count += 1;
            cx.notify();
        })
    };
    section(
        "Pressable",
        "Knows when it is held. Hold half a second for a long press.",
        cx,
    )
    .child(
        row()
            .gap_6()
            .child(probe(
                "prim-press",
                Pressable::new("prim-press", |pressed, _, cx| {
                    let theme = cx.theme();
                    div()
                        .w(px(160.0))
                        .h_16()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(theme.radius(Radius::Lg))
                        .bg(if pressed {
                            theme.colors.active
                        } else {
                            theme.colors.sunken
                        })
                        .border_1()
                        .border_color(theme.colors.border)
                        .child(if pressed { "Holding" } else { "Press" })
                        .into_any_element()
                })
                .on_press(move |_, cx| bump(&taps, cx))
                .on_long_press(move |_, cx| bump(&holds, cx)),
            ))
            .child(code(format!("taps {tap_count} · holds {hold_count}"), cx)),
    )
}

pub fn measure(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let size = window.use_keyed_state("prim-size", cx, |_, _| None::<Size<Pixels>>);
    let wide = window.use_keyed_state("prim-wide", cx, |_, _| false);
    let (measured, is_wide) = (*size.read(cx), *wide.read(cx));
    let theme = cx.theme();
    let readout = match measured {
        Some(size) => format!(
            "{:.0} × {:.0}",
            f32::from(size.width),
            f32::from(size.height)
        ),
        None => "measuring…".to_string(),
    };
    section(
        "Measure",
        "Reports bounds when they change. This is also the ResizeObserver.",
        cx,
    )
    .child(
        row()
            .child(
                Button::new("measure-toggle", if is_wide { "Narrow" } else { "Widen" }).on_click(
                    move |_, _, cx| {
                        wide.update(cx, |wide, cx| {
                            *wide = !*wide;
                            cx.notify();
                        })
                    },
                ),
            )
            .child(code(readout, cx)),
    )
    .child(
        Measure::new("prim-measured", move |bounds, _, cx| {
            size.update(cx, |size, cx| {
                *size = Some(bounds.size);
                cx.notify();
            })
        })
        .w(px(if is_wide { 420.0 } else { 220.0 }))
        .p_3()
        .rounded(theme.radius(Radius::Lg))
        .bg(theme.colors.sunken)
        .text_color(theme.colors.fg_muted)
        .child("Text wraps to the width it gets. The box reports each new size once."),
    )
}

pub fn intersection(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let seen = window.use_keyed_state("prim-seen", cx, |_, _| false);
    let visible = *seen.read(cx);
    let theme = cx.theme();
    let spacer = || div().h(px(140.0));
    let status = if visible { "in view" } else { "out of view" };
    section(
        "IntersectionObserver",
        "Scroll the box. The marker reports entering and leaving.",
        cx,
    )
    .child(
        row()
            .gap_6()
            .child(
                div()
                    .id("prim-scroller")
                    .w(px(260.0))
                    .h(px(120.0))
                    .overflow_y_scroll()
                    .rounded(theme.radius(Radius::Lg))
                    .border_1()
                    .border_color(theme.colors.border)
                    .child(spacer())
                    .child(
                        IntersectionObserver::new("prim-marker", move |visible, _, cx| {
                            seen.update(cx, |seen, cx| {
                                *seen = visible;
                                cx.notify();
                            })
                        })
                        .mx_3()
                        .p_2()
                        .rounded(theme.radius(Radius::Md))
                        .bg(theme.colors.info_subtle)
                        .text_color(theme.colors.info)
                        .child("Marker"),
                    )
                    .child(spacer()),
            )
            .child(code(status, cx)),
    )
}

pub fn tooltip(cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let muted = theme.colors.tooltip_fg.opacity(0.7);
    let lane = div()
        .w(px(260.0))
        .h_12()
        .flex()
        .items_center()
        .justify_center()
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border)
        .text_color(theme.colors.fg_muted)
        .child("Move across me");
    section(
        "Tooltip Trigger",
        "gpui's trigger with Ely's body, or Ely's own trigger for delay and tracking.",
        cx,
    )
    .child(
        row()
            .gap_2()
            .child(probe(
                "tip-plain",
                div()
                    .id("tip-plain")
                    .child(IconButton::new("tip-a", IconName::Copy))
                    .tooltip(Tooltip::text("Copy")),
            ))
            .child(probe(
                "tip-meta",
                div()
                    .id("tip-meta")
                    .child(IconButton::new("tip-b", IconName::Search))
                    .tooltip(Tooltip::with_meta("Search", "⌘K")),
            ))
            .child(probe(
                "tip-rich",
                div()
                    .id("tip-rich")
                    .child(IconButton::new("tip-c", IconName::Info))
                    .tooltip(Tooltip::rich(move |_, _| {
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(div().font_weight(FontWeight::MEDIUM).child("Rich content"))
                            .child(div().text_color(muted).child("Any element fits here."))
                            .into_any_element()
                    })),
            ))
            .child(div().w_4())
            .child(probe(
                "tip-follow",
                TooltipTrigger::new("tip-follow", lane, |_, _| {
                    SharedString::from("No delay. I follow.").into_any_element()
                })
                .delay(Duration::ZERO)
                .follow_mouse(),
            )),
    )
    .child(
        div()
            .text_size(theme.text_size(TextSize::Xs))
            .text_color(theme.colors.fg_subtle)
            .child("Ely's trigger waits 400ms by default; gpui's waits 500ms."),
    )
}
