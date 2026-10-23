use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    layout::{AppShell, Drawer, MasterDetail, Page, Sheet, Sidebar, h_stack},
    primitives::{Icon, IconName, Place},
    theme::{ActiveTheme, ControlSize, IconSize, Radius, TextSize},
    typography::{Caption, Paragraph},
};
use gpui::{App, Entity, IntoElement, ParentElement, Styled, Window, div, prelude::*, px};

use super::basics::tile;
use crate::{
    probe::probe,
    ui::{row, section},
};

pub fn frame(width: f32, height: f32, cx: &App) -> gpui::Div {
    let theme = cx.theme();
    div()
        .w(px(width))
        .h(px(height))
        .overflow_hidden()
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border)
}

fn toggle(window: &mut Window, cx: &mut App, key: &'static str) -> (Entity<bool>, bool) {
    let state = window.use_keyed_state(key, cx, |_, _| false);
    let value = *state.read(cx);
    (state, value)
}

fn flip(state: &Entity<bool>, cx: &mut App) {
    state.update(cx, |value, cx| {
        *value = !*value;
        cx.notify();
    })
}

fn nav(collapsed: bool, cx: &App) -> impl IntoElement + use<> {
    let items = [
        (IconName::House, "Home"),
        (IconName::Inbox, "Inbox"),
        (IconName::Settings, "Settings"),
    ];
    let theme = cx.theme();
    div()
        .flex()
        .flex_col()
        .gap_0p5()
        .p_2()
        .children(items.map(|(icon, label)| {
            h_stack()
                .gap_3()
                .h(theme.control_height(ControlSize::Md))
                .px_2()
                .rounded(theme.radius(Radius::Md))
                .text_size(theme.text_size(TextSize::Sm))
                .text_color(theme.colors.fg_muted)
                .child(
                    Icon::new(icon)
                        .size(IconSize::Sm)
                        .color(theme.colors.fg_muted),
                )
                .when(!collapsed, |item| item.child(label))
        }))
}

pub fn sidebar(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (state, collapsed) = toggle(window, cx, "sidebar-collapsed");
    section("Sidebar", "Folds to an icon rail; the width eases.", cx).child(
        frame(560.0, 200.0, cx)
            .flex()
            .child(Sidebar::new("demo-sidebar", collapsed).child(nav(collapsed, cx)))
            .child(
                div().flex_1().p_4().flex().items_start().child(probe(
                    "sidebar-toggle",
                    Button::new(
                        "sidebar-toggle",
                        if collapsed { "Expand" } else { "Collapse" },
                    )
                    .variant(ButtonVariant::Outline)
                    .icon(IconName::PanelLeft)
                    .on_click(move |_, _, cx| flip(&state, cx)),
                )),
            ),
    )
}

pub fn drawer(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (state, open) = toggle(window, cx, "drawer-open");
    let close = state.clone();
    section(
        "Drawer",
        "Rises from the bottom. Pull the handle down to dismiss.",
        cx,
    )
    .child(
        row()
            .child(probe(
                "drawer-open",
                Button::new("drawer-open", "Open drawer")
                    .on_click(move |_, _, cx| flip(&state, cx)),
            ))
            .when(open, |row| {
                row.child(
                    Drawer::new("demo-drawer", move |_, cx| flip(&close, cx))
                        .child(probe(
                            "drawer-body",
                            Paragraph::new(
                                "Drawers hold short tasks close to where the hand already is.",
                            ),
                        ))
                        .child(Caption::new("Drag the handle down, or click the scrim.")),
                )
            }),
    )
}

pub fn sheet(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (state, open) = toggle(window, cx, "sheet-open");
    let close = state.clone();
    section("Sheet", "Slides in from an edge over a scrim.", cx).child(
        row()
            .child(probe(
                "sheet-open",
                Button::new("sheet-open", "Open sheet").on_click(move |_, _, cx| flip(&state, cx)),
            ))
            .when(open, |row| {
                row.child(
                    Sheet::new("demo-sheet", Place::Right, "Filters", move |_, cx| {
                        flip(&close, cx)
                    })
                    .child(Paragraph::new(
                        "Sheets keep context visible while a side task runs.",
                    ))
                    .child(tile("Status: open", cx))
                    .child(tile("Owner: anyone", cx)),
                )
            }),
    )
}

pub fn page(cx: &App) -> impl IntoElement + use<> {
    section(
        "Page / PageHeader / PageContent",
        "Title, note and actions over the body.",
        cx,
    )
    .child(
        frame(640.0, 240.0, cx).child(
            Page::new("Projects")
                .subtitle("Everything your team is shipping.")
                .action(
                    Button::new("page-new", "New project")
                        .primary()
                        .icon(IconName::Plus),
                )
                .child(
                    row()
                        .child(tile("Atlas", cx))
                        .child(tile("Beacon", cx))
                        .child(tile("Compass", cx)),
                ),
        ),
    )
}

pub fn app_shell(cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let bar = |label: &'static str, cx: &App| {
        div()
            .h(cx.theme().control_height(ControlSize::Md))
            .px_3()
            .flex()
            .items_center()
            .text_size(cx.theme().text_size(TextSize::Xs))
            .text_color(cx.theme().colors.fg_subtle)
            .child(label)
    };
    section("AppShell", "Title bar, sidebar, content, status bar.", cx).child(
        frame(640.0, 260.0, cx).child(
            AppShell::new()
                .title_bar(
                    bar("Ely — Projects", cx)
                        .border_b_1()
                        .border_color(theme.colors.border),
                )
                .sidebar(Sidebar::new("shell-sidebar", false).child(nav(false, cx)))
                .status_bar(
                    bar("main · 3 changes · UTF-8", cx)
                        .border_t_1()
                        .border_color(theme.colors.border),
                )
                .child(div().p_4().child(Paragraph::new(
                    "The shell holds the frame. Your content fills the middle.",
                ))),
        ),
    )
}

pub fn master_detail(cx: &App) -> impl IntoElement + use<> {
    let items = [
        "Quarterly review",
        "Design sync",
        "Launch plan",
        "Hiring loop",
    ];
    section(
        "MasterDetail",
        "A list beside the chosen item. The divider drags.",
        cx,
    )
    .child(
        frame(640.0, 220.0, cx).child(MasterDetail::new(
            "master-detail",
            div()
                .p_2()
                .flex()
                .flex_col()
                .gap_1()
                .children(items.map(|item| tile(item, cx))),
            div()
                .p_5()
                .flex()
                .flex_col()
                .gap_2()
                .child(Paragraph::new("Quarterly review"))
                .child(Caption::new("Numbers, notes and the next three moves.")),
            px(160.0),
        )),
    )
}
