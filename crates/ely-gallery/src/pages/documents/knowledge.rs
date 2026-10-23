use std::rc::Rc;

use ely_gpui_component::{
    charts::NetworkGraph,
    data_display::{PropertyGrid, PropertyGroup},
    documents::{
        Backlink, Backlinks, Favorite, Favorites, PageCover, PageIcon, Template, TemplatePicker,
        TrashBin, Trashed, Version, VersionHistory,
    },
    forms::{Checkbox, Choice, DatePicker, Select, TagInput},
    lists::{Tree, TreeNode},
    theme::{ActiveTheme, Radius, TextSize},
};
use gpui::{App, FontWeight, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, ToSpan};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const COVERS: [&str; 3] = [
    asset!("atrium.jpg"),
    asset!("dunes.jpg"),
    asset!("atrium-olive.jpg"),
];

fn favorite(key: &str, icon: &str, title: &str) -> Favorite {
    Favorite {
        key: key.to_string().into(),
        icon: icon.to_string().into(),
        title: title.to_string().into(),
    }
}

/// A page linking here, the link on the word "lift".
fn backlink(icon: &str, page: &str, context: &str) -> Backlink {
    let at = context.find("lift").expect("the context names the page");
    Backlink {
        icon: icon.to_string().into(),
        page: page.to_string().into(),
        context: context.to_string().into(),
        mention: at..at + "lift".len(),
    }
}

fn pages() -> Vec<TreeNode> {
    vec![
        TreeNode::new("handbook", "📘 Handbook").children([
            TreeNode::new("color", "🎨 Color").children([
                TreeNode::new("lift", "☀️ Lift"),
                TreeNode::new("dark", "🌙 Dark theme"),
            ]),
            TreeNode::new("type", "🔤 Type"),
            TreeNode::new("motion", "🌀 Motion"),
        ]),
        TreeNode::new("projects", "🗂 Projects").children([
            TreeNode::new("site", "🌐 Site"),
            TreeNode::new("release", "🚀 Release 1.0"),
        ]),
        TreeNode::new("journal", "📓 Journal"),
    ]
}

fn templates() -> Rc<Vec<Template>> {
    let template =
        |name: &str, icon: &str, category: &str, description: &str, markdown: &str| Template {
            name: name.to_string().into(),
            icon: icon.to_string().into(),
            category: category.to_string().into(),
            description: description.to_string().into(),
            markdown: markdown.to_string().into(),
        };
    Rc::new(vec![
        template(
            "Meeting notes",
            "🗒",
            "Team",
            "Agenda, notes and actions",
            "# Meeting notes\n\n**Date:** \n\n## Agenda\n\n- \n\n## Actions\n\n- [ ] ",
        ),
        template(
            "Retrospective",
            "🔁",
            "Team",
            "What went well, what to change",
            "# Retrospective\n\n## Went well\n\n- \n\n## To change\n\n- ",
        ),
        template(
            "Roadmap",
            "🗺",
            "Product",
            "The quarters ahead",
            "# Roadmap\n\n| Quarter | Goal |\n|:--|:--|\n| Q1 | |\n| Q2 | |",
        ),
        template(
            "Design review",
            "🎨",
            "Product",
            "A change and its questions",
            "# Design review\n\n> What problem does this solve?\n\n## Options\n\n1. \n2. ",
        ),
        template(
            "Reading list",
            "📚",
            "Personal",
            "Books and where you are",
            "# Reading list\n\n- [ ] \n- [ ] ",
        ),
    ])
}

fn versions(now: Timestamp) -> Vec<Version> {
    let version = |hours: i64, author: &str, markdown: &str| Version {
        at: now - hours.hours(),
        author: author.to_string().into(),
        markdown: markdown.to_string().into(),
    };
    vec![
        version(
            1,
            "Ada",
            "# Lift\n\nA lift blends every color toward white, gently and in gamma space. Half a lift sits midway.",
        ),
        version(
            26,
            "Grace",
            "# Lift\n\nA lift blends a color toward white in gamma space. Half a lift sits midway.",
        ),
        version(74, "Ada", "# Lift\n\nA lift blends a color toward white."),
    ]
}

