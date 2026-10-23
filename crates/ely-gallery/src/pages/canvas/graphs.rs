use ely_gpui_component::{
    canvas::{
        Board, Edge, Frame, Link, Node, NodeGraph, Port, Shape, ShapeKind, Viewport, Whiteboard,
    },
    theme::{ActiveTheme, Radius},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

/// The node demo's nodes, wires, view and selection.
struct Pipeline {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    view: Viewport,
    selected: Vec<SharedString>,
}

impl Pipeline {
    fn new() -> Self {
        let port = Port::new;
        Self {
            nodes: vec![
                Node::new("load", "Load image", (20.0, 40.0))
                    .output(port("image", "Image", "image")),
                Node::new("prompt", "Prompt", (20.0, 170.0)).output(port("text", "Text", "text")),
                Node::new("model", "Model", (290.0, 60.0))
                    .input(port("image", "Image", "image"))
                    .input(port("text", "Prompt", "text"))
                    .input(port("steps", "Steps", "number"))
                    .output(port("image", "Image", "image")),
                Node::new("save", "Save image", (560.0, 90.0))
                    .input(port("image", "Image", "image")),
            ],
            edges: vec![
                Edge::new(("load", "image"), ("model", "image")),
                Edge::new(("prompt", "text"), ("model", "text")),
            ],
            view: Viewport::new(0.0, 0.0, 0.8),
            selected: Vec::new(),
        }
    }
}

fn boxed(width: f32, height: f32, child: impl IntoElement, cx: &App) -> gpui::Div {
    let theme = cx.theme();
    div()
        .w(px(width))
        .h(px(height))
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border)
        .overflow_hidden()
        .child(child)
}

pub fn nodes(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("canvas-nodes", Pipeline::new, window, cx);
    let now = state.read(cx);
    let (nodes, edges, view, selected) = (
        now.nodes.clone(),
        now.edges.clone(),
        now.view,
        now.selected.clone(),
    );
    let [picked, moved, joined, parted, viewed] = [(); 5].map(|_| state.clone());
    section(
        "NodeEditor / NodeGraph · Node / Port / Edge / Connection",
        "Nodes joined by wires, as in a pipeline editor. A title drags its node; a drag out of an output draws a wire that lands on an input of the same kind, each kind in its own hue. An input takes one wire, so a new one replaces the old; a press on a wired input lifts the wire to move it or let it go.",
        cx,
    )
    .child(probe(
        "canvas-nodes",
        boxed(
            620.0,
            300.0,
            NodeGraph::new("canvas-node-graph", view, nodes, edges)
                .selected(selected)
                .on_select(move |keys, _, cx| change(&picked, cx, |graph| graph.selected = keys.to_vec()))
                .on_move(move |key, (dx, dy), _, cx| {
                    change(&moved, cx, |graph| {
                        let node = graph.nodes.iter_mut().find(|node| node.key == *key).expect("a node of the graph");
                        node.at = (node.at.0 + dx, node.at.1 + dy);
                    })
                })
                .on_connect(move |edge, _, cx| change(&joined, cx, |graph| graph.edges.push(edge.clone())))
                .on_disconnect(move |edge, _, cx| change(&parted, cx, |graph| graph.edges.retain(|each| each != edge)))
                .on_viewport(move |next, _, cx| change(&viewed, cx, |graph| graph.view = next)),
            cx,
        ),
    ))
}

fn first_board() -> Board {
    let note = |key: &str, words: &str, x, y, hue| {
        Shape::new(
            key.to_string(),
            "Note",
            ShapeKind::Note(words.to_string().into()),
            Frame::new(x, y, 150.0, 150.0),
        )
        .hue(hue)
    };
    Board {
        shapes: vec![
            note("why", "Why do people leave in week two?", 30.0, 80.0, 2),
            note("guess", "Setup takes too long", 230.0, 80.0, 5),
            note("try", "Try a sample project", 430.0, 80.0, 1),
            Shape::new(
                "goal",
                "Goal",
                ShapeKind::Text("Week two retention".into()),
                Frame::new(30.0, 270.0, 220.0, 32.0),
            ),
        ],
        links: vec![
            Link::new("because", "why", "guess"),
            Link::new("so", "guess", "try"),
        ],
    }
}

pub fn board(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("canvas-board", first_board, window, cx);
    let now = state.read(cx).clone();
    let kept = state.clone();
    section(
        "Whiteboard · StickyNote",
        "A board to think on, with its tools over its corner and its zoom at its foot. The note tool drops a sticky note, a double press writes on it, the connector links two notes, and Delete removes the selection. It keeps its own tool and view and hands the owner the whole board after each change.",
        cx,
    )
    .child(probe(
        "canvas-board",
        boxed(
            620.0,
            340.0,
            Whiteboard::new("canvas-whiteboard", now).on_change(move |next, _, cx| change(&kept, cx, |board| *board = next)),
            cx,
        ),
    ))
}
