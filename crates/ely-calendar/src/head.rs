use std::rc::Rc;

use gpui::{App, Div, ElementId, FontWeight, ParentElement, SharedString, Styled, Window, div};
use jiff::civil::Date;

use crate::{
    buttons::{Button, ButtonVariant, IconButton},
    primitives::IconName,
    theme::{ActiveTheme, ControlSize, TextSize},
};

pub(crate) type OnDay = Rc<dyn Fn(Date, &mut Window, &mut App)>;
/// Steps a view back (-1), to today (0) or on (1).
pub(crate) type Step = Rc<dyn Fn(i32, &mut Window, &mut App)>;

/// A view's title, and beside it Today and the arrows back and on, tipped by the catalog keys `back` and `on`; the arrows drop below a narrow title.
pub(crate) fn head(
    id: &ElementId,
    title: String,
    back: &'static str,
    on: &'static str,
    step: Step,
    cx: &App,
) -> Div {
    let theme = cx.theme();
    let (back, on) = (
        crate::i18n::text(cx, back, &[]),
        crate::i18n::text(cx, on, &[]),
    );
    let button = |name: &str, icon: IconName, tip: SharedString, by: i32| {
        let step = step.clone();
        IconButton::new((id.clone(), name.to_string()), icon)
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Sm)
            .tooltip(tip)
            .on_click(move |_, window, cx| step(by, window, cx))
    };
    let today = {
        let step = step.clone();
        Button::new(
            (id.clone(), "today"),
            crate::i18n::text(cx, "calendar.today", &[]),
        )
        .variant(ButtonVariant::Secondary)
        .size(ControlSize::Sm)
        .on_click(move |_, window, cx| step(0, window, cx))
    };
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_2()
        .child(
            div()
                .flex_1()
                .min_w(theme.label_width())
                .text_size(theme.text_size(TextSize::Lg))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.fg)
                .child(title),
        )
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap_1()
                .child(today)
                .child(button("back", IconName::ChevronLeft, back, -1))
                .child(button("on", IconName::ChevronRight, on, 1)),
        )
}
