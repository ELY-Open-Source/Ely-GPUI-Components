use ely_gpui_component::{
    theme::{ActiveTheme, Radius},
    typography::{
        Blockquote, Caption, Code, Ellipsis, EllipsisTooltip, ExternalLink, Heading, Highlight,
        Kbd, KbdCombo, Label, Link, MiddleEllipsis, Overline, Paragraph, Subtitle, Title,
    },
};
use gpui::{App, IntoElement, ParentElement, Styled, div, px};

use crate::{
    probe::probe,
    ui::{code, row, section, specimen, specimens},
};

const PROSE: &str = "Good interfaces are quiet. They leave room around each idea, \
use one voice for text, and save color for the moments that need it. The rest is spacing.";

pub fn headings(cx: &App) -> impl IntoElement + use<> {
    section(
        "Heading (H1–H6)",
        "Six steps, one weight. Size carries the order.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Heading::h1("Quiet interfaces"))
            .child(Heading::h2("Quiet interfaces"))
            .child(Heading::h3("Quiet interfaces"))
            .child(Heading::h4("Quiet interfaces"))
            .child(Heading::h5("Quiet interfaces"))
            .child(Heading::h6("Quiet interfaces")),
    )
}

pub fn title(cx: &App) -> impl IntoElement + use<> {
    section("Title / Subtitle", "A pair for page and panel headers.", cx).child(
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(Title::new("Quarterly review"))
            .child(Subtitle::new("Numbers, notes and the next three moves.")),
    )
}

pub fn paragraph(cx: &App) -> impl IntoElement + use<> {
    section("Paragraph", "Reading text at 14 with loose leading.", cx)
        .child(Paragraph::new(PROSE).max_w(px(560.0)))
}

pub fn label(cx: &App) -> impl IntoElement + use<> {
    section("Label", "Names a control or a value.", cx).child(
        row()
            .child(Label::new("Display name"))
            .child(Label::new("Workspace")),
    )
}

pub fn caption(cx: &App) -> impl IntoElement + use<> {
    section("Caption", "The quiet line beneath.", cx)
        .child(Caption::new("Updated a moment ago · visible to your team"))
}

pub fn overline(cx: &App) -> impl IntoElement + use<> {
    section("Overline", "An eyebrow in small capitals.", cx).child(
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(Overline::new("Release notes"))
            .child(Title::new("Version 0.1")),
    )
}

pub fn inline_code(cx: &App) -> impl IntoElement + use<> {
    section("Code", "Monospace on a quiet fill.", cx).child(
        row()
            .child("Run")
            .child(Code::new("cargo run -p ely-gallery"))
            .child("to open this window."),
    )
}

pub fn kbd(cx: &App) -> impl IntoElement + use<> {
    section(
        "Kbd",
        "gpui key syntax. secondary means ⌘ on macOS and Ctrl elsewhere.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("secondary-s", Kbd::new("secondary-s"), cx))
            .child(specimen("shift-tab", Kbd::new("shift-tab"), cx))
            .child(specimen("escape", Kbd::new("escape"), cx))
            .child(specimen("enter", Kbd::new("enter"), cx)),
    )
}

pub fn kbd_combo(cx: &App) -> impl IntoElement + use<> {
    section("KbdCombo", "Sequences read left to right.", cx).child(
        specimens()
            .child(specimen(
                "secondary-shift-p",
                KbdCombo::new("secondary-shift-p"),
                cx,
            ))
            .child(specimen(
                "secondary-k secondary-s",
                KbdCombo::new("secondary-k secondary-s"),
                cx,
            )),
    )
}

pub fn blockquote(cx: &App) -> impl IntoElement + use<> {
    section("Blockquote", "A rule and a softer ink.", cx).child(div().max_w(px(520.0)).child(
        Blockquote::new().child(
            "Simplicity is not the absence of clutter. It is the right thing, in the right place.",
        ),
    ))
}

pub fn highlight(cx: &App) -> impl IntoElement + use<> {
    section(
        "Highlight / Mark",
        "Every match of a query, case-insensitive.",
        cx,
    )
    .child(Highlight::matching(
        "Quiet interfaces age well. Loud ones AGE fast.",
        "age",
    ))
    .child(code("Highlight::matching(text, \"age\")", cx))
}

fn narrow(cx: &App) -> gpui::Div {
    let theme = cx.theme();
    div()
        .w(px(240.0))
        .px_3()
        .py_2()
        .rounded(theme.radius(Radius::Md))
        .border_1()
        .border_color(theme.colors.border)
}

pub fn truncate(cx: &App) -> impl IntoElement + use<> {
    section(
        "Truncate / Ellipsis",
        "One line, an ellipsis at the end, in the text style around it, cut by the shaped line at the width its box gets.",
        cx,
    )
    .child(
        narrow(cx)
            .flex()
            .items_center()
            .gap_2()
            .child(div().flex_none().child("Note"))
            .child(Ellipsis::new(PROSE)),
    )
}

pub fn line_clamp(cx: &App) -> impl IntoElement + use<> {
    section(
        "LineClamp",
        "gpui's .line_clamp(n): n lines, then a hard cut.",
        cx,
    )
    .child(
        narrow(cx)
            .w(px(360.0))
            .child(div().line_clamp(2).child(PROSE)),
    )
}

pub fn middle(cx: &App) -> impl IntoElement + use<> {
    section(
        "MiddleEllipsis",
        "Cuts the middle so both ends stay legible.",
        cx,
    )
    .child(narrow(cx).w(px(280.0)).child(MiddleEllipsis::new(
        "~/Documents/GitHub/Ely-GPUI-Components/src/typography/fit.rs",
    )))
}

pub fn ellipsis_tooltip(cx: &App) -> impl IntoElement + use<> {
    section(
        "EllipsisTooltip",
        "Hover shows the whole line, only when it was cut.",
        cx,
    )
    .child(probe(
        "fit-tooltip",
        narrow(cx).child(EllipsisTooltip::new(
            "fit-tooltip-text",
            "Design review · Tuesday, with the full product team and guests",
        )),
    ))
}

pub fn link(cx: &App) -> impl IntoElement + use<> {
    section(
        "Link",
        "Blue marks where a click goes. Underline on hover.",
        cx,
    )
    .child(
        row()
            .child("Read the")
            .child(Link::new("guide", "style guide", |_, _, _| {
                log::info!("gallery: link")
            }))
            .child("before you start."),
    )
}

pub fn external_link(cx: &App) -> impl IntoElement + use<> {
    section("ExternalLink", "Leaves the app. The arrow says so.", cx).child(ExternalLink::new(
        "gpui",
        "gpui.rs",
        "https://www.gpui.rs",
    ))
}
