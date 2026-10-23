use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Window,
};

use super::{
    links::Link,
    plane::{InfiniteCanvas, OnViewport},
    shape::{Shape, ShapeKind},
    view::{Frame, Viewport},
};

/// What a step of a flow is: where it starts or ends, work done, or a choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    /// Where the flow starts or ends.
    Terminal,
    /// Work done.
    Process,
    /// A choice.
    Decision,
}

/// A step of a flow: its key, what it is, its words and its frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    /// Tells it apart from the others.
    pub key: SharedString,
    /// What it is.
    pub kind: StepKind,
    /// Its words.
    pub text: SharedString,
    /// Its box on the canvas.
    pub frame: Frame,
}

impl Step {
    /// A step keyed `key`, of `kind`, saying `text`, in `frame`.
    pub fn new(
        key: impl Into<SharedString>,
        kind: StepKind,
        text: impl Into<SharedString>,
        frame: Frame,
    ) -> Self {
        Self {
            key: key.into(),
            kind,
            text: text.into(),
            frame,
        }
    }

    /// The shape that draws it, its words inside: an ellipse for an end, a box for work, a diamond for a choice.
    pub fn shape(&self) -> Shape {
        let (kind, hue) = match self.kind {
            StepKind::Terminal => (ShapeKind::Ellipse, 5),
            StepKind::Process => (ShapeKind::Rect, 0),
            StepKind::Decision => (ShapeKind::Polygon(4), 2),
        };
        Shape::new(self.key.clone(), self.text.clone(), kind, self.frame)
            .hue(hue)
            .label(self.text.clone())
    }
}

/// Steps joined by links on an endless plane, each link following its steps with its own words. A layer edits it: a ToolLayer given the steps' shapes moves and resizes steps, links them with the connector, and rewrites one on a double press.
///
/// ```
/// use ely_canvas::{FlowChart, Frame, Link, Step, StepKind, Viewport};
///
/// let steps = [
///     Step::new("start", StepKind::Terminal, "Start", Frame::new(0.0, 0.0, 120.0, 48.0)),
///     Step::new("check", StepKind::Decision, "Valid?", Frame::new(0.0, 100.0, 120.0, 80.0)),
/// ];
/// let chart = FlowChart::new("flow", Viewport::new(0.0, 0.0, 1.0), steps, [Link::new("l1", "start", "check")]);
/// ```
#[derive(IntoElement)]
pub struct FlowChart {
    id: ElementId,
    viewport: Viewport,
    steps: Vec<Step>,
    links: Vec<Link>,
    layers: Vec<AnyElement>,
    on_viewport: Option<OnViewport>,
}

impl FlowChart {
    /// A chart under `id` seen through `viewport`, of `steps` joined by `links`.
    pub fn new(
        id: impl Into<ElementId>,
        viewport: Viewport,
        steps: impl IntoIterator<Item = Step>,
        links: impl IntoIterator<Item = Link>,
    ) -> Self {
        Self {
            id: id.into(),
            viewport,
            steps: steps.into_iter().collect(),
            links: links.into_iter().collect(),
            layers: Vec::new(),
            on_viewport: None,
        }
    }

    /// Lays an element over the chart, such as a `ToolLayer`.
    pub fn layer(mut self, element: impl IntoElement) -> Self {
        self.layers.push(element.into_any_element());
        self
    }

    /// Gets the viewport after each pan and zoom.
    pub fn on_viewport(
        mut self,
        handler: impl Fn(Viewport, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_viewport = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for FlowChart {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let canvas = InfiniteCanvas::new(self.id, self.viewport)
            .shapes(self.steps.iter().map(Step::shape))
            .links(self.links);
        let canvas = match self.on_viewport {
            Some(on_viewport) => {
                canvas.on_viewport(move |view, window, cx| on_viewport(view, window, cx))
            }
            None => canvas,
        };
        self.layers
            .into_iter()
            .fold(canvas, |canvas, layer| canvas.layer(layer))
    }
}

impl ParentElement for FlowChart {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.layers.extend(elements);
    }
}
