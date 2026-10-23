use ely_gpui_component::{
    buttons::IconButton,
    primitives::IconName,
    shell::{TitleBar, WindowControls, drag_region},
    theme::{ActiveTheme, ControlSize, Elevation, Platform, Radius, TextSize},
    typography::Caption,
};
use gpui::{App, Div, IntoElement, ParentElement, Styled, Window, div, px};

use crate::ui::{code, row, section, specimen, specimens};

/// A window drawn inside the page.
pub fn window_frame(width: f32, height: f32, cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .w(px(width))
        .h(px(height))
        .flex()
        .flex_col()
        .overflow_hidden()
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.sunken)
        .shadow(theme.elevation(Elevation::Raised))
}

fn demo_bar(id: &'static str, style: Platform) -> TitleBar {
    TitleBar::new(id)
        .platform(style)
        .title("Ely — Projects")
        .on_close(|_, _| log::info!("gallery: demo close refused"))
        .child(IconButton::new((id, 1usize), IconName::PanelLeft).size(ControlSize::Sm))
        .action(IconButton::new((id, 2usize), IconName::Search).size(ControlSize::Sm))
}

pub fn title_bar(cx: &App) -> impl IntoElement + use<> {
    let styles = [
        ("tb-mac", Platform::Mac, "macOS"),
        ("tb-windows", Platform::Windows, "Windows"),
        ("tb-linux", Platform::Linux, "Linux"),
    ];
    section(
        "TitleBar",
        "Title centered, system buttons in their place, empty space drags.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_6()
            .children(styles.map(|(id, style, name)| {
                specimen(
                    name,
                    window_frame(560.0, 120.0, cx).child(demo_bar(id, style)),
                    cx,
                )
            })),
    )
    .child(code(
        "TitleBar::new(id) with no platform leaves room for the real macOS lights.",
        cx,
    ))
}

pub fn window_controls(cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let plate = || {
        div()
            .h(theme.titlebar_height())
            .px_3()
            .flex()
            .items_center()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.bg)
    };
    section(
        "WindowControls / TrafficLights",
        "Three manners. Hover the lights for their glyphs.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "Mac",
                plate().child(
                    WindowControls::new("wc-mac")
                        .platform(Platform::Mac)
                        .on_close(|_, _| log::info!("gallery: demo close refused")),
                ),
                cx,
            ))
            .child(specimen(
                "Windows",
                plate().px_0().child(
                    WindowControls::new("wc-windows")
                        .platform(Platform::Windows)
                        .on_close(|_, _| log::info!("gallery: demo close refused")),
                ),
                cx,
            ))
            .child(specimen(
                "Linux",
                plate().child(
                    WindowControls::new("wc-linux")
                        .platform(Platform::Linux)
                        .on_close(|_, _| log::info!("gallery: demo close refused")),
                ),
                cx,
            )),
    )
    .child(code(
        "Minimize and zoom act on this window. Close here only logs.",
        cx,
    ))
}

pub fn drag_area(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let strip = drag_region("demo-drag", window, cx);
    let theme = cx.theme();
    section(
        "WindowDragRegion",
        "Windows hit-tests it natively, Linux moves by hand, double-click zooms.",
        cx,
    )
    .child(
        strip
            .w(px(420.0))
            .h(theme.control_height(ControlSize::Lg))
            .flex()
            .items_center()
            .justify_center()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_dashed()
            .border_color(theme.colors.border_strong)
            .text_size(theme.text_size(TextSize::Sm))
            .text_color(theme.colors.fg_subtle)
            .child("Double-click to zoom this window"),
    )
    .child(code("drag_region(id, window, cx)", cx))
}

pub fn resize_border(cx: &App) -> impl IntoElement + use<> {
    section(
        "WindowResizeBorder",
        "A margin to resize from, a hairline, a shadow, for client-drawn windows.",
        cx,
    )
    .child(Caption::new(
        "Linux draws its own edges. This macOS window keeps the system's, so the border lays out only.",
    ))
    .child(code("ResizeBorder::new().child(app)", cx))
}

pub fn menus(cx: &App) -> impl IntoElement + use<> {
    section(
        "NativeMenu / MenuBar / JumpList",
        "One menu model: the system bar, the Dock, and later the window.",
        cx,
    )
    .child(
        row()
            .child(Caption::new("Look up: Ely and View come from"))
            .child(code("cx.set_menus(menus)", cx)),
    )
    .child(
        row()
            .child(Caption::new(
                "Right-click the Dock icon: New Window comes from",
            ))
            .child(code("cx.set_dock_menu(items)", cx)),
    )
    .child(Caption::new(
        "MenuBar draws the same menus inside the window. It lands with Menus, chapter 8.",
    ))
}
