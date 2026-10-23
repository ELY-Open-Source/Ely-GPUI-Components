use ely_gpui_component::{
    forms::Choice,
    navigation::{EditorTab, EditorTabs, TabPlacement, Tabs},
    primitives::IconName,
    typography::Paragraph,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set, specimen, specimens},
};

fn panel(text: &'static str) -> impl IntoElement {
    div().py_2().child(Paragraph::new(text))
}

pub fn tabs(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let top = keep("tabs-top", || SharedString::from("overview"), window, cx);
    let left = keep("tabs-left", || SharedString::from("general"), window, cx);
    let bottom = keep("tabs-bottom", || SharedString::from("console"), window, cx);
    let (top_now, left_now, bottom_now) = (
        top.read(cx).clone(),
        left.read(cx).clone(),
        bottom.read(cx).clone(),
    );
    let top_panel = match top_now.as_ref() {
        "overview" => "Three services, all healthy. The last deploy went out an hour ago.",
        "activity" => "Twelve events since noon: four deploys, eight config reads.",
        _ => "Alerts go to the on-call channel. Deploys need one review.",
    };
    section(
        "Tabs",
        "A line slides to the chosen tab. Arrows move it; disabled tabs are skipped.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "top",
                probe(
                    "tabs-top",
                    div().w(px(420.0)).child(
                        Tabs::new(
                            "tabs-top",
                            [
                                Choice::new("overview", "Overview"),
                                Choice::new("activity", "Activity").note("12"),
                                Choice::new("settings", "Settings"),
                                Choice::new("archived", "Archived").disabled(),
                            ],
                            top_now,
                        )
                        .panel(panel(top_panel))
                        .on_change(move |next, _, cx| set(&top, next.clone(), cx)),
                    ),
                ),
                cx,
            ))
            .child(specimen(
                "left",
                div().w(px(420.0)).child(
                    Tabs::new(
                        "tabs-left",
                        [
                            Choice::new("general", "General").icon(IconName::Settings),
                            Choice::new("appearance", "Appearance").icon(IconName::Palette),
                            Choice::new("keys", "Keyboard").icon(IconName::Keyboard),
                        ],
                        left_now,
                    )
                    .placement(TabPlacement::Left)
                    .panel(panel(
                        "Settings for this section sit here, beside their tab.",
                    ))
                    .on_change(move |next, _, cx| set(&left, next.clone(), cx)),
                ),
                cx,
            ))
            .child(specimen(
                "bottom",
                div().w(px(420.0)).child(
                    Tabs::new(
                        "tabs-bottom",
                        [
                            Choice::new("console", "Console"),
                            Choice::new("output", "Output"),
                            Choice::new("problems", "Problems").note("3"),
                        ],
                        bottom_now,
                    )
                    .placement(TabPlacement::Bottom)
                    .panel(panel("Logs stream above their tabs, as a panel dock does."))
                    .on_change(move |next, _, cx| set(&bottom, next.clone(), cx)),
                ),
                cx,
            )),
    )
}

/// An open document, as the demo app keeps it.
#[derive(Clone)]
struct Doc {
    id: &'static str,
    title: &'static str,
    dirty: bool,
    pinned: bool,
    preview: bool,
}

fn open_docs() -> Vec<Doc> {
    let doc = |id, title| Doc {
        id,
        title,
        dirty: false,
        pinned: false,
        preview: false,
    };
    vec![
        Doc {
            pinned: true,
            ..doc("cargo", "Cargo.toml")
        },
        doc("main", "main.rs"),
        Doc {
            dirty: true,
            ..doc("lib", "lib.rs")
        },
        doc("theme", "theme.rs"),
        doc("tokens", "tokens.rs"),
        Doc {
            preview: true,
            ..doc("readme", "README.md")
        },
    ]
}

fn edit(docs: &gpui::Entity<Vec<Doc>>, id: &SharedString, cx: &mut App, change: impl Fn(&mut Doc)) {
    docs.update(cx, |docs, cx| {
        docs.iter_mut()
            .filter(|doc| doc.id == id.as_ref())
            .for_each(change);
        cx.notify();
    })
}

pub fn editor_tabs(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let docs = keep("editor-docs", open_docs, window, cx);
    let chosen = keep("editor-chosen", || SharedString::from("main"), window, cx);
    let tabs: Vec<EditorTab> = docs
        .read(cx)
        .iter()
        .map(|doc| {
            let tab = EditorTab::new(doc.id, doc.title)
                .dirty(doc.dirty)
                .pinned(doc.pinned)
                .preview(doc.preview);
            if doc.title.ends_with(".rs") {
                tab.icon(IconName::FileText)
            } else {
                tab
            }
        })
        .collect();
    let (closing, pinning, keeping, moving) = (docs.clone(), docs.clone(), docs.clone(), docs);
    let (picked, select) = (chosen.read(cx).clone(), chosen);
    section(
        "EditorTabs / TabOverflowMenu",
        "Close, pin, drag to reorder. A dot marks unsaved work until you hover; italics mark a preview. The chevron lists every tab.",
        cx,
    )
    .child(probe(
        "editor-tabs",
        div().w(px(560.0)).child(
            EditorTabs::new("editor-tabs", tabs)
                .selected(picked)
                .on_select(move |id, _, cx| set(&select, id.clone(), cx))
                .on_close(move |id, _, cx| {
                    closing.update(cx, |docs, cx| {
                        docs.retain(|doc| doc.id != id.as_ref());
                        cx.notify();
                    })
                })
                .on_pin(move |id, _, cx| edit(&pinning, id, cx, |doc| doc.pinned = false))
                .on_keep(move |id, _, cx| edit(&keeping, id, cx, |doc| doc.preview = false))
                .on_reorder(move |from, to, _, cx| {
                    moving.update(cx, |docs, cx| {
                        let doc = docs.remove(from);
                        docs.insert(to, doc);
                        cx.notify();
                    })
                }),
        ),
    ))
}
