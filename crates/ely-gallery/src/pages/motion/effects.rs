use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    feedback::InlineMessage,
    motion::{
        AnimatedGradient, Blink, Confetti, Flash, Glow, Marquee, ParticleBackground, Pulse, Ripple,
        Shake,
    },
    primitives::Severity,
    theme::{ActiveTheme, Radius, TextSize},
    typography::tabular,
};
use gpui::{
    App, FontWeight, IntoElement, ParentElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::{
    probe::probe,
    ui::{blocked, keep, row, section, set, specimen, specimens},
};

pub fn marquee(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let (muted, size) = (theme.colors.fg_muted, theme.text_size(TextSize::Sm));
    section(
        "Marquee",
        "A line that runs past at an even pace, with no seam where it loops.",
        cx,
    )
    .child(
        Marquee::new("marquee", move || {
            div()
                .flex()
                .gap_8()
                .pr_8()
                .text_size(size)
                .text_color(muted)
                .children([
                    "Designed in the open",
                    "Built in Rust",
                    "Light and dark",
                    "Motion with restraint",
                ])
        })
        .w_128()
        .py_2()
        .border_t_1()
        .border_b_1()
        .border_color(theme.colors.border),
    )
}

pub fn ripple(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let bursts = keep("confetti-bursts", || 0usize, window, cx);
    let (now, more) = (*bursts.read(cx), bursts.clone());
    let theme = cx.theme();
    section(
        "Ripple / Confetti",
        "A circle of light spreads from each press, cut to the card's corners. A burst of paper for a real win, in the chart's quiet colors.",
        cx,
    )
    .child(
        row()
            .items_start()
            .child(probe(
                "ripple",
                Ripple::new("ripple")
                    .w_64()
                    .h_32()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(theme.radius(Radius::Lg))
                    .border_1()
                    .border_color(theme.colors.border)
                    .bg(theme.colors.surface)
                    .child("Press anywhere"),
            ))
            .child(
                div()
                    .relative()
                    .w_64()
                    .h_32()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(probe(
                        "celebrate",
                        Button::new("celebrate", "Celebrate")
                            .primary()
                            .on_click(move |_, _, cx| set(&more, now + 1, cx)),
                    ))
                    .child(Confetti::new("confetti", now)),
            ),
    )
}

pub fn light(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    section(
        "Glow / Pulse",
        "A soft light that breathes around a card; rings that call out a live dot.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "glow",
                Glow::new("glow")
                    .px_5()
                    .py_4()
                    .rounded(theme.radius(Radius::Lg))
                    .border_1()
                    .border_color(theme.colors.border)
                    .bg(theme.colors.surface)
                    .child("Focus mode"),
                cx,
            ))
            .child(specimen(
                "pulse",
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Pulse::new("pulse")
                            .size_2()
                            .rounded_full()
                            .bg(theme.colors.success),
                    )
                    .child("Live"),
                cx,
            )),
    )
}

pub fn cues(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let price = keep("flash-price", || 1284u32, window, cx);
    let tries = keep("shake-tries", || 0usize, window, cx);
    let (value, tick) = (*price.read(cx), price.clone());
    let (count, again) = (*tries.read(cx), tries.clone());
    let theme = cx.theme();
    section(
        "Blink / Flash / Shake",
        "A caret's rhythm; a tint that fades when a value moves; a side-to-side no for a wrong entry.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "blink",
                div().flex().items_center().child("Type here").child(
                    Blink::new("blink").w_0p5().h_4().ml_0p5().bg(theme.colors.fg),
                ),
                cx,
            ))
            .child(specimen(
                "flash",
                row()
                    .child(
                        Flash::new("flash", value)
                            .px_2()
                            .rounded(theme.radius(Radius::Sm))
                            .child(tabular(div()).font_weight(FontWeight::MEDIUM).child(format!("{}.{:02}", value / 100, value % 100))),
                    )
                    .child(probe(
                        "flash-tick",
                        Button::new("flash-tick", "Tick")
                            .variant(ButtonVariant::Ghost)
                            .on_click(move |_, _, cx| set(&tick, value + 37, cx)),
                    )),
                cx,
            ))
            .child(specimen(
                "shake",
                row()
                    .child(
                        Shake::new("shake", count).child(
                            div()
                                .w_48()
                                .px_3()
                                .py_1p5()
                                .rounded(theme.radius(Radius::Md))
                                .border_1()
                                .border_color(if count > 0 { theme.colors.danger } else { theme.colors.border })
                                .child("••••••"),
                        ),
                    )
                    .child(probe(
                        "shake-try",
                        Button::new("shake-try", "Sign in").on_click(move |_, _, cx| set(&again, count + 1, cx)),
                    ))
                    .when(count > 0, |line| line.child(InlineMessage::new(Severity::Danger, "Wrong password"))),
                cx,
            )),
    )
}

pub fn backdrops(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let panel = |text: &'static str| {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .text_size(theme.text_size(TextSize::Lg))
            .font_weight(FontWeight::SEMIBOLD)
            .child(text)
    };
    section(
        "ParticleBackground / AnimatedGradient / Lottie Player",
        "Faint motes that drift behind a panel; a tone that drifts, slowly, a breath toward the accent.",
        cx,
    )
    .child(
        row()
            .child(
                ParticleBackground::new("particles")
                    .w_80()
                    .h_40()
                    .rounded(theme.radius(Radius::Lg))
                    .border_1()
                    .border_color(theme.colors.border)
                    .child(panel("Quiet motion")),
            )
            .child(
                AnimatedGradient::new("tide")
                    .w_80()
                    .h_40()
                    .rounded(theme.radius(Radius::Lg))
                    .border_1()
                    .border_color(theme.colors.border)
                    .child(panel("Tonal drift")),
            ),
    )
    .child(blocked(
        "Lottie Player is blocked: gpui paints paths without the clip masks, mattes, blend modes and radial gradients Lottie files lean on, so frames would come out wrong.",
        cx,
    ))
}
