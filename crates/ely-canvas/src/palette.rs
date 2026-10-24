use std::rc::Rc;

use gpui::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use super::{edit::Tool, gesture::Brush};
use crate::{
    buttons::{ToggleGroup, ToggleItem},
    forms::{ColorPalette, Slider},
    i18n,
    primitives::IconName,
    theme::{ActiveTheme, HUE_NAMES, TextSize},
};
use ely_core::typography::tabular;

type OnTool = Rc<dyn Fn(Tool, &mut Window, &mut App)>;
type OnBrush = Rc<dyn Fn(Brush, &mut Window, &mut App)>;

fn icon(tool: Tool) -> IconName {
    match tool {
        Tool::Select => IconName::MousePointer2,
        Tool::Hand => IconName::Hand,
        Tool::Rect => IconName::Square,
        Tool::Ellipse => IconName::Circle,
        Tool::Polygon => IconName::Hexagon,
        Tool::Line => IconName::Slash,
        Tool::Arrow => IconName::MoveUpRight,
        Tool::Pen => IconName::PenTool,
        Tool::Text => IconName::Type,
        Tool::Brush => IconName::Brush,
        Tool::Note => IconName::StickyNote,
        Tool::Connector => IconName::Spline,
    }
}

/// The canvas's tools in a row, one picked, each named in a tip. It offers every tool unless given fewer.
///
/// ```
/// use ely_canvas::{Tool, ToolPalette};
///
/// let palette = ToolPalette::new("tools", Tool::Select).on_change(|_tool, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct ToolPalette {
    id: ElementId,
    tool: Tool,
    tools: Vec<Tool>,
    on_change: Option<OnTool>,
}

impl ToolPalette {
    /// A palette under `id` with `tool` picked, offering every tool.
    pub fn new(id: impl Into<ElementId>, tool: Tool) -> Self {
        Self {
            id: id.into(),
            tool,
            tools: Tool::ALL.to_vec(),
            on_change: None,
        }
    }

    /// The tools it offers, in order; the picked one among them.
    pub fn tools(mut self, tools: impl IntoIterator<Item = Tool>) -> Self {
        self.tools = tools.into_iter().collect();
        self
    }

    /// Gets the tool picked.
    pub fn on_change(mut self, handler: impl Fn(Tool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ToolPalette {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.tools.contains(&self.tool) {
            log::error!(
                "tool palette {:?}: {} is not offered; none chosen",
                self.id,
                self.tool.id()
            );
        }
        let on_change = self.on_change;
        self.tools
            .into_iter()
            .fold(ToggleGroup::new(self.id), |group, tool| {
                group.item(
                    ToggleItem::new(tool.id())
                        .icon(icon(tool))
                        .tooltip(tool.words(cx)),
                )
            })
            .selected([self.tool.id()])
            .on_change(move |values: &[SharedString], window, cx| {
                let Some(id) = values.first() else { return };
                let tool = Tool::ALL
                    .into_iter()
                    .find(|tool| tool.id() == id.as_ref())
                    .expect("a tool of the palette");
                log::info!("tool palette: {}", tool.id());
                if let Some(on_change) = &on_change {
                    on_change(tool, window, cx);
                }
            })
    }
}

/// A brush's width on a slider and its hue from the chart's named colors.
///
/// ```
/// use ely_canvas::{Brush, BrushSettings};
///
/// let settings = BrushSettings::new("brush", Brush { size: 4.0, hue: 2 }).on_change(|_brush, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct BrushSettings {
    id: ElementId,
    brush: Brush,
    on_change: Option<OnBrush>,
}

impl BrushSettings {
    /// Settings under `id` showing `brush`.
    pub fn new(id: impl Into<ElementId>, brush: Brush) -> Self {
        Self {
            id: id.into(),
            brush,
            on_change: None,
        }
    }

    /// Gets the brush after each change.
    pub fn on_change(mut self, handler: impl Fn(Brush, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for BrushSettings {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let chart = theme.colors.chart;
        let brush = self.brush;
        let (sized, hued) = (self.on_change.clone(), self.on_change);
        let width = Slider::new((self.id.clone(), "size"), f64::from(brush.size))
            .label(i18n::text(cx, "brush.size", &[]))
            .range(1.0, 24.0)
            .step(1.0)
            .on_change(move |size, window, cx| {
                if let Some(on_change) = &sized {
                    on_change(
                        Brush {
                            size: size as f32,
                            ..brush
                        },
                        window,
                        cx,
                    );
                }
            });
        let hues = ColorPalette::new(
            (self.id.clone(), "hue"),
            HUE_NAMES
                .iter()
                .zip(chart)
                .map(|(name, color)| (*name, color)),
        )
        .columns(8)
        .selected(chart[brush.hue])
        .on_change(move |color, window, cx| {
            let hue = chart
                .iter()
                .position(|each| *each == color)
                .expect("a hue of the chart");
            if let Some(on_change) = &hued {
                on_change(Brush { hue, ..brush }, window, cx);
            }
        });
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().flex_1().child(width))
                    .child(
                        tabular(div())
                            .flex_none()
                            .text_size(theme.text_size(TextSize::Sm))
                            .text_color(theme.colors.fg_muted)
                            .child(format!("{} pt", brush.size)),
                    ),
            )
            .child(hues)
    }
}

ely_core::capable!(ToolPalette: change(Tool));
ely_core::capable!(BrushSettings: change(Brush));
