use ely_gpui_component::{
    canvas::{
        AutoLayout, AutoLayoutControls, Border, BorderEditor, ColorPanel, Constraints,
        ConstraintsEditor, Dash, ExportFormat, ExportPanel, ExportSetting, Flow, Frame, Pin, Place,
        Shadow, ShadowEditor, TypeStyle, TypographyPanel,
    },
    forms::{GradientEditor, GradientStop, TextInput},
    theme::{ActiveTheme, TextSize},
};
use gpui::{
    App, BoxShadow, FontWeight, Hsla, IntoElement, ParentElement, SharedString, Styled, TextAlign,
    Window, div, point, prelude::*, px, rgb, rgba,
};

use crate::ui::{change, keep, section};

/// The style demo's values.
struct Look {
    color: Hsla,
    stops: Vec<GradientStop>,
    shadows: Vec<Shadow>,
    border: Border,
    style: TypeStyle,
}

impl Look {
    fn new() -> Self {
        Self {
            color: rgb(0xf4efe6).into(),
            stops: vec![
                GradientStop {
                    at: 0.0,
                    color: rgb(0xe9e4da).into(),
                },
                GradientStop {
                    at: 1.0,
                    color: rgb(0x8a7f6d).into(),
                },
            ],
            shadows: vec![Shadow {
                x: 0.0,
                y: 8.0,
                blur: 24.0,
                spread: 0.0,
                color: rgba(0x1816132e).into(),
            }],
            border: Border {
                width: 1.0,
                dash: Dash::Solid,
                color: rgb(0xd8d1c4).into(),
                radii: [12.0; 4],
            },
            style: TypeStyle {
                family: "Inter".into(),
                weight: FontWeight::MEDIUM,
                size: 20.0,
                line_height: 1.3,
                tracking: 0.0,
                align: TextAlign::Left,
            },
        }
    }
}

const SWATCHES: [(&str, u32); 6] = [
    ("Paper", 0xf4efe6),
    ("Sand", 0xe9e4da),
    ("Stone", 0xb8ad9b),
    ("Moss", 0x7d8a6a),
    ("Ink", 0x2b2a27),
    ("Clay", 0xb86f50),
];

pub fn styles(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let look = keep("canvas-look", Look::new, window, cx);
    let search = window.use_keyed_state("canvas-type-search", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Font")
    });
    let now = look.read(cx);
    let (color, stops, shadows, border, style) = (
        now.color,
        now.stops.clone(),
        now.shadows.clone(),
        now.border,
        now.style.clone(),
    );
    let [colored, graded, shaded, bordered, typed] = [(); 5].map(|_| look.clone());
    let [r0, r1, r2, r3] = border.radii;
    let card = div()
        .w(px(200.))
        .h(px(120.))
        .bg(color)
        .border(px(border.width))
        .border_color(border.color)
        .when(border.dash != Dash::Solid, |card| card.border_dashed())
        .rounded_tl(px(r0))
        .rounded_tr(px(r1))
        .rounded_br(px(r2))
        .rounded_bl(px(r3))
        .shadow(
            shadows
                .iter()
                .map(|shadow| BoxShadow {
                    color: shadow.color,
                    offset: point(px(shadow.x), px(shadow.y)),
                    blur_radius: px(shadow.blur),
                    spread_radius: px(shadow.spread),
                    inset: false,
                })
                .collect(),
        );
    section(
        "ColorPanel · GradientPanel · ShadowEditor · BorderEditor · TypographyPanel",
        "What a shape looks like. The color panel sets its fill, over the document's colors; the gradient panel is forms::GradientEditor; shadows stack; a border's corners join or split; the type panel sets a style over a line of it. The card takes each edit.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(
                div().w(px(260.)).child(
                    ColorPanel::new("canvas-color", color, move |next, _, cx| change(&colored, cx, |look| look.color = next))
                        .swatches(SWATCHES.map(|(name, hex)| (name, Hsla::from(rgb(hex))))),
                ),
            )
            .child(
                div()
                    .w(px(280.))
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(
                        GradientEditor::new("canvas-gradient", stops)
                            .on_change(move |next, _, cx| change(&graded, cx, |look| look.stops = next.to_vec())),
                    )
                    .child(
                        ShadowEditor::new("canvas-shadows", shadows, move |next, _, cx| change(&shaded, cx, |look| look.shadows = next)),
                    ),
            )
            .child(
                div()
                    .w(px(260.))
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(
                        BorderEditor::new("canvas-border", border, move |next, _, cx| change(&bordered, cx, |look| look.border = next)),
                    )
                    .child(div().p_6().child(card)),
            )
            .child(
                div().w(px(280.)).child(
                    TypographyPanel::new("canvas-type", style, &search, move |next, _, cx| change(&typed, cx, |look| look.style = next)),
                ),
            ),
    )
}

