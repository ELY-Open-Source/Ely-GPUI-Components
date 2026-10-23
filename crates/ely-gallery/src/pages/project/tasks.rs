use ely_gpui_component::{
    documents::Checklist,
    forms::DatePicker,
    project::{
        AssigneePicker, IssueId, IssueIdBadge, Priority, PriorityIndicator, Status, StatusSelect,
        TaskList,
    },
    theme::{ActiveTheme, TextSize},
};
use gpui::{AnyElement, App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::{board, quiet, team};
use crate::{
    probe::probe,
    ui::{change, keep, section, set},
};

pub fn marks(cx: &mut App) -> impl IntoElement + use<> {
    let priorities = [
        Priority::Urgent,
        Priority::High,
        Priority::Medium,
        Priority::Low,
        Priority::None,
    ];
    section(
        "PriorityIndicator / IssueIdBadge",
        "How urgent a task is in three bars, urgent in red, its words in a tip or beside it. An issue goes by its team's key and its number.",
        cx,
    )
    .child(probe(
        "project-marks",
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_6()
            .children(priorities.map(|priority| {
                let id = SharedString::from(format!("project-priority-{priority:?}"));
                PriorityIndicator::new(id, priority).labeled()
            }))
            .child(IssueIdBadge::new(IssueId::new("ARC", 41)))
            .child(IssueIdBadge::new(IssueId::new("ENG", 1204))),
    ))
}

/// A label and its control, the label in a fixed column.
fn property(label: &'static str, control: AnyElement, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .w(px(88.))
                .flex_none()
                .text_size(theme.text_size(TextSize::Sm))
                .text_color(theme.colors.fg_muted)
                .child(label),
        )
        .child(div().flex_1().min_w_0().child(control))
}

pub fn fields(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tasks = board(window, cx);
    let task = tasks
        .read(cx)
        .iter()
        .find(|task| task.key.as_ref() == "brief")
        .cloned()
        .expect("the brief is listed");
    let edit = move |cx: &mut App, apply: &dyn Fn(&mut ely_gpui_component::project::Task)| {
        change(&tasks, cx, |tasks| {
            let brief = tasks
                .iter_mut()
                .find(|task| task.key.as_ref() == "brief")
                .expect("the brief is listed");
            apply(brief)
        })
    };
    let (moved, handed, dated) = (edit.clone(), edit.clone(), edit);
    let status = StatusSelect::new("project-status", task.status)
        .on_change(move |status, _, cx| moved(cx, &|task| task.status = status));
    let assignee = task.assignee.as_ref().map(|person| person.key.clone());
    let picker = AssigneePicker::new("project-assignee", team()).on_change(move |person, _, cx| {
        let person = person.cloned();
        handed(cx, &|task| task.assignee = person.clone())
    });
    let picker = match assignee {
        Some(key) => picker.assignee(key),
        None => picker,
    };
    let due = DatePicker::new("project-due", task.due)
        .on_change(move |day, _, cx| dated(cx, &|task| task.due = Some(day)));
    section(
        "StatusSelect / AssigneePicker / DueDatePicker",
        "A task's properties: where it stands in its workflow, who has it, and when it is due; DueDatePicker is the forms DatePicker. Each change shows in the list below.",
        cx,
    )
    .child(probe(
        "project-fields",
        div()
            .w(px(360.))
            .flex()
            .flex_col()
            .gap_3()
            .child(property("Status", status.into_any_element(), cx))
            .child(property("Assignee", picker.into_any_element(), cx))
            .child(property("Due", due.into_any_element(), cx)),
    ))
}

pub fn list(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tasks = board(window, cx);
    let listed = tasks.read(cx).clone();
    let said = keep("project-list-said", || None::<SharedString>, window, cx);
    let shown = said.read(cx).clone();
    let titles = listed.clone();
    section(
        "TaskItem / TaskList",
        "Tasks in rows: a box ticks one done, then its priority, its issue and its title; the due day and who has it sit at the end, and a day past due reads in red. Arrows move between rows; Enter or a double press opens one.",
        cx,
    )
    .child(probe(
        "project-tasks",
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                TaskList::new("project-tasks", listed)
                    .on_toggle(move |key, done, _, cx| {
                        change(&tasks, cx, |tasks| {
                            let task = tasks
                                .iter_mut()
                                .find(|task| task.key == *key)
                                .expect("a listed task");
                            task.status = if done { Status::Done } else { Status::Todo };
                        })
                    })
                    .on_open(move |key, _, cx| {
                        let task = titles
                            .iter()
                            .find(|task| task.key == *key)
                            .expect("a listed task");
                        set(&said, Some(format!("Opened {}.", task.title).into()), cx)
                    }),
            )
            .children(shown.map(|shown| quiet(shown, cx))),
    ))
}

pub fn subtasks(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let items = keep(
        "project-subtasks",
        || {
            vec![
                (SharedString::from("Measure the room"), true),
                (SharedString::from("Sketch two options"), true),
                (SharedString::from("Price the fixtures"), false),
                (SharedString::from("Send to Chloé"), false),
            ]
        },
        window,
        cx,
    );
    let listed = items.read(cx).clone();
    let (toggled, added) = (items.clone(), items);
    section(
        "SubtaskList",
        "A task's steps with how many are done, as a count and a bar. It is the documents Checklist, whose field adds a step on Enter.",
        cx,
    )
    .child(probe(
        "project-subtasks",
        div().w(px(360.)).child(
            Checklist::new("project-subtasks", "Subtasks", listed)
                .on_toggle(move |ix, done, _, cx| change(&toggled, cx, |items| items[ix].1 = done))
                .on_add("Add a subtask", move |words, _, cx| {
                    let words = words.clone();
                    change(&added, cx, |items| items.push((words, false)))
                }),
        ),
    ))
}
