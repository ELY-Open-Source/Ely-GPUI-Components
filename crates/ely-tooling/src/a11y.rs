use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div,
};

use crate::{
    data_display::{Badge, Tone},
    devtools::{Check, audit},
    theme::{ActiveTheme, Radius, TextSize},
    typography::{Ellipsis, literal, tabular},
};

/// Audits the theme's contrast in the mode shown: each text color over the surfaces it sits on and each code color over the editor's surfaces at 4.5:1, the focus ring and the chart hues beside the page at 3:1, failures first under a count. It reads the theme as it stands, an owner's palette and high contrast included.
///
/// ```
/// use ely_tooling::A11yChecker;
///
/// let checker = A11yChecker::new();
/// ```
#[derive(IntoElement, Default)]
pub struct A11yChecker;

impl A11yChecker {
    /// The checker.
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for A11yChecker {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let mut checks = audit(colors);
        checks.sort_by_key(Check::passes);
        let (total, failing) = (checks.len(), checks.iter().filter(|c| !c.passes()).count());
        let summary = match failing {
            0 => Badge::new(format!("All {total} pass")).tone(Tone::Success),
            n => Badge::new(format!("{n} of {total} fail")).tone(Tone::Danger),
        };
        let rows = checks.into_iter().map(|check| {
            let (label, grade) = (format!("{} on {}", check.front, check.back), check.grade());
            let shown = format!("contrast-{}-{}-{grade}", check.front, check.back);
            div()
                .debug_selector(move || shown)
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex_none()
                        .px_2()
                        .rounded(theme.radius(Radius::Sm))
                        .border_1()
                        .border_color(colors.border)
                        .bg(check.bg)
                        .text_color(check.fg)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Aa"),
                )
                .child(
                    literal(div())
                        .flex_1()
                        .min_w_0()
                        .child(Ellipsis::new(label)),
                )
                .child(
                    tabular(div())
                        .flex_none()
                        .text_color(colors.fg_muted)
                        .child(format!("{:.2}:1", check.ratio)),
                )
                .child(
                    div()
                        .flex_none()
                        .child(Badge::new(grade).tone(match check.passes() {
                            true => Tone::Success,
                            false => Tone::Danger,
                        })),
                )
        });
        div()
            .flex()
            .flex_col()
            .gap_2()
            .text_size(theme.text_size(TextSize::Sm))
            .child(
                div()
                    .debug_selector(move || format!("contrast-summary-{total}-{failing}"))
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Contrast"))
                    .child(summary),
            )
            .children(rows)
    }
}
