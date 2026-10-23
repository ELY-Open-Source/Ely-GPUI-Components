use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    forms::Switch,
    i18n::SkipLink,
    theme::{ActiveTheme, HUE_NAMES, Radius, Theme},
};
use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, Window,
    div, px,
};

use crate::probe::probe;
use crate::ui::{UNREAD, blocked, section, specimen, specimens};

const NAV: [&str; 5] = ["Home", "Docs", "Pricing", "Blog", "About"];

pub fn skip_link(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let main = window
        .use_keyed_state("skip-main", cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let theme = cx.theme();
    let nav = NAV.map(|name| {
        let id = ElementId::Name(SharedString::from(format!("skip-nav-{name}")));
        Button::new(id, name).variant(ButtonVariant::Ghost)
    });
    section(
        "SkipLink · KeyboardNavigationHelper",
        "A link that shows only while focused and moves focus past the nav to the main part: Tab into the card, then Enter. Arrows among a group's items live in interaction::RovingFocus.",
        cx,
    )
    .child(
        probe(
            "skip-demo",
            div()
                .max_w(px(480.0))
                .flex()
                .flex_col()
                .rounded(theme.radius(Radius::Lg))
                .border_1()
                .border_color(theme.colors.border)
                .child(SkipLink::new("skip", "Skip to content", &main))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_1()
                        .p_2()
                        .border_b_1()
                        .border_color(theme.colors.border)
                        .children(nav),
                )
                .child(
                    div()
                        .track_focus(&main)
                        .flex()
                        .flex_col()
                        .items_start()
                        .gap_3()
                        .p_4()
                        .text_color(theme.colors.fg_muted)
                        .child("The main part starts here.")
                        .child(Button::new("skip-read", "Read the guide")),
                ),
        ),
    )
    .child(blocked(
        format!("LiveRegion / Announcer, AccessibleIcon, AriaLabel, ScreenReaderOnly: not yet shown: {UNREAD}."),
        cx,
    ))
}

pub fn color_blind(cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let on = theme.color_blind_safe();
    let swatches = HUE_NAMES
        .iter()
        .zip(theme.colors.chart)
        .map(|(name, color)| {
            specimen(
                *name,
                div()
                    .size(px(40.0))
                    .rounded(theme.radius(Radius::Md))
                    .bg(color),
                cx,
            )
        });
    section(
        "ColorBlindSafe Palette",
        "Ely's chart hues share one lightness, so protanopia, deuteranopia and tritanopia run some together. The safe hues differ in lightness too: every pair stays apart for each, and each is 3:1 on its page.",
        cx,
    )
    .child(probe(
        "color-blind",
        Switch::new("color-blind", on)
            .label("Color-blind safe charts")
            .on_change(|on, _, cx| Theme::set_color_blind_safe(on, cx)),
    ))
    .child(specimens().gap_3().children(swatches))
}
