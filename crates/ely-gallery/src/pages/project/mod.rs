use ely_gpui_component::{
    mail::Label,
    project::{IssueId, Person, Priority, Status, Task},
    theme::{ActiveTheme, TextSize},
};
use gpui::{
    AnyElement, App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div,
};
use jiff::{Timestamp, ToSpan, civil::Date, tz::TimeZone};

mod boards;
mod plans;
mod tasks;
mod time;

use super::Page;
use crate::{step::Step, ui::keep};

pub const PAGE: Page = Page {
    number: 31,
    slug: "project",
    title: "Project",
    summary: "Tasks and issues, who has them, where they stand, and when they are due.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("project-fields", 150.0, 16.0),
    Step::UpAt("project-fields", 150.0, 16.0),
    Step::Wait(250),
    Step::Shot("status-open"),
    Step::Key("escape"),
    Step::DownAt("project-tasks", 26.0, 22.0),
    Step::UpAt("project-tasks", 26.0, 22.0),
    Step::Wait(200),
    Step::Shot("task-ticked"),
    Step::DownAt("project-board", 80.0, 80.0),
    Step::DragTo("project-board", 200.0, 90.0),
    Step::DragTo("project-board", 360.0, 110.0),
    Step::Wait(250),
    Step::Shot("board-drag"),
    Step::UpAt("project-board", 360.0, 110.0),
    Step::Wait(300),
    Step::Shot("board-dropped"),
    Step::DownAt("project-database", 105.0, 17.0),
    Step::UpAt("project-database", 105.0, 17.0),
    Step::Wait(300),
    Step::Shot("database-board"),
    Step::DownAt("project-database", 349.0, 17.0),
    Step::UpAt("project-database", 349.0, 17.0),
    Step::Wait(300),
    Step::Shot("database-timeline"),
];

/// What a demo says it did, under it.
fn quiet(text: SharedString, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .text_size(theme.text_size(TextSize::Sm))
        .text_color(theme.colors.fg_muted)
        .child(text)
}

/// Today in the gallery's zone.
fn today() -> Date {
    let zone = TimeZone::try_system().expect("the gallery reads the system time zone");
    Timestamp::now().to_zoned(zone).date()
}

fn team() -> Vec<Person> {
    vec![
        Person::new("chloe", "Chloé Martin"),
        Person::new("ana", "Ana Lima"),
        Person::new("ben", "Ben Ito"),
        Person::new("dev", "Dev Rao"),
    ]
}

/// The studio's labels.
fn studio_labels() -> Vec<Label> {
    vec![
        Label::new("client", "Client", 0),
        Label::new("site", "Site", 5),
        Label::new("render", "Render", 4),
        Label::new("budget", "Budget", 2),
    ]
}

/// One of the studio's labels, by key.
fn tag(key: &str) -> Label {
    studio_labels()
        .into_iter()
        .find(|label| label.key.as_ref() == key)
        .expect("one of the studio's labels")
}

/// A studio's open work around today: late, due soon, in review, done.
fn work(today: Date) -> Vec<Task> {
    let day = |offset: i64| today.checked_add(offset.days()).expect("a day nearby");
    let who = |key: &str| {
        team()
            .into_iter()
            .find(|person| person.key.as_ref() == key)
            .expect("one of the team")
    };
    vec![
        Task::new("brief", "Write the client brief")
            .issue(IssueId::new("ARC", 41))
            .status(Status::InProgress)
            .priority(Priority::Urgent)
            .starts(day(-9))
            .due(day(-1))
            .assignee(who("ana"))
            .label(tag("client")),
        Task::new("samples", "Order plaster samples")
            .issue(IssueId::new("ARC", 44))
            .priority(Priority::High)
            .starts(day(-3))
            .due(day(0))
            .assignee(who("ben"))
            .label(tag("budget")),
        Task::new("model", "Photograph the site model")
            .issue(IssueId::new("ARC", 38))
            .status(Status::InReview)
            .priority(Priority::Medium)
            .starts(day(-5))
            .due(day(3))
            .assignee(who("chloe"))
            .label(tag("site"))
            .label(tag("render")),
        Task::new("budget", "Revise the lighting budget")
            .issue(IssueId::new("ARC", 47))
            .priority(Priority::Low)
            .due(day(12)),
        Task::new("survey", "Book the land survey")
            .issue(IssueId::new("ARC", 29))
            .status(Status::Done)
            .priority(Priority::Medium)
            .starts(day(-16))
            .due(day(-4))
            .assignee(who("dev"))
            .label(tag("site")),
        Task::new("archive", "Archive last year's renders")
            .issue(IssueId::new("ARC", 17))
            .status(Status::Backlog),
    ]
}

/// The page's tasks, which ticks and picks change.
fn board(window: &mut Window, cx: &mut App) -> Entity<Vec<Task>> {
    keep("project-work", || work(today()), window, cx)
}

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(tasks::marks(cx))
        .child(tasks::fields(window, cx))
        .child(tasks::list(window, cx))
        .child(tasks::subtasks(window, cx))
        .child(boards::kanban(window, cx))
        .child(boards::panel(window, cx))
        .child(boards::labels(window, cx))
        .child(plans::roadmap(cx))
        .child(plans::burndown(cx))
        .child(plans::milestones(cx))
        .child(plans::workload(cx))
        .child(time::tracker(window, cx))
        .child(time::pomodoro(window, cx))
        .child(time::database(window, cx))
        .into_any_element()
}
