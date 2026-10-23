use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    layout::{Dock, DockPanel, DockSide, FloatingPanel, PaneGroup, Transform, Viewport, Workspace},
    primitives::IconName,
    theme::{ActiveTheme, Radius, TextSize},
    typography::{Caption, Paragraph},
};
use gpui::{
    AnyElement, App, Axis, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div,
    point, prelude::*, px,
};

use super::{basics::tile, shells::frame};
use crate::{
    probe::probe,
    ui::{code, row, section},
};

fn demo_dock(cx: &mut App) -> Dock {
    let panel = |id: &str, title: &str, icon| DockPanel {
        id: SharedString::from(id.to_string()),
        title: SharedString::from(title.to_string()),
        icon,
    };
    let mut dock = Dock::new(
        |_, cx| {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(cx.theme().colors.fg_subtle)
                .child("Center. Drag a tab onto a zone.")
                .into_any_element()
        },
        |id, _, cx| {
            div()
                .p_3()
                .text_size(cx.theme().text_size(TextSize::Sm))
                .text_color(cx.theme().colors.fg_muted)
                .child(SharedString::from(format!("{id} panel")))
                .into_any_element()
        },
        cx,
    );
    dock.add(panel("files", "Files", IconName::Folder), DockSide::Left);
    dock.add(
        panel("outline", "Outline", IconName::Layers),
        DockSide::Left,
    );
    dock.add(panel("search", "Search", IconName::Search), DockSide::Right);
    dock.add(
        panel("terminal", "Terminal", IconName::Terminal),
        DockSide::Bottom,
    );
    dock
}

fn demo_panes(cx: &mut Context<PaneGroup>) -> PaneGroup {
    let mut group = PaneGroup::new(|id, _, cx| {
        div()
            .size_full()
            .p_3()
            .text_color(cx.theme().colors.fg_subtle)
            .child(SharedString::from(format!("buffer {id}")))
            .into_any_element()
    });
    group.split(1, Axis::Horizontal, cx);
    group
}

/// The dock and pane group shared by the demos and the workspace section.
fn systems(window: &mut Window, cx: &mut App) -> (Entity<Dock>, Entity<PaneGroup>) {
    let dock = window.use_keyed_state("demo-dock", cx, |_, cx| demo_dock(cx));
    let panes = window.use_keyed_state("demo-panes", cx, |_, cx| demo_panes(cx));
    (dock, panes)
}

pub fn dock(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (dock, _) = systems(window, cx);
    section(
        "Dock / DockPanel",
        "Panels dock on any edge, tab together, and float.",
        cx,
    )
    .child(probe("dock", frame(760.0, 380.0, cx).child(dock)))
}

pub fn dock_zones(cx: &App) -> impl IntoElement + use<> {
    section(
        "DockZone / DropIndicator",
        "While a tab drags, the drop zones appear and the one below lights.",
        cx,
    )
    .child(Caption::new(
        "Drag a tab across the dock above to see them.",
    ))
}

pub fn floating(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = window.use_keyed_state("floating-open", cx, |_, _| false);
    let shown = *open.read(cx);
    section(
        "FloatingPanel",
        "Drag the title bar. It stays inside the window.",
        cx,
    )
    .child(
        row()
            .child(probe(
                "floating-open",
                Button::new(
                    "floating-open",
                    if shown { "Hide panel" } else { "Show panel" },
                )
                .on_click(move |_, _, cx| {
                    open.update(cx, |open, cx| {
                        *open = !*open;
                        cx.notify();
                    })
                }),
            ))
            .when(shown, |row| {
                row.child(
                    FloatingPanel::new("demo-float", "Inspector", point(px(720.0), px(160.0)))
                        .w(px(240.0))
                        .child(Caption::new("Floating panels sit above the page."))
                        .child(tile("Opacity 100%", cx)),
                )
            }),
    )
}

