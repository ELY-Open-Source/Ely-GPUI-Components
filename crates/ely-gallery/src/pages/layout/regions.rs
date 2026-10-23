use ely_gpui_component::{
    buttons::{Button, ButtonVariant, IconButton},
    layout::{
        Accordion, AccordionItem, Card, CardHeader, Collapsible, Fieldset, Frame, Panel, Section,
        SplitPane, Well, resize_handle,
    },
    primitives::IconName,
    theme::{ActiveTheme, ControlSize, Radius},
    typography::{Caption, Label, Paragraph},
};
use gpui::{App, Axis, IntoElement, ParentElement, Styled, Window, div, px};

use super::basics::tile;
use crate::{
    probe::probe,
    ui::{code, row, section},
};

fn pane(label: &'static str, cx: &App) -> gpui::Div {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(cx.theme().colors.sunken)
        .text_color(cx.theme().colors.fg_subtle)
        .child(label)
}

fn frame(cx: &App) -> gpui::Div {
    let theme = cx.theme();
    div()
        .w(px(560.0))
        .h(px(220.0))
        .overflow_hidden()
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border)
}

pub fn split_pane(cx: &App) -> impl IntoElement + use<> {
    section(
        "SplitPane",
        "Drag a divider. Panes nest across and down.",
        cx,
    )
    .child(probe(
        "split",
        frame(cx).child(
            SplitPane::new("split-outer", Axis::Horizontal, px(96.0))
                .sizes(&[0.3, 0.7])
                .pane(pane("files", cx))
                .pane(
                    SplitPane::new("split-inner", Axis::Vertical, px(48.0))
                        .pane(pane("editor", cx))
                        .pane(pane("terminal", cx)),
                ),
        ),
    ))
    .child(code(
        "ResizablePanel and ThreeColumnLayout are SplitPane.",
        cx,
    ))
}

pub fn resize_handles(cx: &App) -> impl IntoElement + use<> {
    section(
        "ResizeHandle",
        "A hairline with a wide grab. Hover lights it.",
        cx,
    )
    .child(
        row()
            .gap_8()
            .child(
                div()
                    .h(px(80.0))
                    .child(resize_handle("handle-x", Axis::Horizontal, cx)),
            )
            .child(
                div()
                    .w(px(160.0))
                    .child(resize_handle("handle-y", Axis::Vertical, cx)),
            ),
    )
}

pub fn collapsible(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = window.use_keyed_state("collapse-open", cx, |_, _| true);
    let is_open = *open.read(cx);
    section(
        "Collapsible",
        "Height animates to the measured content.",
        cx,
    )
    .child(
        div()
            .w(px(420.0))
            .flex()
            .flex_col()
            .gap_2()
            .child(probe(
                "collapse-toggle",
                Button::new(
                    "collapse-toggle",
                    if is_open { "Collapse" } else { "Expand" },
                )
                .variant(ButtonVariant::Outline)
                .trailing_icon(if is_open {
                    IconName::ChevronUp
                } else {
                    IconName::ChevronDown
                })
                .on_click(move |_, _, cx| {
                    open.update(cx, |open, cx| {
                        *open = !*open;
                        cx.notify();
                    })
                }),
            ))
            .child(
                Collapsible::new("collapse", is_open).child(
                    Paragraph::new(
                        "Content keeps its natural height. The frame grows to it, then lets go.",
                    )
                    .pt_2(),
                ),
            ),
    )
}

pub fn accordion(cx: &App) -> impl IntoElement + use<> {
    section(
        "Accordion / AccordionItem",
        "One open at a time. The chevron turns with it.",
        cx,
    )
    .child(probe(
        "accordion",
        div().w(px(480.0)).child(
            Accordion::new("faq")
                .first_open()
                .item(
                    AccordionItem::new("What is Ely?")
                        .child("A component library for gpui, in light and dark."),
                )
                .item(
                    AccordionItem::new("Does it animate?")
                        .child("Where motion explains a change, and nowhere else."),
                )
                .item(
                    AccordionItem::new("Can I theme it?")
                        .child("Every color, size and radius comes from one token set."),
                ),
        ),
    ))
}

pub fn card(cx: &App) -> impl IntoElement + use<> {
    section(
        "Card / CardHeader / CardBody / CardFooter",
        "Flat: a hairline, no shadow.",
        cx,
    )
    .child(
        Card::new()
            .w(px(380.0))
            .header(
                CardHeader::new("Storage")
                    .description("Across all workspaces")
                    .action(IconButton::new("card-more", IconName::Ellipsis).size(ControlSize::Sm)),
            )
            .child(Label::new("42.8 GB of 100 GB"))
            .child(Caption::new("Media takes the most room."))
            .footer(
                div()
                    .flex()
                    .gap_2()
                    .child(Button::new("card-cancel", "Later").variant(ButtonVariant::Ghost))
                    .child(Button::new("card-go", "Manage").primary()),
            ),
    )
}

pub fn panel(cx: &App) -> impl IntoElement + use<> {
    section(
        "Panel / PanelHeader",
        "An app region: a quiet header bar and a body.",
        cx,
    )
    .child(
        Panel::new("Outline")
            .w(px(320.0))
            .h(px(160.0))
            .action(IconButton::new("panel-filter", IconName::Filter).size(ControlSize::Sm))
            .action(IconButton::new("panel-more", IconName::Ellipsis).size(ControlSize::Sm))
            .child(
                div()
                    .p_3()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(tile("fn render", cx))
                    .child(tile("struct Theme", cx)),
            ),
    )
}

pub fn section_demo(cx: &App) -> impl IntoElement + use<> {
    section(
        "Section / SectionHeader",
        "A page block: title, note, action.",
        cx,
    )
    .child(
        Section::new("Members")
            .w(px(520.0))
            .description("People who can open this workspace.")
            .action(Button::new("invite", "Invite").icon(IconName::Plus))
            .child(
                row()
                    .child(tile("Ada", cx))
                    .child(tile("Grace", cx))
                    .child(tile("Linus", cx)),
            ),
    )
}

pub fn fieldset(cx: &App) -> impl IntoElement + use<> {
    section("Group / Fieldset", "Related fields under one name.", cx).child(
        Fieldset::new("Shipping address")
            .w(px(420.0))
            .child(tile("Street", cx))
            .child(row().child(tile("City", cx)).child(tile("Postcode", cx))),
    )
}

pub fn frame_demo(cx: &App) -> impl IntoElement + use<> {
    section("Frame", "A bordered region with no fill.", cx).child(
        Frame::new()
            .w(px(320.0))
            .child(Caption::new("Framed content sits on the page tone.")),
    )
}

pub fn well(cx: &App) -> impl IntoElement + use<> {
    section("Well", "A recessed region on the sunken tone.", cx).child(
        Well::new()
            .w(px(320.0))
            .child(Caption::new("Wells hold secondary material.")),
    )
}