pub fn knowledge(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let favorites = keep(
        "know-favorites",
        || {
            vec![
                favorite("lift", "☀️", "Lift"),
                favorite("release", "🚀", "Release 1.0"),
                favorite("journal", "📓", "Journal"),
            ]
        },
        window,
        cx,
    );
    let now_favorites = favorites.read(cx).clone();
    let cover = keep(
        "know-cover",
        || Some(SharedString::from(COVERS[0])),
        window,
        cx,
    );
    let icon = keep("know-icon", || SharedString::from("☀️"), window, cx);
    let status = keep("know-status", || SharedString::from("Drafting"), window, cx);
    let done = keep("know-done", || false, window, cx);
    let due = keep("know-due", || jiff::civil::date(2026, 10, 12), window, cx);
    let tags = keep(
        "know-tags",
        || vec![SharedString::from("color"), SharedString::from("theme")],
        window,
        cx,
    );
    let (now_cover, now_icon, now_status, now_done, now_due, now_tags) = (
        cover.read(cx).clone(),
        icon.read(cx).clone(),
        status.read(cx).clone(),
        *done.read(cx),
        *due.read(cx),
        tags.read(cx).clone(),
    );
    let trash = keep(
        "know-trash",
        || {
            let now = Timestamp::now();
            vec![
                Trashed {
                    key: "old-palette".into(),
                    icon: "🎨".into(),
                    title: "Old palette".into(),
                    place: "Handbook".into(),
                    deleted: now - 3.hours(),
                },
                Trashed {
                    key: "draft".into(),
                    icon: "📝".into(),
                    title: "Launch draft".into(),
                    place: "Projects".into(),
                    deleted: now - 50.hours(),
                },
            ]
        },
        window,
        cx,
    );
    let now_trash = trash.read(cx).clone();
    let picked_version = keep("know-version", || 0usize, window, cx);
    let now_version = *picked_version.read(cx);
    let history = keep("know-history", || versions(Timestamp::now()), window, cx)
        .read(cx)
        .clone();
    let theme = cx.theme();
    let (unpinned, moved, restored, deleted, emptied) = (
        favorites.clone(),
        favorites.clone(),
        trash.clone(),
        trash.clone(),
        trash,
    );
    let graph = [
        "Lift",
        "Dark theme",
        "Color",
        "Type",
        "Motion",
        "Handbook",
        "Release 1.0",
        "Site",
    ]
    .iter()
    .enumerate()
    .fold(NetworkGraph::new("know-graph"), |graph, (ix, name)| {
        graph.node(*name, if ix < 5 { 0 } else { 1 })
    });
    let graph = [
        (0, 1),
        (0, 2),
        (1, 2),
        (2, 5),
        (3, 5),
        (4, 5),
        (0, 4),
        (6, 5),
        (6, 7),
        (7, 0),
    ]
    .into_iter()
    .fold(graph, |graph, (a, b)| graph.edge(a, b));
    div()
        .child(
            section(
                "PageTree / Favorites / Pinned",
                "Pages kept close at hand above the tree of every page: favorites open with a press, drag to reorder and let go by their star. The tree is lists::Tree, a page's emoji leading its label.",
                cx,
            )
            .child(
                div()
                    .w(px(300.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_2()
                    .rounded(theme.radius(Radius::Md))
                    .border_1()
                    .border_color(theme.colors.border)
                    .child(
                        Favorites::new("know-favorites", now_favorites)
                            .on_open(|key, _, _| log::info!("gallery: open {key}"))
                            .on_unpin(move |key, _, cx| {
                                let next = unpinned.read(cx).iter().filter(|page| page.key != *key).cloned().collect();
                                set(&unpinned, next, cx);
                            })
                            .on_move(move |from, to, _, cx| {
                                let mut next = moved.read(cx).clone();
                                let page = next.remove(from);
                                next.insert(to, page);
                                set(&moved, next, cx);
                            }),
                    )
                    .child(div().h(px(220.)).child(Tree::new("know-tree", pages())
                            .open(["handbook", "color"])
                            .selected(["lift"])
                            .size_full())),
            ),
        )
        .child(
            section(
                "PageCover / PageIcon / PageProperties",
                "A page's head: its cover, its icon set large over the cover's edge, its title, and its properties. Hover the cover to change or remove it, the icon to pick another. The properties are data_display::PropertyGrid, a field's editor in each row.",
                cx,
            )
            .child(
                div()
                    .w(px(760.))
                    .flex()
                    .flex_col()
                    .rounded(theme.radius(Radius::Lg))
                    .border_1()
                    .border_color(theme.colors.border)
                    .overflow_hidden()
                    .children(now_cover.map(|source| {
                        let cover = cover.clone();
                        PageCover::new("know-cover", source).choices(COVERS, move |next, _, cx| set(&cover, next, cx))
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .px_8()
                            .pb_6()
                            .child(div().mt_neg_8().child(PageIcon::new("know-icon", now_icon).on_change(move |next, _, cx| set(&icon, next.clone(), cx))))
                            .child(div().text_size(theme.text_size(TextSize::Xxl)).font_weight(FontWeight::SEMIBOLD).child("Lift"))
                            .child(
                                PropertyGrid::new("know-properties").group(
                                    PropertyGroup::new("Properties")
                                        .row(
                                            "Status",
                                            Select::new("know-status", ["Idea", "Drafting", "Review", "Done"].map(|name| Choice::new(name, name)))
                                                .selected(now_status)
                                                .on_change(move |next, _, cx| set(&status, next.clone(), cx)),
                                        )
                                        .row("Due", DatePicker::new("know-due", Some(now_due)).on_change(move |next, _, cx| set(&due, next, cx)))
                                        .row("Tags", TagInput::new("know-tags", now_tags).on_change(move |next, _, cx| set(&tags, next, cx)))
                                        .row("Published", Checkbox::new("know-published", now_done).on_change(move |on, _, cx| set(&done, on, cx))),
                                ),
                            ),
                    ),
            ),
        )
        .child(
            section(
                "Backlinks / GraphView",
                "The pages that link here, each with the words around its link; and every page as a node, its links as edges. GraphView is charts::NetworkGraph.",
                cx,
            )
            .child(
                div()
                    .flex()
                    .gap_6()
                    .child(
                        div().w(px(360.)).child(
                            Backlinks::new(
                                "know-backlinks",
                                [
                                    backlink("🌙", "Dark theme", "Accents in dark carry more lift than in light, so they read the same."),
                                    backlink("🎨", "Color", "Every accent is a base color and a lift."),
                                    backlink("🌀", "Motion", "A lift of motion is a step on the duration scale."),
                                ],
                            )
                            .on_open(|ix, _, _| log::info!("gallery: backlink {ix}")),
                        ),
                    )
                    .child(div().w(px(360.)).h(px(280.)).child(graph)),
            ),
        )
        .child(
            section(
                "TemplatePicker",
                "Pages to start from: found by name, kept by category, the one in view shown as it will read; Use takes it.",
                cx,
            )
            .child(div().w(px(760.)).h(px(400.)).child(TemplatePicker::new("know-templates", templates()).on_use(|ix, _, _| log::info!("gallery: template {ix}")))),
        )
        .child(
            section(
                "TrashBin",
                "Deleted pages, newest first, with where each lived and when it went: restore one, delete one for good or empty it all, each final step asked twice.",
                cx,
            )
            .child(
                div().w(px(560.)).child(
                    TrashBin::new("know-trash", now_trash)
                        .on_restore(move |key, _, cx| {
                            let next = restored.read(cx).iter().filter(|page| page.key != *key).cloned().collect();
                            set(&restored, next, cx);
                        })
                        .on_delete(move |key, _, cx| {
                            let next = deleted.read(cx).iter().filter(|page| page.key != *key).cloned().collect();
                            set(&deleted, next, cx);
                        })
                        .on_empty(move |_, cx| set(&emptied, Vec::new(), cx)),
                ),
            ),
        )
        .child(
            section(
                "VersionHistory / PageHistoryDiff",
                "A page's versions by time and author: read one, or see word by word what it changed from the one before, added words washed and removed ones struck; restore any but the current.",
                cx,
            )
            .child(
                probe("know-versions", div().w(px(760.)).h(px(320.)).child(
                    VersionHistory::new("know-versions", history, now_version)
                        .on_select(move |ix, _, cx| set(&picked_version, ix, cx))
                        .on_restore(|ix, _, _| log::info!("gallery: restore version {ix}")),
                )),
            ),
        )
}
