use ely_gpui_component::{
    charts::BurndownChart,
    project::{Initiative, Load, MilestoneProgress, Roadmap, WorkloadView},
};
use gpui::{App, IntoElement, ParentElement, Styled, div, px};
use jiff::{ToSpan, civil::Date};

use super::{team, today};
use crate::{probe::probe, ui::section};

/// A day `offset` days from `today`.
fn day(today: Date, offset: i64) -> Date {
    today.checked_add(offset.days()).expect("a day nearby")
}

pub fn roadmap(cx: &mut App) -> impl IntoElement + use<> {
    let today = today();
    let at = |offset: i64| day(today, offset);
    let map = Roadmap::new("project-roadmap", at(-40), 6)
        .today(today)
        .lane(
            "Design",
            [
                Initiative::new("concept", "Concept", at(-40), at(-12)).progress(1.0),
                Initiative::new("schematic", "Schematic design", at(-14), at(30)).progress(0.55),
                Initiative::new("details", "Detail drawings", at(28), at(95)),
            ],
        )
        .lane(
            "Site",
            [
                Initiative::new("survey", "Land survey", at(-20), at(6)).progress(0.8),
                Initiative::new("permit", "Permit set", at(10), at(70)).progress(0.1),
            ],
        )
        .lane(
            "Client",
            [Initiative::new("reviews", "Monthly reviews", at(-35), at(120)).progress(0.3)],
        );
    section(
        "GanttChart / TimelineView · Roadmap",
        "A roadmap lays initiatives across months in lanes, each bar filled as far as it has come, under a line at today; hover a bar for its dates. GanttChart and TimelineView are the charts GanttChart, for tasks by the day.",
        cx,
    )
    .child(probe("project-roadmap", div().w(px(720.)).child(map)))
}

pub fn burndown(cx: &mut App) -> impl IntoElement + use<> {
    let today = today();
    let (first, last) = (day(today, -6), day(today, 7));
    let chart = BurndownChart::new("project-burndown", first, last, 40.0)
        .left([40.0, 38.0, 38.0, 33.0, 34.0, 29.0, 24.0])
        .today(today);
    section(
        "Burndown Chart",
        "Work left over a sprint: the ideal line from the scope down to nothing, what was left at each day's end so far, and a line at today. Work added mid-sprint lifts the line. Hover a day to read it. It lives with the charts as BurndownChart.",
        cx,
    )
    .child(probe("project-burndown", div().w(px(640.)).child(chart)))
}

pub fn milestones(cx: &mut App) -> impl IntoElement + use<> {
    let today = today();
    let at = |offset: i64| day(today, offset);
    let rows = [
        MilestoneProgress::new("project-concept", "Concept", (at(-40), at(-12)), (14, 14)),
        MilestoneProgress::new(
            "project-schematic",
            "Schematic design",
            (at(-14), at(30)),
            (9, 20),
        ),
        MilestoneProgress::new("project-permit", "Permit set", (at(-30), at(10)), (4, 16)),
        MilestoneProgress::new("project-survey", "Land survey", (at(-20), at(-2)), (7, 9)),
    ];
    section(
        "MilestoneProgress",
        "A milestone's issues done as a bar and a count, when it is due, and whether it keeps pace: on track, at risk when the work trails the time gone, late past its day, or done.",
        cx,
    )
    .child(probe(
        "project-milestones",
        div()
            .w(px(420.))
            .flex()
            .flex_col()
            .gap_6()
            .children(rows.map(|row| row.today(today))),
    ))
}

pub fn workload(cx: &mut App) -> impl IntoElement + use<> {
    let people = team();
    let loads = [(12.0, 10.0), (7.0, 10.0), (9.0, 8.0), (4.0, 10.0)];
    let rows = people
        .into_iter()
        .zip(loads)
        .map(|(person, (assigned, capacity))| Load::new(person, assigned, capacity));
    section(
        "WorkloadView",
        "Who carries how much this sprint: each person's points against what they can take, the bar turning red past it.",
        cx,
    )
    .child(probe(
        "project-workload",
        div().w(px(420.)).child(WorkloadView::new("project-workload", rows, "pts")),
    ))
}
