use ely_gpui_component::{
    layout::{ScrollArea, ScrollShadow, ScrollToTop, Scrollbar, StickyHeader},
    theme::{ActiveTheme, Radius, TextSize},
};
use gpui::{
    App, Axis, FontWeight, InteractiveElement, IntoElement, ParentElement, ScrollHandle,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::{probe::probe, ui::section};

fn lines(count: usize, cx: &App) -> impl Iterator<Item = gpui::Div> + '_ {
    let theme = cx.theme();
    (1..=count).map(move |ix| {
        div()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(theme.colors.border)
            .text_size(theme.text_size(TextSize::Sm))
            .child(SharedString::from(format!(
                "Line {ix:02} · quiet content that scrolls"
            )))
    })
}

fn well(cx: &App) -> gpui::Div {
    let theme = cx.theme();
    div()
        .relative()
        .w(px(320.0))
        .h(px(200.0))
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border)
        .overflow_hidden()
}

pub fn scroll_area(cx: &App) -> impl IntoElement + use<> {
    section(
        "ScrollArea",
        "Overlay scrollbars appear while scrolling or hovered.",
        cx,
    )
    .child(probe(
        "scroll-area",
        well(cx).child(
            ScrollArea::new("scroll-area")
                .size_full()
                .children(lines(24, cx)),
        ),
    ))
}

fn handle(window: &mut Window, cx: &mut App, key: &'static str) -> ScrollHandle {
    window
        .use_keyed_state(key, cx, |_, _| ScrollHandle::new())
        .read(cx)
        .clone()
}

pub fn scrollbar(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let scroll = handle(window, cx, "bar-handle");
    section(
        "Scrollbar",
        "Any ScrollHandle can wear one. Drag the thumb.",
        cx,
    )
    .child(probe(
        "scrollbar",
        well(cx)
            .child(
                div()
                    .id("bar-body")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .children(lines(30, cx)),
            )
            .child(Scrollbar::new("bar", &scroll, Axis::Vertical)),
    ))
}

pub fn scroll_shadow(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let scroll = handle(window, cx, "shadow-handle");
    section(
        "ScrollShadow",
        "A soft edge wherever more content waits.",
        cx,
    )
    .child(
        well(cx)
            .child(
                div()
                    .id("shadow-body")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .children(lines(30, cx)),
            )
            .child(ScrollShadow::new(&scroll)),
    )
}

pub fn scroll_to_top(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let scroll = handle(window, cx, "top-handle");
    section(
        "ScrollToTop",
        "Past half a screen, a button eases back up.",
        cx,
    )
    .child(
        well(cx)
            .child(
                div()
                    .id("top-body")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .children(lines(40, cx)),
            )
            .child(ScrollToTop::new("to-top", &scroll)),
    )
}

pub fn sticky_header(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let scroll = handle(window, cx, "sticky-handle");
    let groups = ["Today", "Yesterday", "Earlier"];
    section(
        "StickyHeader",
        "Each group's header pins until the next pushes it off.",
        cx,
    )
    .child(probe(
        "sticky",
        well(cx)
            .child(
                div()
                    .id("sticky-body")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .children(groups.iter().enumerate().map(|(ix, group)| {
                        let title = SharedString::from(*group);
                        StickyHeader::new(("sticky", ix), &scroll, move |_, cx| {
                            let theme = cx.theme();
                            div()
                                .px_3()
                                .py_1p5()
                                .bg(theme.colors.sunken)
                                .border_b_1()
                                .border_color(theme.colors.border)
                                .text_size(theme.text_size(TextSize::Xs))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.colors.fg_muted)
                                .child(title.clone())
                                .into_any_element()
                        })
                        .children(lines(8, cx))
                    })),
            )
            .child(Scrollbar::new("sticky-bar", &scroll, Axis::Vertical)),
    ))
}
