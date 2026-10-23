use ely_gpui_component::{
    canvas::{
        Brush, BrushSettings, Frame, HistoryPanel, InfiniteCanvas, Shape, ShapeKind, Tool,
        ToolLayer, ToolPalette, Viewport,
    },
    theme::{ActiveTheme, Radius},
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

/// The tools demo's drawing: its shapes, what is selected, the tool and brush, the view, and each step with the shapes after it.
pub(super) struct Studio {
    pub(super) shapes: Vec<Shape>,
    pub(super) selected: Vec<SharedString>,
    tool: Tool,
    brush: Brush,
    view: Viewport,
    steps: Vec<(SharedString, Vec<Shape>)>,
    at: usize,
    pub(super) made: usize,
}

impl Studio {
    fn new() -> Self {
        let shapes = vec![
            Shape::new(
                "card",
                "Card",
                ShapeKind::Rect,
                Frame::new(40.0, 40.0, 220.0, 140.0),
            )
            .hue(0),
            Shape::new(
                "badge",
                "Badge",
                ShapeKind::Ellipse,
                Frame::new(300.0, 60.0, 90.0, 90.0),
            )
            .hue(5),
            Shape::new(
                "label",
                "Label",
                ShapeKind::Text("Drag me".into()),
                Frame::new(60.0, 210.0, 160.0, 32.0),
            ),
        ];
        Self {
            shapes: shapes.clone(),
            selected: vec!["card".into()],
            tool: Tool::Select,
            brush: Brush { size: 4.0, hue: 3 },
            view: Viewport::new(0.0, 0.0, 1.0),
            steps: vec![("Open the drawing".into(), shapes)],
            at: 1,
            made: 0,
        }
    }

    /// Keeps the shapes as a new step, dropping any steps undone.
    pub(super) fn record(&mut self, words: String) {
        self.steps.truncate(self.at);
        self.steps.push((words.into(), self.shapes.clone()));
        self.at = self.steps.len();
    }
}

pub(super) fn studio(window: &mut Window, cx: &mut App) -> Entity<Studio> {
    keep("canvas-studio", Studio::new, window, cx)
}

pub fn tools(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let studio = studio(window, cx);
    let now = studio.read(cx);
    let (tool, view, brush, at) = (now.tool, now.view, now.brush, now.at);
    let (shapes, selected) = (now.shapes.clone(), now.selected.clone());
    let steps: Vec<SharedString> = now.steps.iter().map(|(words, _)| words.clone()).collect();
    let [
        picked,
        viewed,
        chosen,
        drawn,
        moved,
        sized,
        brushed,
        travelled,
    ] = [(); 8].map(|_| studio.clone());
    let layer = ToolLayer::new("canvas-tools-layer", tool, view, shapes.clone())
        .selected(selected)
        .brush(brush)
        .on_select(move |keys, _, cx| change(&chosen, cx, |studio| studio.selected = keys.to_vec()))
        .on_draw(move |kind, frame, _, cx| {
            change(&drawn, cx, |studio| {
                studio.made += 1;
                let key = SharedString::from(format!("made-{}", studio.made));
                let hue = match kind {
                    ShapeKind::Path { .. } => studio.brush.hue,
                    _ => studio.made % 8,
                };
                let words = format!(
                    "Draw {}",
                    match &kind {
                        ShapeKind::Path { .. } => "a stroke",
                        ShapeKind::Text(_) => "text",
                        _ => "a shape",
                    }
                );
                studio
                    .shapes
                    .push(Shape::new(key.clone(), "Drawn", kind, frame).hue(hue));
                studio.selected = vec![key];
                studio.record(words);
            })
        })
        .on_move(move |keys, (dx, dy), _, cx| {
            change(&moved, cx, |studio| {
                studio
                    .shapes
                    .iter_mut()
                    .filter(|shape| keys.contains(&shape.key))
                    .for_each(|shape| {
                        shape.frame.x += dx;
                        shape.frame.y += dy;
                    });
                studio.record(format!("Move {}", keys.len()));
            })
        })
        .on_resize(move |key, frame, _, cx| {
            change(&sized, cx, |studio| {
                let shape = studio
                    .shapes
                    .iter_mut()
                    .find(|shape| shape.key == *key)
                    .expect("a listed shape");
                shape.frame = frame;
                studio.record(format!("Resize {key}"));
            })
        });
    let canvas = InfiniteCanvas::new("canvas-tools", view)
        .shapes(shapes)
        .on_viewport(move |next, _, cx| change(&viewed, cx, |studio| studio.view = next))
        .layer(layer);
    let theme = cx.theme();
    section(
        "ToolPalette · SelectionBox · TransformHandles · SnapIndicator · ShapeTools · PenTool / PathEditor · TextTool · BrushTool / BrushSettings · HistoryPanel",
        "Tools at work on the plane. Select presses a shape, Shift adds, a drag moves it and snaps its edges to the others, a handle resizes it, and a drag on empty space draws a marquee. The shape tools draw by a drag, Shift keeping them square; the pen adds a point per press and a double press ends the path; text drops a line; the brush strokes. Each change is a step in the history, which a double press or Enter takes back to.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(probe(
                        "canvas-palette",
                        ToolPalette::new("canvas-palette", tool)
                            .on_change(move |tool, _, cx| change(&picked, cx, |studio| studio.tool = tool)),
                    ))
                    .child(probe(
                        "canvas-tools",
                        div()
                            .w(px(560.))
                            .h(px(360.))
                            .rounded(theme.radius(Radius::Lg))
                            .border_1()
                            .border_color(theme.colors.border)
                            .overflow_hidden()
                            .child(canvas),
                    )),
            )
            .child(
                div()
                    .w(px(260.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(BrushSettings::new("canvas-brush", brush).on_change(move |brush, _, cx| change(&brushed, cx, |studio| studio.brush = brush)))
                    .child(HistoryPanel::new("canvas-history", steps, at).on_step(move |at, _, cx| {
                        change(&travelled, cx, |studio| {
                            studio.at = at;
                            studio.shapes = studio.steps[at - 1].1.clone();
                            studio.selected.clear();
                        })
                    })),
            ),
    )
}
