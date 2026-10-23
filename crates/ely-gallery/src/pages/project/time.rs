use std::time::Duration;

use ely_gpui_component::project::{DatabaseView, PomodoroTimer, Status, TimeEntry, TimeTracker};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use web_time::Instant;

use super::{board, quiet};
use crate::{
    probe::probe,
    ui::{change, keep, section, set},
};

pub fn tracker(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let entries = keep(
        "project-entries",
        || {
            vec![
                TimeEntry::new("e1", "Client brief", Duration::from_secs(95 * 60)),
                TimeEntry::new("e2", "Site photos", Duration::from_secs(40 * 60)),
                TimeEntry::new("e3", "Budget review", Duration::from_secs(25 * 60)),
            ]
        },
        window,
        cx,
    );
    let running = keep(
        "project-running",
        || None::<(SharedString, Instant)>,
        window,
        cx,
    );
    let (listed, now) = (entries.read(cx).clone(), running.read(cx).clone());
    let (started, stopped, logged) = (running.clone(), running, entries);
    let tracker = TimeTracker::new("project-tracker", listed)
        .on_start(move |task, at, _, cx| set(&started, Some((task.clone(), at)), cx))
        .on_stop(move |at, _, cx| {
            let Some((task, since)) = stopped.update(cx, |running, cx| {
                cx.notify();
                running.take()
            }) else {
                return;
            };
            change(&logged, cx, |entries| {
                let key = SharedString::from(format!("e{}", entries.len() + 1));
                entries.insert(0, TimeEntry::new(key, task, at.duration_since(since)));
            });
        });
    let tracker = match now {
        Some((task, since)) => tracker.running(task, since),
        None => tracker,
    };
    section(
        "TimeTracker",
        "Time against a task: name it, Start or Enter starts the clock, and Stop turns it into an entry. Today's entries sit under it with their total.",
        cx,
    )
    .child(probe("project-tracker", div().w(px(420.)).child(tracker)))
}

pub fn pomodoro(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let said = keep("project-pomodoro-said", || None::<SharedString>, window, cx);
    let shown = said.read(cx).clone();
    section(
        "Pomodoro Timer",
        "Focus for twenty-five minutes, rest for five, and after the fourth round for fifteen. The ring counts the phase down around the time left; a phase that ends waits for Start.",
        cx,
    )
    .child(probe(
        "project-pomodoro",
        div()
            .w(px(280.))
            .flex()
            .flex_col()
            .items_center()
            .gap_3()
            .child(
                PomodoroTimer::new("project-pomodoro")
                    .on_phase(move |phase, _, cx| set(&said, Some(format!("{} begins.", phase.words()).into()), cx)),
            )
            .children(shown.map(|shown| quiet(shown, cx))),
    ))
}

pub fn database(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tasks = board(window, cx);
    let listed = tasks.read(cx).clone();
    let opened = keep(
        "project-opened",
        || Some(SharedString::from("brief")),
        window,
        cx,
    );
    let moved = tasks.clone();
    let view = DatabaseView::new("project-database", listed)
        .on_open(move |key, _, cx| set(&opened, Some(key.clone()), cx))
        .on_status(move |key, status: Status, _, cx| {
            change(&moved, cx, |tasks| {
                let task = tasks
                    .iter_mut()
                    .find(|task| task.key == *key)
                    .expect("a listed task");
                task.status = status;
            })
        });
    section(
        "DatabaseView",
        "The studio's tasks five ways: a table, a board by status, a gallery of cards, a month calendar by due day, and a timeline from start to due. A card moved on the board takes its column's status, and a record opens in the panel above.",
        cx,
    )
    .child(probe("project-database", div().w(px(860.)).child(view)))
}
