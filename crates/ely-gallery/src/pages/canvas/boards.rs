use ely_gpui_component::{
    canvas::{
        FlowChart, Frame, Link, MindMap, Step, StepKind, Tool, ToolLayer, ToolPalette, Topic,
        Viewport,
    },
    theme::{ActiveTheme, Radius},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

/// The flow demo's steps, links, view, tool and selection.
struct Flow {
    steps: Vec<Step>,
    links: Vec<Link>,
    view: Viewport,
    tool: Tool,
    selected: Vec<SharedString>,
}

impl Flow {
    fn new() -> Self {
        let step = |key, kind, text, x, y, w| Step::new(key, kind, text, Frame::new(x, y, w, 56.0));
        Self {
            steps: vec![
                step("start", StepKind::Terminal, "Start", 20.0, 132.0, 110.0),
                step(
                    "draft",
                    StepKind::Process,
                    "Write the draft",
                    180.0,
                    132.0,
                    150.0,
                ),
                step(
                    "check",
                    StepKind::Decision,
                    "Reviewed?",
                    380.0,
                    122.0,
                    140.0,
                ),
                step("ship", StepKind::Process, "Publish", 580.0, 132.0, 120.0),
                step("end", StepKind::Terminal, "End", 580.0, 250.0, 120.0),
                step(
                    "revise",
                    StepKind::Process,
                    "Ask for changes",
                    370.0,
                    250.0,
                    160.0,
                ),
            ],
            links: vec![
                Link::new("l1", "start", "draft"),
                Link::new("l2", "draft", "check"),
                Link::new("l3", "check", "ship").label("Yes"),
                Link::new("l4", "check", "revise").label("No"),
                Link::new("l5", "ship", "end"),
            ],
            view: Viewport::new(0.0, 0.0, 0.8),
            tool: Tool::Select,
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

pub fn flow(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("canvas-flow", Flow::new, window, cx);
    let now = state.read(cx);
    let (steps, links, view, tool, selected) = (
        now.steps.clone(),
        now.links.clone(),
        now.view,
        now.tool,
        now.selected.clone(),
    );
    let [picked, chosen, moved, sized, linked, written, viewed] = [(); 7].map(|_| state.clone());
    let at = |steps: &mut Vec<Step>, key: &SharedString| {
        steps
            .iter()
            .position(|step| step.key == *key)
            .expect("a step of the flow")
    };
    let layer = ToolLayer::new(
        "canvas-flow-tools",
        tool,
        view,
        steps.iter().map(Step::shape),
    )
    .selected(selected)
    .on_select(move |keys, _, cx| change(&chosen, cx, |flow| flow.selected = keys.to_vec()))
    .on_move(move |keys, (dx, dy), _, cx| {
        change(&moved, cx, |flow| {
            for step in flow
                .steps
                .iter_mut()
                .filter(|step| keys.contains(&step.key))
            {
                step.frame = Frame::new(
                    step.frame.x + dx,
                    step.frame.y + dy,
                    step.frame.w,
                    step.frame.h,
                );
            }
        })
    })
    .on_resize(move |key, frame, _, cx| {
        change(&sized, cx, |flow| {
            let ix = at(&mut flow.steps, key);
            flow.steps[ix].frame = frame;
        })
    })
    .on_link(move |from, to, _, cx| {
        change(&linked, cx, |flow| {
            let key = format!("made-{}", flow.links.len() + 1);
            flow.links.push(Link::new(key, from.clone(), to.clone()));
        })
    })
    .on_text(move |key, text, _, cx| {
        change(&written, cx, |flow| {
            let ix = at(&mut flow.steps, key);
            flow.steps[ix].text = text.clone();
        })
    });
    section(
        "FlowChart / DiagramEditor · Connector Line",
        "Steps joined by links that follow them: ends as ellipses, work as boxes, choices as diamonds, and words on a link. With Select a step moves, resizes, and takes new words on a double press; the connector draws a link from one step to another.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .items_start()
            .gap_2()
            .child(
                ToolPalette::new("canvas-flow-palette", tool)
                    .tools([Tool::Select, Tool::Connector])
                    .on_change(move |next, _, cx| change(&picked, cx, |flow| flow.tool = next)),
            )
            .child(probe(
                "canvas-flow",
                boxed(
                    620.0,
                    300.0,
                    FlowChart::new("canvas-flow-chart", view, steps, links)
                        .layer(layer)
                        .on_viewport(move |next, _, cx| change(&viewed, cx, |flow| flow.view = next)),
                    cx,
                ),
            )),
    )
}

/// The mind map demo's topics, selection, view and count of topics made.
struct Thinking {
    root: Topic,
    selected: Option<SharedString>,
    view: Viewport,
    made: usize,
}

impl Thinking {
    fn new() -> Self {
        let topic = Topic::new;
        Self {
            root: topic("root", "Product launch")
                .child(
                    topic("site", "Website")
                        .child(topic("copy", "Copy"))
                        .child(topic("shots", "Screenshots")),
                )
                .child(topic("press", "Press").child(topic("kit", "Press kit")))
                .child(topic("budget", "Budget"))
                .child(topic("team", "Team").child(topic("roles", "Roles"))),
            selected: Some("site".into()),
            view: Viewport::fitting(
                Frame::new(-520.0, -140.0, 1040.0, 280.0),
                (620.0, 300.0),
                16.0,
            ),
            made: 0,
        }
    }
}

pub fn mind(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("canvas-mind", Thinking::new, window, cx);
    let now = state.read(cx);
    let (root, selected, view) = (now.root.clone(), now.selected.clone(), now.view);
    let [picked, added, removed, renamed, viewed] = [(); 5].map(|_| state.clone());
    section(
        "MindMap",
        "Topics branching from a root, half to each side. A press selects; arrows walk, Tab adds a child and Enter a sibling, Delete removes one, and F2 or a double press rewrites it. The owner keeps the map.",
        cx,
    )
    .child(boxed(
        620.0,
        300.0,
        MindMap::new("canvas-mind-map", root, view)
            .selected(selected)
            .on_select(move |key, _, cx| change(&picked, cx, |mind| mind.selected = key.cloned()))
            .on_add(move |parent, _, cx| {
                change(&added, cx, |mind| {
                    mind.made += 1;
                    let key = SharedString::from(format!("idea-{}", mind.made));
                    if let Some(root) = mind.root.adding(parent, Topic::new(key.clone(), "New idea")) {
                        mind.root = root;
                        mind.selected = Some(key);
                    }
                })
            })
            .on_remove(move |key, _, cx| {
                change(&removed, cx, |mind| {
                    if let Some(root) = mind.root.removing(key) {
                        mind.selected = mind.root.parent_of(key);
                        mind.root = root;
                    }
                })
            })
            .on_rename(move |key, text, _, cx| {
                change(&renamed, cx, |mind| {
                    if let Some(root) = mind.root.renaming(key, text.clone()) {
                        mind.root = root;
                    }
                })
            })
            .on_viewport(move |next, _, cx| change(&viewed, cx, |mind| mind.view = next)),
        cx,
    ))
}
