use std::rc::Rc;

use ely_gpui_component::{
    git::{ConflictResolver, DiffLayout, DiffViewer, Take, ThreeWayMerge, regions},
    theme::{ActiveTheme, Radius},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const OLD: &str = r#"use std::collections::HashMap;

/// A palette of named colors.
pub struct Palette {
    colors: HashMap<String, Color>,
    fallback: Color,
}

impl Palette {
    pub fn new(fallback: Color) -> Self {
        Self { colors: HashMap::new(), fallback }
    }

    pub fn insert(&mut self, name: &str, color: Color) {
        self.colors.insert(name.to_string(), color);
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// The color for `name`.
    pub fn lookup(&self, name: &str) -> Color {
        self.colors[name]
    }
}
"#;

const NEW: &str = r#"use std::collections::BTreeMap;

/// A palette of named colors, in a steady order.
pub struct Palette {
    colors: BTreeMap<String, Color>,
    fallback: Color,
}

impl Palette {
    pub fn new(fallback: Color) -> Self {
        Self { colors: BTreeMap::new(), fallback }
    }

    pub fn insert(&mut self, name: &str, color: Color) {
        self.colors.insert(name.to_string(), color);
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// The color for `name`, blended toward white by `lift`.
    pub fn lookup(&self, name: &str, lift: f32) -> Color {
        let base = self.colors.get(name).copied().unwrap_or(self.fallback);
        base.lift(lift)
    }
}
"#;

pub fn viewer(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let layout = keep("git-layout", || DiffLayout::Unified, window, cx);
    let open = keep("git-open", Vec::<usize>::new, window, cx);
    let (now_layout, now_open) = (*layout.read(cx), open.read(cx).clone());
    let (flip, unfold) = (layout.clone(), open.clone());
    let theme = cx.theme();
    section(
        "DiffViewer / InlineDiff",
        "One file's changes, unified or side by side: each change in its context with old and new numbers, the words that changed washed, the stretches between folded until pressed. InlineDiff inside an editor is CodeEditor::set_diff.",
        cx,
    )
    .child(probe(
        "git-diff",
        div()
            .w(px(840.))
            .h(px(360.))
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .overflow_hidden()
            .child(
                DiffViewer::new("git-diff", "src/palette.rs", OLD, NEW)
                    .layout(now_layout)
                    .open(now_open)
                    .on_layout(move |next, _, cx| set(&flip, next, cx))
                    .on_open(move |stretch, _, cx| {
                        let mut next = unfold.read(cx).clone();
                        next.push(stretch);
                        set(&unfold, next, cx)
                    }),
            ),
    ))
}

const BASE: &str =
    "fn lift(color: Color) -> Color {\n    color.mix(WHITE, 0.5)\n}\n\nfn keep() {}\n";
const OURS: &str =
    "fn lift(color: Color) -> Color {\n    color.mix(WHITE, 0.6)\n}\n\nfn keep() {}\n";
const THEIRS: &str = "fn lift(color: Color) -> Color {\n    color.mix(WHITE, LIFT)\n}\n\nfn keep() { log::info!(\"kept\") }\n";

const MARKED: &str = "fn lift(color: Color) -> Color {\n<<<<<<< HEAD\n    color.mix(WHITE, 0.6)\n=======\n    color.mix(WHITE, LIFT)\n>>>>>>> feature/lift\n}\n\nfn keep() { log::info!(\"kept\") }\n";

pub fn merges(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let takes = keep("git-takes", Vec::<Option<Take>>::new, window, cx);
    let marked = keep("git-marked", || SharedString::from(MARKED), window, cx);
    let (now_takes, now_marked) = (takes.read(cx).clone(), marked.read(cx).clone());
    let (take, settle) = (takes.clone(), marked.clone());
    section(
        "ThreeWayMerge / ConflictResolver",
        "Ours, the base and theirs side by side: clean changes merge themselves, each conflict waits for a side, and the result shows what is still open. In a file with markers, each conflict offers the current change, the incoming one, or both.",
        cx,
    )
    .child(
        div().w(px(840.)).h(px(320.)).child(
            ThreeWayMerge::new("git-merge", ["main", "base", "feature/lift"], Rc::new(regions(BASE, OURS, THEIRS)))
                .takes(now_takes)
                .on_take(move |conflict, side, _, cx| {
                    let mut next = take.read(cx).clone();
                    if next.len() <= conflict {
                        next.resize(conflict + 1, None);
                    }
                    next[conflict] = Some(side);
                    set(&take, next, cx)
                }),
        ),
    )
    .child(
        div().w(px(840.)).child(
            ConflictResolver::new("git-conflicts", now_marked)
                .on_resolve(move |text, _, cx| set(&settle, SharedString::from(text), cx)),
        ),
    )
}