/// The layout demo's values.
struct Rules {
    constraints: Constraints,
    layout: AutoLayout,
    exports: Vec<ExportSetting>,
    exported: Vec<String>,
}

impl Rules {
    fn new() -> Self {
        Self {
            constraints: Constraints {
                across: Pin::Both,
                down: Pin::Start,
            },
            layout: AutoLayout {
                flow: Flow::Row,
                gap: 12.0,
                padding: (16.0, 16.0),
                place: (Place::Start, Place::Center),
            },
            exports: vec![
                ExportSetting {
                    scale: 1.0,
                    format: ExportFormat::Png,
                },
                ExportSetting {
                    scale: 2.0,
                    format: ExportFormat::Png,
                },
                ExportSetting {
                    scale: 1.0,
                    format: ExportFormat::Svg,
                },
            ],
            exported: Vec::new(),
        }
    }
}

/// A frame in a preview, in its own pixels.
fn boxed(frame: Frame) -> gpui::Div {
    div()
        .absolute()
        .left(px(frame.x))
        .top(px(frame.y))
        .w(px(frame.w))
        .h(px(frame.h))
}

pub fn rules(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let rules = keep("canvas-rules", Rules::new, window, cx);
    let now = rules.read(cx);
    let (constraints, layout, exports, exported) = (
        now.constraints,
        now.layout,
        now.exports.clone(),
        now.exported.clone(),
    );
    let [pinned, laid, set, sent] = [(); 4].map(|_| rules.clone());
    let theme = cx.theme();
    let (line, fill, quiet) = (
        theme.colors.border,
        theme.colors.active,
        theme.colors.fg_subtle,
    );
    let (before, child) = (
        Frame::new(0.0, 0.0, 120.0, 80.0),
        Frame::new(16.0, 16.0, 56.0, 24.0),
    );
    let grown = Frame::new(0.0, 0.0, 240.0, 120.0);
    let pinned_preview = div()
        .relative()
        .w(px(grown.w))
        .h(px(grown.h))
        .border_1()
        .border_color(line)
        .child(boxed(before).border_1().border_color(line).border_dashed())
        .child(
            boxed(constraints.placed(child, before, grown))
                .bg(fill)
                .border_1()
                .border_color(quiet),
        );
    let room = Frame::new(0.0, 0.0, 240.0, 120.0);
    let laid_preview = div()
        .relative()
        .w(px(room.w))
        .h(px(room.h))
        .border_1()
        .border_color(line)
        .children(
            layout
                .laid(room, &[(40.0, 24.0), (56.0, 40.0), (32.0, 32.0)])
                .into_iter()
                .map(|frame| boxed(frame).bg(fill).border_1().border_color(quiet)),
        );
    let note = |text: SharedString| {
        div()
            .text_size(theme.text_size(TextSize::Xs))
            .text_color(quiet)
            .child(text)
    };
    section(
        "ConstraintsEditor · AutoLayoutControls · ExportPanel",
        "How a shape keeps to its frame and lays out its children, and what it exports to. The first box shows where a child pinned so lands once its frame grows from the dashed size; the second lays three children by the controls; export names each file once.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(
                div()
                    .w(px(280.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        ConstraintsEditor::new("canvas-constraints", constraints, move |next, _, cx| change(&pinned, cx, |rules| rules.constraints = next)),
                    )
                    .child(pinned_preview),
            )
            .child(
                div()
                    .w(px(280.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        AutoLayoutControls::new("canvas-auto-layout", layout, move |next, _, cx| change(&laid, cx, |rules| rules.layout = next)),
                    )
                    .child(laid_preview),
            )
            .child(
                div()
                    .w(px(260.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        ExportPanel::new("canvas-export", "Card", exports, move |next, _, cx| change(&set, cx, |rules| rules.exports = next), move |files, _, cx| change(&sent, cx, |rules| rules.exported = files)),
                    )
                    .when(!exported.is_empty(), |panel| {
                        panel.child(note(format!("Exported {}", exported.join(", ")).into()))
                    }),
            ),
    )
}
