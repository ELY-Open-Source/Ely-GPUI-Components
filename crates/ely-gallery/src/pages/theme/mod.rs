use std::time::Duration;

use ely_gpui_component::{
    motion,
    theme::{ActiveTheme, Elevation, Radius, TextSize},
};
use gpui::{
    Animation, AnimationExt, AnyElement, App, FontStyle, FontWeight, HighlightStyle, Hsla,
    IntoElement, ParentElement, SharedString, Styled, StyledText, Window, div, px,
};

mod editing;
mod importing;
mod platform;

use super::Page;
use crate::step::Step;
use crate::ui::{section, specimen, specimens};

pub const PAGE: Page = Page {
    number: 39,
    slug: "theme",
    title: "Theme",
    summary: "Tokens every component reads. Light and dark share one grammar.",
    render,
    script: &[
        Step::Click("frosted-open"),
        Step::Wait(700),
        Step::ShotWindow("frosted", "frosted-window"),
        Step::CloseWindow("frosted"),
    ],
};

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    let colors = cx.theme().colors.clone();
    let surfaces = [
        ("bg", colors.bg),
        ("surface", colors.surface),
        ("sunken", colors.sunken),
        ("hover", colors.hover),
        ("active", colors.active),
        ("border", colors.border),
        ("border strong", colors.border_strong),
    ];
    let inks = [
        ("fg", colors.fg),
        ("muted", colors.fg_muted),
        ("subtle", colors.fg_subtle),
        ("disabled", colors.fg_disabled),
        ("focus", colors.focus),
        ("link", colors.link),
    ];
    let signals = [
        ("success", colors.success, colors.success_subtle),
        ("warning", colors.warning, colors.warning_subtle),
        ("danger", colors.danger, colors.danger_subtle),
        ("info", colors.info, colors.info_subtle),
    ];

    div()
        .child(
            section(
                "Surfaces",
                "Warm neutrals. Depth comes from tone, not shadow.",
                cx,
            )
            .child(
                specimens()
                    .gap_3()
                    .children(surfaces.map(|(name, color)| swatch(name, color, cx))),
            ),
        )
        .child(
            section("Ink", "Four steps of text. Blue marks focus and links.", cx).child(
                specimens()
                    .gap_3()
                    .children(inks.map(|(name, color)| swatch(name, color, cx))),
            ),
        )
        .child(
            section("Signals", "Status colors, each with a quiet fill.", cx).child(
                specimens()
                    .gap_3()
                    .children(signals.map(|(name, strong, quiet)| {
                        specimen(name, signal_pair(strong, quiet, cx), cx)
                    })),
            ),
        )
        .child(
            section("Series", "Eight chart colors, even in weight.", cx).child(
                specimens()
                    .gap_2()
                    .children(colors.chart.iter().enumerate().map(|(ix, &color)| {
                        specimen(format!("{}", ix + 1), chip(color, 40.0, cx), cx)
                    })),
            ),
        )
        .child(
            section("Syntax", "Code tokens sit below text in weight.", cx).child(code_sample(cx)),
        )
        .child(
            section("Type", "Inter for words. JetBrains Mono for code.", cx).child(type_scale(cx)),
        )
        .child(
            section("Shape", "Four radii. Three elevations.", cx)
                .child(
                    specimens().children([Radius::Sm, Radius::Md, Radius::Lg, Radius::Xl].map(
                        |radius| {
                            specimen(
                                format!("{radius:?}"),
                                div()
                                    .size(px(64.0))
                                    .rounded(cx.theme().radius(radius))
                                    .bg(colors.sunken)
                                    .border_1()
                                    .border_color(colors.border_strong),
                                cx,
                            )
                        },
                    )),
                )
                .child(specimens().gap_8().pt_4().children(
                    [Elevation::Raised, Elevation::Floating, Elevation::Modal].map(|level| {
                        specimen(
                            format!("{level:?}"),
                            div()
                                .w(px(120.0))
                                .h(px(72.0))
                                .rounded(cx.theme().radius(Radius::Lg))
                                .bg(colors.overlay)
                                .shadow(cx.theme().elevation(level)),
                            cx,
                        )
                    }),
                )),
        )
        .child(
            section(
                "Motion",
                "Ease out to arrive. Springs only where a hand leads. Slowed to one second here.",
                cx,
            )
            .child(track(
                "ease-out",
                "ease out cubic",
                motion::ease_out_cubic,
                cx,
            ))
            .child(track(
                "ease-in-out",
                "ease in out cubic",
                motion::ease_in_out_cubic,
                cx,
            ))
            .child(track("spring", "spring · damping 0.75", motion::spring, cx)),
        )
        .child(editing::editor(window, cx))
        .child(importing::importing(window, cx))
        .child(platform::platform(window, cx))
        .into_any_element()
}

