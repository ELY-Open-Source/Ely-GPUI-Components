use std::rc::Rc;

use gpui::{
    App, ElementId, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
};

use crate::{
    canvas::color_well,
    data_display::{Badge, Tone},
    theme::{ActiveTheme, Radius, TextSize, contrast, grades, over},
};

type OnColors = Rc<dyn Fn(Hsla, Hsla, &mut Window, &mut App)>;
type OnColor = Rc<dyn Fn(Hsla, &mut Window, &mut App)>;

/// Text on a background, each color opening a picker, with the contrast between them and the WCAG levels it meets, over a sample of body and large text. Translucent text counts as it blends in; the background counts as opaque.
///
/// ```
/// use ely_devtools::ColorContrastChecker;
/// use gpui::rgb;
///
/// let checker = ColorContrastChecker::new("contrast", rgb(0x333333), rgb(0xffffff), |_text, _back, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct ColorContrastChecker {
    id: ElementId,
    text: Hsla,
    background: Hsla,
    on_change: OnColors,
}

impl ColorContrastChecker {
    /// A checker under `id` for `text` on `background`; `on_change` gets both after each pick.
    pub fn new(
        id: impl Into<ElementId>,
        text: impl Into<Hsla>,
        background: impl Into<Hsla>,
        on_change: impl Fn(Hsla, Hsla, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            background: background.into(),
            on_change: Rc::new(on_change),
        }
    }
}

impl RenderOnce for ColorContrastChecker {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let (text, background) = (self.text, self.background);
        let solid = Hsla {
            a: 1.0,
            ..background
        };
        let ratio = contrast(over(text, solid), solid);
        let on_change = self.on_change;
        let theme = cx.theme();
        let well = |name: &'static str, color: Hsla, set: OnColor| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(color_well(
                    cx,
                    (self.id.clone(), name).into(),
                    color,
                    move |color, window, cx| set(color, window, cx),
                ))
                .child(div().text_size(theme.text_size(TextSize::Sm)).child(name))
        };
        let (fore, back) = (on_change.clone(), on_change);
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_4()
                    .child(well(
                        "Text",
                        text,
                        Rc::new(move |color, window, cx| {
                            log::info!("color contrast checker: text");
                            fore(color, background, window, cx);
                        }),
                    ))
                    .child(well(
                        "Background",
                        background,
                        Rc::new(move |color, window, cx| {
                            log::info!("color contrast checker: background");
                            back(text, color, window, cx);
                        }),
                    )),
            )
            .child(
                div()
                    .p_4()
                    .rounded(theme.radius(Radius::Md))
                    .border_1()
                    .border_color(theme.colors.border)
                    .bg(background)
                    .text_color(text)
                    .child(
                        div()
                            .text_size(theme.text_size(TextSize::Base))
                            .child("Body text reads at this size."),
                    )
                    .child(
                        div()
                            .text_size(theme.text_size(TextSize::Xl))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Large text"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(theme.text_size(TextSize::Lg))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("{ratio:.2} : 1")),
                    )
                    .children(grades(ratio).into_iter().map(|(level, met)| {
                        Badge::new(level).tone(if met { Tone::Success } else { Tone::Danger })
                    })),
            )
    }
}
