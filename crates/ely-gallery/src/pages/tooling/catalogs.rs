use ely_gpui_component::{
    forms::IconPicker,
    primitives::{Icon, IconName},
    theme::{ActiveTheme, IconSize},
    tooling::{A11yChecker, DesignTokenViewer},
    typography::Code,
};
use gpui::{AnyElement, App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::probe::probe;
use crate::ui::section;

pub fn sections(window: &mut Window, cx: &mut App) -> Vec<AnyElement> {
    let picked = window.use_keyed_state("icon-browser-picked", cx, |_, _| IconName::Search);
    let icon = *picked.read(cx);
    let fg = cx.theme().colors.fg;
    vec![
        section(
            "DesignTokenViewer",
            "The theme's shared colors and scales in the mode shown, each named as code reads it: every palette, syntax, chart and terminal color with its hex, and the type, radius, control, icon, avatar and container sizes, the elevations and the spacing steps.",
            cx,
        )
        .child(div().max_w(px(840.0)).child(DesignTokenViewer::new()))
        .into_any_element(),
        section(
            "IconBrowser",
            "The library's icons, searchable: this is forms::IconPicker, its home. Pick one to see the name code uses.",
            cx,
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_start()
                .gap_6()
                .child(probe(
                    "icon-browser",
                    IconPicker::new("icon-browser")
                        .selected(icon)
                        .on_change(move |next, _, cx| {
                            picked.update(cx, |picked, cx| {
                                *picked = next;
                                cx.notify();
                            })
                        }),
                ))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(Icon::new(icon).size(IconSize::Xxl).color(fg))
                        .child(Code::new(format!("IconName::{icon:?}"))),
                ),
        )
        .into_any_element(),
        section(
            "A11yChecker",
            "The theme's contrast in the mode shown: each text color over the surfaces it sits on and each code color over the editor's at 4.5:1, the focus ring and the chart hues beside the page at 3:1, failures first. Ely's palettes pass in both modes, with high contrast and without.",
            cx,
        )
        .child(div().max_w(px(560.0)).child(A11yChecker::new()))
        .into_any_element(),
        section(
            "ScreenshotTest Harness",
            "The gallery photographs itself: --capture writes every page top to bottom, light and dark, then each page's scripted states, with input posted to its own event queue. Each pass starts its page fresh, and reduced motion holds the shots still.",
            cx,
        )
        .child(Code::new(
            "cargo run -p ely-gallery -- --page tooling --capture shots",
        ))
        .into_any_element(),
    ]
}
