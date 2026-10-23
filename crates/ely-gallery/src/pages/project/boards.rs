use ely_gpui_component::{
    mail::Label,
    project::{
        IssueCard, KanbanBoard, KanbanCard, KanbanColumn, LabelManager, Status, Task,
        TaskDetailPanel,
    },
    theme::{ActiveTheme, Radius},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::{board, quiet, studio_labels, team};
use crate::{
    probe::probe,
    ui::{change, keep, section, set},
};

/// The board's columns: a status each gathers, and the status a card dropped there takes.
const LANES: [(&str, &str, Status, &[Status]); 3] = [
    (
        "todo",
        "Todo",
        Status::Todo,
        &[Status::Backlog, Status::Todo],
    ),
    (
        "doing",
        "In progress",
        Status::InProgress,
        &[Status::InProgress, Status::InReview],
    ),
    (
        "done",
        "Done",
        Status::Done,
        &[Status::Done, Status::Canceled],
    ),
];

pub fn kanban(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tasks = board(window, cx);
    let listed = tasks.read(cx).clone();
    let opened = keep(
        "project-opened",
        || Some(SharedString::from("brief")),
        window,
        cx,
    );
    let columns = LANES.map(|(key, title, _, statuses)| {
        let cards = listed
            .iter()
            .filter(|task| statuses.contains(&task.status))
            .map(|task| {
                let face = IssueCard::new(
                    SharedString::from(format!("project-issue-{}", task.key)),
                    task.clone(),
                );
                KanbanCard::new(task.key.clone(), face)
            });
        KanbanColumn::new(key, title).cards(cards)
    });
    let moved = tasks.clone();
    let board = columns
        .into_iter()
        .fold(KanbanBoard::new("project-board"), |board, column| {
            board.column(column)
        })
        .on_move(move |key, column, index, _, cx| {
            let (_, _, status, statuses) = LANES
                .into_iter()
                .find(|(lane, ..)| *lane == column.as_ref())
                .expect("a lane of the board");
            change(&moved, cx, |tasks| {
                let at = tasks
                    .iter()
                    .position(|task| task.key == *key)
                    .expect("a task on the board");
                let mut task = tasks.remove(at);
                if !statuses.contains(&task.status) {
                    task.status = status;
                }
                let lane: Vec<usize> = (0..tasks.len())
                    .filter(|ix| statuses.contains(&tasks[*ix].status))
                    .collect();
                let place = lane
                    .get(index)
                    .copied()
                    .unwrap_or_else(|| lane.last().map_or(tasks.len(), |last| last + 1));
                tasks.insert(place, task);
            })
        })
        .on_open(move |key, _, cx| set(&opened, Some(key.clone()), cx));
    section(
        "KanbanBoard / KanbanColumn / KanbanCard · IssueCard · Sprint Board",
        "Columns of issue cards for a sprint. A card drags to a place in any column while the others glide aside, Option with an arrow moves the focused one, and a press or Enter opens it in the panel below. Sprint Board is this board.",
        cx,
    )
    .child(probe("project-board", div().child(board)))
}

pub fn panel(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tasks = board(window, cx);
    let opened = keep(
        "project-opened",
        || Some(SharedString::from("brief")),
        window,
        cx,
    );
    let steps = keep(
        "project-steps",
        || {
            vec![
                (SharedString::from("Gather the client's notes"), true),
                (SharedString::from("Draft the scope"), false),
                (SharedString::from("Share for comments"), false),
            ]
        },
        window,
        cx,
    );
    let shown = opened
        .read(cx)
        .clone()
        .and_then(|key| tasks.read(cx).iter().find(|task| task.key == key).cloned());
    let theme = cx.theme();
    let body = match shown {
        Some(task) => {
            let (edited, closed, ticked, added) =
                (tasks.clone(), opened.clone(), steps.clone(), steps.clone());
            TaskDetailPanel::new("project-panel", task, team())
                .notes("The client wants the brief before the site visit: scope, budget range and the three questions for the survey.")
                .steps(steps.read(cx).clone())
                .on_change(move |task: &Task, _, cx| {
                    let task = task.clone();
                    change(&edited, cx, |tasks| {
                        let at = tasks.iter().position(|each| each.key == task.key).expect("a listed task");
                        tasks[at] = task;
                    })
                })
                .on_close(move |_, cx| set(&closed, None, cx))
                .on_step(move |ix, done, _, cx| change(&ticked, cx, |steps| steps[ix].1 = done))
                .on_add_step(move |words, _, cx| {
                    let words = words.clone();
                    change(&added, cx, |steps| steps.push((words, false)))
                })
                .into_any_element()
        }
        None => quiet("Open a card on the board to see it here.".into(), cx).into_any_element(),
    };
    section(
        "TaskDetailPanel",
        "A task in full beside the board: its title edits in place, its properties change where they stand, and its notes and steps follow. Each edit shows on the board and in the list.",
        cx,
    )
    .child(probe(
        "project-panel",
        div()
            .w(px(400.))
            .p_5()
            .border_1()
            .border_color(theme.colors.border)
            .rounded(theme.radius(Radius::Lg))
            .child(body),
    ))
}

pub fn labels(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let labels = keep("project-labels", studio_labels, window, cx);
    let tasks = board(window, cx);
    let listed = labels.read(cx).clone();
    let (renamed, recolored, deleted, made) =
        (labels.clone(), labels.clone(), labels.clone(), labels);
    let (renamed_tasks, recolored_tasks, deleted_tasks) = (tasks.clone(), tasks.clone(), tasks);
    let manager = LabelManager::new("project-labels", listed)
        .on_rename(move |key, name, _, cx| {
            let name = name.clone();
            change(&renamed, cx, |labels| rename(labels.iter_mut(), key, &name));
            change(&renamed_tasks, cx, |tasks| {
                rename(
                    tasks.iter_mut().flat_map(|task| task.labels.iter_mut()),
                    key,
                    &name,
                )
            });
        })
        .on_recolor(move |key, hue, _, cx| {
            change(&recolored, cx, |labels| {
                recolor(labels.iter_mut(), key, hue)
            });
            change(&recolored_tasks, cx, |tasks| {
                recolor(
                    tasks.iter_mut().flat_map(|task| task.labels.iter_mut()),
                    key,
                    hue,
                )
            });
        })
        .on_delete(move |key, _, cx| {
            change(&deleted, cx, |labels| {
                labels.retain(|label| label.key != *key)
            });
            change(&deleted_tasks, cx, |tasks| {
                tasks
                    .iter_mut()
                    .for_each(|task| task.labels.retain(|label| label.key != *key))
            });
        })
        .on_create(move |name, _, cx| {
            let name = name.clone();
            change(&made, cx, |labels| {
                let key = SharedString::from(format!("label-{}", labels.len() + 1));
                let hue = labels.len() % 8;
                labels.push(Label::new(key, name, hue));
            })
        });
    section(
        "LabelManager",
        "A team's labels: a press on a hue picks another, a name edits in place, and the field at the end makes a new one. Names stay unique whatever their case, and each change shows on the cards.",
        cx,
    )
    .child(probe("project-label-manager", div().w(px(360.)).child(manager)))
}

fn rename<'a>(
    labels: impl Iterator<Item = &'a mut Label>,
    key: &SharedString,
    name: &SharedString,
) {
    labels
        .filter(|label| label.key == *key)
        .for_each(|label| label.name = name.clone());
}

fn recolor<'a>(labels: impl Iterator<Item = &'a mut Label>, key: &SharedString, hue: usize) {
    labels
        .filter(|label| label.key == *key)
        .for_each(|label| label.hue = hue);
}
