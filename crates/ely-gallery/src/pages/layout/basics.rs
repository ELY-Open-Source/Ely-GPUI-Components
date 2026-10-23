use ely_gpui_component::{
    layout::{AspectRatio, Container, Masonry, SimpleGrid, ZStack, h_stack, v_stack},
    theme::{ActiveTheme, ContainerSize, Radius, TextSize},
    typography::Caption,
};
use gpui::{App, Div, Hsla, IntoElement, ParentElement, SharedString, Styled, div, px};

use crate::ui::{code, section, specimen, specimens};

pub fn tile(label: impl Into<SharedString>, cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .flex()
        .items_center()
        .justify_center()
        .px_3()
        .py_2()
        .rounded(theme.radius(Radius::Md))
        .bg(theme.colors.sunken)
        .border_1()
        .border_color(theme.colors.border)
        .text_size(theme.text_size(TextSize::Sm))
        .text_color(theme.colors.fg_muted)
        .child(label.into())
}

fn outline(cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border)
}

pub fn stacks(cx: &App) -> impl IntoElement + use<> {
    section(
        "Stack / HStack / VStack",
        "h_stack() and v_stack(): flex rows and columns.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "h_stack",
                h_stack()
                    .gap_2()
                    .child(tile("A", cx))
                    .child(tile("B", cx))
                    .child(tile("C", cx)),
                cx,
            ))
            .child(specimen(
                "v_stack",
                v_stack().gap_2().child(tile("A", cx)).child(tile("B", cx)),
                cx,
            )),
    )
}

pub fn flex(cx: &App) -> impl IntoElement + use<> {
    section("Flex", "gpui's .flex(), with grow and justify.", cx).child(
        outline(cx)
            .p_2()
            .w(px(420.0))
            .flex()
            .gap_2()
            .child(tile("fixed", cx))
            .child(tile("flex_1", cx).flex_1())
            .child(tile("fixed", cx)),
    )
}

pub fn grid(cx: &App) -> impl IntoElement + use<> {
    section("Grid / GridItem", "gpui's .grid() and .col_span().", cx).child(
        div()
            .w(px(420.0))
            .grid()
            .grid_cols(3)
            .gap_2()
            .child(tile("span 2", cx).col_span(2))
            .child(tile("1", cx))
            .child(tile("1", cx))
            .child(tile("span 2", cx).col_span(2)),
    )
}

pub fn simple_grid(cx: &App) -> impl IntoElement + use<> {
    section(
        "SimpleGrid",
        "As many equal columns as fit at a minimum width.",
        cx,
    )
    .child(
        SimpleGrid::new("simple-grid", px(140.0), px(12.0))
            .max_w(px(720.0))
            .children((1..=7).map(|ix| tile(format!("Item {ix}"), cx).h(px(56.0)))),
    )
}

pub fn masonry(cx: &App) -> impl IntoElement + use<> {
    let heights = [72.0, 120.0, 56.0, 96.0, 140.0, 64.0, 88.0, 110.0];
    section("Masonry", "Each item joins the shortest column.", cx).child(
        div().w(px(560.0)).child(
            Masonry::new("masonry", 3, px(10.0)).children(
                heights
                    .iter()
                    .enumerate()
                    .map(|(ix, height)| tile(format!("{}", ix + 1), cx).h(px(*height))),
            ),
        ),
    )
}

pub fn center(cx: &App) -> impl IntoElement + use<> {
    section("Center", "flex, items_center, justify_center.", cx).child(
        outline(cx)
            .w(px(280.0))
            .h(px(96.0))
            .flex()
            .items_center()
            .justify_center()
            .child(tile("centered", cx)),
    )
}

pub fn container(cx: &App) -> impl IntoElement + use<> {
    section(
        "Container",
        "A centered column capped by a width token.",
        cx,
    )
    .child(
        outline(cx).w_full().py_3().child(
            Container::new(ContainerSize::Sm).child(
                div()
                    .h(px(40.0))
                    .rounded(cx.theme().radius(Radius::Md))
                    .bg(cx.theme().colors.sunken),
            ),
        ),
    )
    .child(code("Container::new(ContainerSize::Sm)", cx))
}

pub fn aspect_ratio(cx: &App) -> impl IntoElement + use<> {
    let fill = |ratio: &str, cx: &App| {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(cx.theme().colors.sunken)
            .text_color(cx.theme().colors.fg_subtle)
            .child(SharedString::from(ratio.to_string()))
    };
    section("AspectRatio", "Width fills; height follows the ratio.", cx).child(
        specimens()
            .child(
                div().w(px(240.0)).child(
                    AspectRatio::new(16.0 / 9.0)
                        .rounded(cx.theme().radius(Radius::Lg))
                        .child(fill("16 : 9", cx)),
                ),
            )
            .child(
                div().w(px(120.0)).child(
                    AspectRatio::new(1.0)
                        .rounded(cx.theme().radius(Radius::Lg))
                        .child(fill("1 : 1", cx)),
                ),
            ),
    )
}

pub fn inset(cx: &App) -> impl IntoElement + use<> {
    section("Inset / Padding", "gpui's .p_*() on the rem scale.", cx).child(
        specimens()
            .child(specimen(
                "p_2",
                outline(cx).p_2().child(tile("content", cx)),
                cx,
            ))
            .child(specimen(
                "p_6",
                outline(cx).p_6().child(tile("content", cx)),
                cx,
            )),
    )
}

pub fn wrap(cx: &App) -> impl IntoElement + use<> {
    let tags = [
        "design", "motion", "type", "color", "layout", "grid", "rhythm", "space",
    ];
    section(
        "Wrap",
        "gpui's .flex_wrap() moves overflow to the next line.",
        cx,
    )
    .child(
        outline(cx)
            .p_3()
            .w(px(320.0))
            .flex()
            .flex_wrap()
            .gap_2()
            .children(tags.map(|tag| tile(tag, cx))),
    )
}

pub fn absolute(cx: &App) -> impl IntoElement + use<> {
    section(
        "Absolute / Positioned",
        "gpui's .absolute() with edge offsets.",
        cx,
    )
    .child(
        outline(cx)
            .relative()
            .w(px(280.0))
            .h(px(112.0))
            .child(tile("top left", cx).absolute().top_2().left_2())
            .child(tile("bottom right", cx).absolute().bottom_2().right_2()),
    )
}

pub fn z_stack(cx: &App) -> impl IntoElement + use<> {
    let colors = &cx.theme().colors;
    let layer = |color: Hsla, inset: f32| {
        div()
            .size_full()
            .p(px(inset))
            .child(div().size_full().rounded_md().bg(color))
    };
    section(
        "Layer / ZStack",
        "The first child sets the size; the rest cover it.",
        cx,
    )
    .child(
        ZStack::new()
            .w(px(200.0))
            .h(px(120.0))
            .child(div().size_full().rounded_lg().bg(colors.sunken))
            .child(layer(colors.hover, 16.0))
            .child(layer(colors.active, 32.0))
            .child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Caption::new("three layers")),
            ),
    )
}
