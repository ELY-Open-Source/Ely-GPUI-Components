use std::time::Duration;

use ely_gpui_component::{
    lists::{DropAt, Outline, Tree, TreeNode, TreeSelect},
    primitives::IconName,
    theme::{ActiveTheme, Radius},
    typography::Caption,
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

/// A file in the demo project: key, name, parent, whether it is a folder, and whether its children are still to come.
#[derive(Clone)]
struct Item {
    key: SharedString,
    name: SharedString,
    parent: Option<SharedString>,
    folder: bool,
    pending: bool,
}

fn item(key: &str, name: &str, parent: Option<&str>, folder: bool) -> Item {
    Item {
        key: key.to_string().into(),
        name: name.to_string().into(),
        parent: parent.map(|parent| parent.to_string().into()),
        folder,
        pending: false,
    }
}

fn project() -> Vec<Item> {
    let mut assets = item("assets", "assets", Some("app"), true);
    assets.pending = true;
    vec![
        item("app", "ely-notes", None, true),
        item("src", "src", Some("app"), true),
        item("main", "main.rs", Some("src"), false),
        item("ui", "ui", Some("src"), true),
        item("sidebar", "sidebar.rs", Some("ui"), false),
        item("editor", "editor.rs", Some("ui"), false),
        assets,
        item("readme", "README.md", Some("app"), false),
        item("cargo", "Cargo.toml", Some("app"), false),
    ]
}

fn build(items: &[Item], parent: Option<&SharedString>) -> Vec<TreeNode> {
    items
        .iter()
        .filter(|item| item.parent.as_ref() == parent)
        .map(|item| {
            let node = TreeNode::new(item.key.clone(), item.name.clone()).icon(if item.folder {
                IconName::Folder
            } else {
                IconName::FileText
            });
            match (item.folder, item.pending) {
                (true, true) => node.pending(),
                (true, false) => node.children(build(items, Some(&item.key))),
                _ => node,
            }
        })
        .collect()
}

/// Moves `key` before, after or into `target`, keeping sibling order by list order.
fn moved(items: &mut Vec<Item>, key: &SharedString, target: &SharedString, at: DropAt) {
    let from = items
        .iter()
        .position(|item| item.key == *key)
        .expect("a known file");
    let mut item = items.remove(from);
    let to = items
        .iter()
        .position(|item| item.key == *target)
        .expect("a known target");
    match at {
        DropAt::Inside => {
            item.parent = Some(target.clone());
            items.push(item);
        }
        DropAt::Before | DropAt::After => {
            item.parent = items[to].parent.clone();
            items.insert(if at == DropAt::Before { to } else { to + 1 }, item);
        }
    }
}

fn change(state: &Entity<Vec<Item>>, cx: &mut App, edit: impl FnOnce(&mut Vec<Item>)) {
    let mut next = state.read(cx).clone();
    edit(&mut next);
    set(state, next, cx);
}

pub fn tree(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let files = keep("tree-files", project, window, cx);
    let picked = keep("tree-picked", Vec::<SharedString>::new, window, cx);
    let nodes = build(files.read(cx), None);
    let (now, pick) = (picked.read(cx).clone(), picked.clone());
    let (renamed, dropped, loading) = (files.clone(), files.clone(), files);
    let theme = cx.theme();
    section(
        "Tree / TreeView / VirtualTree",
        "Folders open and close; Right, Left, Up and Down walk them, F2 renames, and a drag lands before, after or into a folder. Shift and Cmd widen the pick. Assets loads its children the first time it opens. Rows draw only in view, so large trees stay quick.",
        cx,
    )
    .child(probe(
        "tree",
        div()
            .w(px(320.))
            .h(px(300.))
            .p_1()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .child(
                Tree::new("tree", nodes)
                    .open(["app", "src"])
                    .multiple()
                    .selected(now.clone())
                    .on_select(move |keys, _, cx| set(&pick, keys.to_vec(), cx))
                    .on_rename(move |key, name, _, cx| {
                        change(&renamed, cx, |items| {
                            if let Some(item) = items.iter_mut().find(|item| item.key == *key) {
                                item.name = name.clone();
                            }
                        })
                    })
                    .on_move(move |key, target, at, _, cx| change(&dropped, cx, |items| moved(items, key, target, at)))
                    .on_load(move |key, _, cx| {
                        let (loading, key) = (loading.clone(), key.clone());
                        cx.spawn(async move |cx| {
                            cx.background_executor().timer(Duration::from_millis(800)).await;
                            cx.update(|cx| {
                                change(&loading, cx, |items| {
                                    if let Some(folder) = items.iter_mut().find(|item| item.key == key) {
                                        folder.pending = false;
                                    }
                                    items.push(item("logo", "logo.svg", Some(&key), false));
                                    items.push(item("inter", "Inter.ttf", Some(&key), false));
                                })
                            });
                        })
                        .detach();
                    })
                    .size_full(),
            ),
    ))
    .child(Caption::new(match now.len() {
        0 => "Nothing picked.".to_string(),
        _ => format!("Picked: {}", now.join(", ")),
    }))
}

pub fn checkbox(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let granted = keep("grants", || vec![SharedString::from("read")], window, cx);
    let (now, store) = (granted.read(cx).clone(), granted.clone());
    let branch =
        |key: &'static str, name: &'static str, leaves: [(&'static str, &'static str); 2]| {
            TreeNode::new(key, name).children(leaves.map(|(key, name)| TreeNode::new(key, name)))
        };
    let theme = cx.theme();
    section(
        "CheckboxTree",
        "A box on every row. A branch's box checks all it holds, and shows a dash when only some are.",
        cx,
    )
    .child(
        div()
            .w(px(320.))
            .h(px(200.))
            .p_1()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .child(
                Tree::new(
                    "grants-tree",
                    [
                        branch("documents", "Documents", [("read", "Read"), ("write", "Write")]),
                        branch("billing", "Billing", [("invoices", "See invoices"), ("plan", "Change the plan")]),
                    ],
                )
                .open(["documents", "billing"])
                .checked(now.clone())
                .on_check(move |keys, _, cx| set(&store, keys.to_vec(), cx))
                .size_full(),
            ),
    )
    .child(Caption::new(format!("Granted: {}", now.join(", "))))
}

pub fn select(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let team = keep("team", || SharedString::from("apps"), window, cx);
    let (now, store) = (team.read(cx).clone(), team.clone());
    section(
        "TreeSelect",
        "A select whose choices are a tree, each indented under its parent.",
        cx,
    )
    .child(
        div().w(px(260.)).child(
            TreeSelect::new(
                "team-select",
                [
                    TreeNode::new("design", "Design")
                        .icon(IconName::PenTool)
                        .children([
                            TreeNode::new("brand", "Brand"),
                            TreeNode::new("product", "Product"),
                        ]),
                    TreeNode::new("engineering", "Engineering")
                        .icon(IconName::Code)
                        .children([
                            TreeNode::new("platform", "Platform"),
                            TreeNode::new("apps", "Apps").children([
                                TreeNode::new("ios", "iOS"),
                                TreeNode::new("macos", "macOS"),
                            ]),
                        ]),
                    TreeNode::new("operations", "Operations").icon(IconName::Wrench),
                ],
            )
            .selected(now)
            .on_change(move |key, _, cx| set(&store, key.clone(), cx)),
        ),
    )
}

pub fn outline(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let current = keep(
        "outline-current",
        || SharedString::from("install"),
        window,
        cx,
    );
    let (now, store) = (current.read(cx).clone(), current.clone());
    const HEADINGS: [(&str, &str, usize); 8] = [
        ("intro", "Introduction", 1),
        ("install", "Installation", 2),
        ("needs", "Requirements", 3),
        ("start", "Quick start", 2),
        ("components", "Components", 1),
        ("buttons", "Buttons", 2),
        ("forms", "Forms", 2),
        ("theme", "Theming", 1),
    ];
    section(
        "Outline / NestedList",
        "A document's headings, each level stepping in. The marker glides to the section you pick.",
        cx,
    )
    .child(probe(
        "outline",
        div().w(px(240.)).child(
            HEADINGS
                .iter()
                .fold(Outline::new("outline"), |outline, (key, title, level)| {
                    outline.item(*key, *title, *level)
                })
                .current(now)
                .on_select(move |key, _, cx| set(&store, key.clone(), cx)),
        ),
    ))
}
