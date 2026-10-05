use ely_gpui_component::{
    buttons::IconButton,
    navigation::NavItem,
    primitives::IconName,
    shell::{ColumnShell, ShellColumn},
    theme::{ActiveTheme, ControlSize},
    typography::{Caption, Paragraph},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, div, px};

use super::chrome::window_frame;
use crate::ui::{code, section, specimen};

fn icon(id: &'static str, ix: usize, name: IconName) -> IconButton {
    IconButton::new((id, ix), name).size(ControlSize::Sm)
}

fn shell(id: &'static str, cx: &App) -> ColumnShell {
    ColumnShell::new(id)
        .platform(cx.theme().platform)
        .on_close(|_, _| log::info!("gallery: demo close refused"))
}

fn sidebar(id: &'static str, items: [(IconName, &'static str); 3], cx: &App) -> ShellColumn {
    let nav = div()
        .p_2()
        .flex()
        .flex_col()
        .gap_0p5()
        .children(items.into_iter().enumerate().map(|(ix, (glyph, label))| {
            NavItem::new(SharedString::from(format!("{id}-{ix}")), glyph, label).active(ix == 0)
        }));
    ShellColumn::new(id)
        .width(cx.theme().sidebar_width(false))
        .leading(icon(id, 0, IconName::PanelLeft))
        .leading(icon(id, 1, IconName::ArrowLeft))
        .leading(icon(id, 2, IconName::ArrowRight))
        .child(nav)
}

fn two_columns(cx: &App) -> impl IntoElement + use<> {
    let items = [
        (IconName::SquarePen, "New task"),
        (IconName::Search, "Search"),
        (IconName::Inbox, "Inbox"),
    ];
    let main = ShellColumn::new("cs2-main")
        .title("New task")
        .action(icon("cs2-main", 0, IconName::Info))
        .action(icon("cs2-main", 1, IconName::PanelRight))
        .child(
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(Paragraph::new("What should we build?")),
        );
    window_frame(640.0, 280.0, cx).child(
        shell("cs2", cx)
            .column(sidebar("cs2-side", items, cx))
            .column(main),
    )
}

fn three_columns(cx: &App) -> impl IntoElement + use<> {
    let items = [
        (IconName::Inbox, "Inbox"),
        (IconName::Send, "Sent"),
        (IconName::Archive, "Archive"),
    ];
    let mail = [
        (
            "Quarterly review",
            "Numbers, notes and the next three moves.",
        ),
        ("Design sync", "Tokens for the new sidebar."),
        ("Launch plan", "Dates for the beta."),
    ];
    let list = ShellColumn::new("cs3-list")
        .width(px(220.0))
        .title("Inbox")
        .action(icon("cs3-list", 0, IconName::Search))
        .child(
            div()
                .p_2()
                .flex()
                .flex_col()
                .gap_3()
                .children(mail.map(|(subject, line)| {
                    div()
                        .px_2()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .child(Paragraph::new(subject))
                        .child(Caption::new(line))
                })),
        );
    let message = ShellColumn::new("cs3-main")
        .title("Quarterly review")
        .action(icon("cs3-main", 0, IconName::Reply))
        .action(icon("cs3-main", 1, IconName::Archive))
        .child(div().p_5().child(Paragraph::new(
            "Revenue rose eight percent. Three moves come next: hire, ship the beta, close the books.",
        )));
    window_frame(760.0, 280.0, cx).child(
        shell("cs3", cx)
            .column(sidebar("cs3-side", items, cx))
            .column(list)
            .column(message),
    )
}

pub fn column_shell(cx: &App) -> impl IntoElement + use<> {
    section(
        "ColumnShell / ShellColumn",
        "The sidebar runs to the window's top; each column brings a header that drags.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(specimen("Two columns", two_columns(cx), cx))
            .child(specimen("Three columns", three_columns(cx), cx)),
    )
    .child(code(
        "ColumnShell::new(id) with no platform leaves the first header room for the real macOS lights.",
        cx,
    ))
}