pub fn panes(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (_, panes) = systems(window, cx);
    section(
        "Pane / PaneGroup",
        "Split right or down from any pane; close all but one.",
        cx,
    )
    .child(probe("panes", frame(760.0, 300.0, cx).child(panes)))
}

pub fn workspace(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (dock, panes) = systems(window, cx);
    let saved = window.use_keyed_state("workspace-json", cx, |_, _| None::<String>);
    let report = window.use_keyed_state("workspace-report", cx, |_, _| SharedString::from(""));
    let preview: Option<String> = saved.read(cx).clone();
    let note = report.read(cx).clone();
    let (save_dock, save_panes, save_to) = (dock.clone(), panes.clone(), saved.clone());
    let (load_from, load_note) = (saved, report.clone());
    section(
        "Workspace",
        "Layout to versioned JSON and back, all or nothing.",
        cx,
    )
    .child(
        row()
            .child(probe(
                "workspace-save",
                Button::new("workspace-save", "Save layout")
                    .icon(IconName::Save)
                    .on_click(move |_, _, cx| {
                        let json =
                            Workspace::capture(save_dock.read(cx), save_panes.read(cx)).to_json();
                        save_to.update(cx, |saved, cx| {
                            *saved = Some(json);
                            cx.notify();
                        })
                    }),
            ))
            .child(probe(
                "workspace-restore",
                Button::new("workspace-restore", "Restore")
                    .variant(ButtonVariant::Ghost)
                    .icon(IconName::RotateCcw)
                    .disabled(preview.is_none())
                    .on_click(move |_, _, cx| {
                        let json = load_from
                            .read(cx)
                            .clone()
                            .expect("restore is disabled until a layout is saved");
                        let outcome = Workspace::from_json(&json)
                            .and_then(|layout| layout.apply(&dock, &panes, cx));
                        let text = match outcome {
                            Ok(()) => "Restored.".to_string(),
                            Err(error) => format!("Refused: {error:#}"),
                        };
                        load_note.update(cx, |note, cx| {
                            *note = SharedString::from(text);
                            cx.notify();
                        })
                    }),
            ))
            .child(Caption::new(note)),
    )
    .when_some(preview, |block, json| {
        let excerpt: String = json.lines().take(12).collect::<Vec<_>>().join("\n");
        block.child(code(excerpt, cx))
    })
}

fn blob(
    at: gpui::Point<gpui::Pixels>,
    size: f32,
    transform: Transform,
    label: &'static str,
    cx: &App,
) -> AnyElement {
    let theme = cx.theme();
    let local = transform.apply(at);
    div()
        .absolute()
        .left(local.x)
        .top(local.y)
        .w(px(size) * transform.scale)
        .h(px(size * 0.6) * transform.scale)
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border_strong)
        .bg(theme.colors.surface)
        .flex()
        .items_center()
        .justify_center()
        .text_color(theme.colors.fg_muted)
        .child(label)
        .into_any_element()
}

pub fn viewport(cx: &App) -> impl IntoElement + use<> {
    section(
        "Viewport",
        "Drag to pan. Hold ⌘ or Ctrl and scroll to zoom at the pointer.",
        cx,
    )
    .child(probe(
        "viewport",
        frame(640.0, 260.0, cx)
            .bg(cx.theme().colors.sunken)
            .child(Viewport::new("demo-viewport", |transform, _, cx| {
                div()
                    .size_full()
                    .child(blob(
                        point(px(60.0), px(50.0)),
                        140.0,
                        transform,
                        "Idea",
                        cx,
                    ))
                    .child(blob(
                        point(px(260.0), px(110.0)),
                        140.0,
                        transform,
                        "Draft",
                        cx,
                    ))
                    .child(blob(
                        point(px(460.0), px(40.0)),
                        120.0,
                        transform,
                        "Ship",
                        cx,
                    ))
                    .into_any_element()
            })),
    ))
    .child(
        Paragraph::new("Content places itself with the transform; zoom stays between 0.1 and 8.")
            .max_w(px(560.0)),
    )
}