fn chip(color: Hsla, side: f32, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .w(px(side))
        .h(px(side))
        .rounded(theme.radius(Radius::Md))
        .bg(color)
        .border_1()
        .border_color(theme.colors.border)
}

fn swatch(name: &'static str, color: Hsla, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    specimen(
        name,
        div()
            .w(px(88.0))
            .h(px(56.0))
            .rounded(theme.radius(Radius::Lg))
            .bg(color)
            .border_1()
            .border_color(theme.colors.border),
        cx,
    )
}

fn signal_pair(strong: Hsla, quiet: Hsla, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .flex()
        .items_center()
        .gap_2()
        .h(px(40.0))
        .px_3()
        .rounded(theme.radius(Radius::Md))
        .bg(quiet)
        .child(div().size(px(8.0)).rounded_full().bg(strong))
        .child(
            div()
                .text_color(strong)
                .font_weight(FontWeight::MEDIUM)
                .child("Aa 12"),
        )
}

fn code_sample(cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let syntax = &theme.colors.syntax;
    let text = "fn answer() -> u32 { let x = 42; x } // truth";
    let color = |c: Hsla| HighlightStyle {
        color: Some(c),
        ..Default::default()
    };
    let spans = [
        (0..2, color(syntax.keyword)),
        (3..9, color(syntax.function)),
        (9..11, color(syntax.punctuation)),
        (12..14, color(syntax.operator)),
        (15..18, color(syntax.type_name)),
        (19..20, color(syntax.punctuation)),
        (21..24, color(syntax.keyword)),
        (25..26, color(syntax.variable)),
        (27..28, color(syntax.operator)),
        (29..31, color(syntax.number)),
        (31..32, color(syntax.punctuation)),
        (35..36, color(syntax.punctuation)),
        (
            37..45,
            HighlightStyle {
                color: Some(syntax.comment),
                font_style: Some(FontStyle::Italic),
                ..Default::default()
            },
        ),
    ];
    div()
        .px_4()
        .py_3()
        .rounded(theme.radius(Radius::Lg))
        .bg(theme.colors.sunken)
        .font_family(theme.mono_family.clone())
        .text_size(theme.text_size(TextSize::Base))
        .text_color(theme.colors.fg)
        .child(StyledText::new(text).with_highlights(spans))
}

fn type_scale(cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let steps = [
        (TextSize::Display, "Display 32", FontWeight::SEMIBOLD),
        (TextSize::Xxl, "Heading 24", FontWeight::SEMIBOLD),
        (TextSize::Xl, "Title 20", FontWeight::SEMIBOLD),
        (TextSize::Lg, "Subtitle 16", FontWeight::MEDIUM),
        (TextSize::Md, "Body large 14", FontWeight::NORMAL),
        (TextSize::Base, "Body 13", FontWeight::NORMAL),
        (TextSize::Sm, "Small 12", FontWeight::NORMAL),
        (TextSize::Xs, "Caption 11", FontWeight::NORMAL),
    ];
    div()
        .flex()
        .flex_col()
        .gap_3()
        .children(steps.map(|(size, name, weight)| {
            div()
                .flex()
                .items_center()
                .gap_6()
                .child(
                    div()
                        .w(px(120.0))
                        .text_size(theme.text_size(TextSize::Xs))
                        .text_color(theme.colors.fg_subtle)
                        .child(SharedString::from(name)),
                )
                .child(
                    div()
                        .text_size(theme.text_size(size))
                        .font_weight(weight)
                        .text_color(theme.colors.fg)
                        .child("Quiet interfaces age well."),
                )
        }))
}

fn track(
    id: &'static str,
    name: &'static str,
    ease: fn(f32) -> f32,
    cx: &App,
) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let width = 360.0;
    let travel = width - 12.0;
    let knob = div()
        .absolute()
        .top(px(6.0))
        .size(px(12.0))
        .rounded_full()
        .bg(theme.colors.fg);
    let knob = if theme.reduced_motion {
        knob.left(px(travel)).into_any_element()
    } else {
        knob.with_animation(
            id,
            Animation::new(Duration::from_millis(2400)).repeat(),
            move |knob, t| {
                let offset = match t {
                    t if t < 0.42 => ease(t / 0.42),
                    t if t < 0.5 => 1.0,
                    t if t < 0.92 => 1.0 - ease((t - 0.5) / 0.42),
                    _ => 0.0,
                };
                knob.left(px(offset * travel))
            },
        )
        .into_any_element()
    };
    specimen(
        name,
        div()
            .relative()
            .w(px(width))
            .h(px(24.0))
            .child(
                div()
                    .absolute()
                    .top(px(11.5))
                    .left_0()
                    .w(px(width))
                    .h(px(1.0))
                    .bg(theme.colors.border),
            )
            .child(knob),
        cx,
    )
}
