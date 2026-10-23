use std::time::Duration;

use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    theme::{ActiveTheme, TextSize},
    typography::{AnimatedNumber, GradientText, ShimmerText, Typewriter},
};
use gpui::{App, IntoElement, ParentElement, Styled, Task, Window, div};

use crate::ui::{live, row, section, specimen, specimens};

const STEP: Duration = Duration::from_millis(1600);
const VALUES: [f64; 5] = [1_284.0, 1_917.0, 12_406.0, 9_998.0, 10_031.0];

struct Ticker {
    index: usize,
    _task: Task<()>,
}

pub fn animated_number(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let index = if live("AnimatedNumber", cx) {
        ticker(window, cx).read(cx).index
    } else {
        0
    };
    let value = VALUES[index];
    section(
        "AnimatedNumber",
        "Digits roll by place; or the value counts up.",
        cx,
    )
    .child(
        specimens()
            .gap_12()
            .child(specimen("roll", AnimatedNumber::new("roll", value), cx))
            .child(specimen(
                "count up",
                AnimatedNumber::new("count", value * 3.7)
                    .decimals(1)
                    .count_up(),
                cx,
            )),
    )
}

/// Steps through the values on a clock of its own.
fn ticker(window: &mut Window, cx: &mut App) -> gpui::Entity<Ticker> {
    window.use_keyed_state("type-ticker", cx, |window, cx| Ticker {
        index: 0,
        _task: cx.spawn_in(window, async move |ticker, cx| {
            loop {
                cx.background_executor().timer(STEP).await;
                let advanced = cx.update(|_, cx| {
                    ticker.update(cx, |ticker: &mut Ticker, cx| {
                        ticker.index = (ticker.index + 1) % VALUES.len();
                        cx.notify();
                    })
                });
                if advanced.and_then(|inner| inner).is_err() {
                    return;
                }
            }
        }),
    })
}

pub fn typewriter(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let take = window.use_keyed_state("type-take", cx, |_, _| 0usize);
    let lines = [
        "Writing, one grapheme at a time.",
        "中文与 emoji ✨ 也按字形逐个出现。",
    ];
    let line = lines[*take.read(cx) % lines.len()];
    section(
        "Typewriter",
        "Reveals by grapheme, so emoji and CJK stay whole.",
        cx,
    )
    .child(
        row()
            .gap_6()
            .child(div().min_w_80().child(Typewriter::new("typewriter", line)))
            .child(
                Button::new("type-again", "Next line")
                    .variant(ButtonVariant::Ghost)
                    .on_click(move |_, _, cx| {
                        take.update(cx, |take, cx| {
                            *take += 1;
                            cx.notify();
                        })
                    }),
            ),
    )
}

pub fn gradient(cx: &App) -> impl IntoElement + use<> {
    let colors = &cx.theme().colors;
    let (ink, focus) = (colors.fg, colors.focus);
    let display = cx.theme().text_size(TextSize::Display);
    section("GradientText", "A tonal ramp, glyph by glyph. No neon.", cx).child(
        specimens()
            .gap_10()
            .child(specimen(
                "ink to muted",
                div()
                    .text_size(display)
                    .child(GradientText::new("Quiet type")),
                cx,
            ))
            .child(specimen(
                "ink to focus",
                div()
                    .text_size(display)
                    .child(GradientText::new("Quiet type").colors(ink, focus)),
                cx,
            )),
    )
}

pub fn shimmer(cx: &App) -> impl IntoElement + use<> {
    section(
        "ShimmerText",
        "A slow band of light while something loads.",
        cx,
    )
    .child(
        div()
            .text_size(cx.theme().text_size(TextSize::Lg))
            .child(ShimmerText::new("shimmer", "Thinking through the request…")),
    )
}
