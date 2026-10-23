use ely_gpui_component::{
    data_display::{DescriptionList, Tone},
    tables::{Cell, Column, DataTable, Row},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px, rems};

use super::core::{order_columns, order_rows};
use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn wide(cx: &mut App) -> impl IntoElement + use<> {
    let columns = std::iter::once(Column::new("team", "Team").width(rems(10.)).pinned()).chain(
        MONTHS
            .iter()
            .map(|month| Column::new(*month, *month).width(rems(5.5)).end()),
    );
    let rows = ["Design", "Platform", "Apps", "Growth", "Support"]
        .iter()
        .enumerate()
        .map(|(ix, team)| {
            Row::new(
                *team,
                std::iter::once(Cell::from(*team)).chain(
                    (0..12).map(|month| Cell::Number(((ix * 7 + month * 13) % 40 + 10) as f64)),
                ),
            )
        });
    section(
        "ColumnPinning / ColumnResizer / ColumnReorder",
        "Team stays put while the months scroll sideways. Drag a header's right edge to resize it; drag a header onto another to move it there.",
        cx,
    )
    .child(div().w(px(620.)).child(DataTable::new("wide", columns).rows(rows.collect::<Vec<_>>())))
}

/// Tasks the demo edits in place: key, name, owner, state.
fn tasks() -> Vec<(SharedString, SharedString, &'static str, &'static str)> {
    [
        ("t1", "Draft the launch notes", "Ada Lovelace", "Doing"),
        ("t2", "Review the type scale", "Grace Hopper", "Done"),
        ("t3", "Record the demo", "Alan Turing", "Next"),
    ]
    .map(|(key, name, owner, state)| (key.into(), name.into(), owner, state))
    .to_vec()
}

pub fn rows(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let kept = keep("tasks", tasks, window, cx);
    let (now, store) = (kept.read(cx).clone(), kept.clone());
    let rows: Vec<Row> = now
        .iter()
        .map(|(key, name, owner, state)| {
            let tone = match *state {
                "Done" => Tone::Success,
                "Doing" => Tone::Info,
                _ => Tone::Neutral,
            };
            Row::new(
                key.clone(),
                [
                    Cell::Text(name.clone()),
                    Cell::Person((*owner).into()),
                    Cell::Tag((*state).into(), tone),
                ],
            )
        })
        .collect();
    let about = now.clone();
    section(
        "RowExpansion / InlineRowEdit",
        "The chevron opens a row's detail beneath it. Double press a task's name to edit it; Enter keeps the change, Escape drops it.",
        cx,
    )
    .child(probe(
        "tasks",
        div().w(px(640.)).child(
            DataTable::new(
                "tasks",
                [Column::new("name", "Task").editable(), Column::new("owner", "Owner").width(rems(12.)), Column::new("state", "State").width(rems(6.))],
            )
            .rows(rows)
            .detail(move |key, _, _| {
                let (_, name, owner, state) = about.iter().find(|task| task.0 == *key).expect("a listed task").clone();
                DescriptionList::new()
                    .item("Task", name)
                    .item("Owner", owner)
                    .item("State", state)
                    .into_any_element()
            })
            .on_edit(move |key, _, text, _, cx| {
                let mut next = store.read(cx).clone();
                if let Some(task) = next.iter_mut().find(|task| task.0 == *key) {
                    task.1 = text.clone();
                }
                set(&store, next, cx)
            }),
        ),
    ))
}

pub fn grouped(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "RowGrouping",
        "Orders gathered by region, each group with its count; a press on a group folds it.",
        cx,
    )
    .child(probe(
        "grouped",
        div().w(px(900.)).child(
            DataTable::new("grouped", order_columns())
                .rows(order_rows().into_iter().take(9).collect::<Vec<_>>())
                .group_by("region"),
        ),
    ))
    .child(Caption::new(
        "Groups keep the order their first rows come in; sort to reorder them.",
    ))
}
