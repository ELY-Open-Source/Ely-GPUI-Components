use ely_gpui_component::charts::{
    Bullet, BulletChart, ChordDiagram, FunnelChart, GanttChart, NetworkGraph, ParallelCoordinates,
    ProgressChart, SankeyChart, Task,
};
use gpui::{App, IntoElement, ParentElement, Styled, div, px};
use jiff::civil::date;

use crate::{
    probe::probe,
    ui::{noise, section},
};

pub fn flows(cx: &mut App) -> impl IntoElement + use<> {
    let budget = SankeyChart::new(
        "budget",
        [
            "Salary",
            "Freelance",
            "Income",
            "Home",
            "Food",
            "Savings",
            "Travel",
        ],
    )
    .link(0, 2, 5200.0)
    .link(1, 2, 1400.0)
    .link(2, 3, 2100.0)
    .link(2, 4, 900.0)
    .link(2, 5, 2600.0)
    .link(2, 6, 1000.0)
    .format(|value| format!("${value:.0}"));
    let signups = [
        ("Visited", 48200.0),
        ("Signed up", 9640.0),
        ("Verified", 7420.0),
        ("Invited a teammate", 3110.0),
        ("Paid", 1260.0),
    ]
    .iter()
    .fold(FunnelChart::new("signups"), |funnel, (name, value)| {
        funnel.stage(*name, *value)
    });
    section(
        "SankeyChart / FunnelChart",
        "A month's money from where it came to where it went, and how many of a month's visitors reached each step. Hover a node, a ribbon or a stage.",
        cx,
    )
    .child(probe("sankey", div().w(px(720.)).child(budget)))
    .child(probe("funnel", div().w(px(720.)).child(signups)))
}

pub fn schedule(cx: &mut App) -> impl IntoElement + use<> {
    let plan = GanttChart::new("launch")
        .task(Task::new("Research", date(2026, 9, 1), date(2026, 9, 11)).done(1.0))
        .task(
            Task::new("Design", date(2026, 9, 9), date(2026, 9, 25))
                .done(0.8)
                .after(0),
        )
        .task(
            Task::new("Build", date(2026, 9, 21), date(2026, 10, 16))
                .done(0.35)
                .after(1),
        )
        .task(
            Task::new("Write the docs", date(2026, 10, 5), date(2026, 10, 20))
                .done(0.1)
                .after(1),
        )
        .task(
            Task::new("Beta", date(2026, 10, 19), date(2026, 10, 30))
                .after(2)
                .after(3),
        )
        .task(Task::new("Launch", date(2026, 11, 2), date(2026, 11, 3)).after(4))
        .today(date(2026, 9, 25));
    section(
        "GanttChart",
        "A launch plan: each bar filled as far as it is done, arrows from the work it waits for, and a line at today. Hover a row.",
        cx,
    )
    .child(probe("gantt", div().w(px(720.)).child(plan)))
}

pub fn relations(cx: &mut App) -> impl IntoElement + use<> {
    let people = [
        "Ada", "Grace", "Alan", "Edsger", "Barbara", "Donald", "Ken", "Dennis", "Bjarne", "Guido",
        "Linus", "Margaret",
    ];
    let graph = people
        .iter()
        .enumerate()
        .fold(NetworkGraph::new("people"), |graph, (ix, name)| {
            graph.node(*name, ix % 3)
        });
    let graph = [
        (0, 1),
        (0, 2),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 1),
        (5, 3),
        (6, 7),
        (7, 8),
        (6, 8),
        (8, 9),
        (9, 10),
        (10, 6),
        (11, 0),
        (11, 5),
        (5, 6),
    ]
    .into_iter()
    .fold(graph, |graph, (a, b)| graph.edge(a, b));
    let regions = ["Europe", "Americas", "Asia", "Africa", "Oceania"];
    let trade = vec![
        vec![0.0, 42.0, 31.0, 12.0, 5.0],
        vec![38.0, 0.0, 44.0, 6.0, 7.0],
        vec![35.0, 50.0, 0.0, 14.0, 18.0],
        vec![15.0, 5.0, 11.0, 0.0, 2.0],
        vec![4.0, 6.0, 20.0, 1.0, 0.0],
    ];
    let mut rng = noise(43);
    let axes = ["Price", "Weight", "Range", "Charge", "Seats"];
    let cars = ["Aria", "Bolt", "Crest", "Dune", "Echo", "Flux"]
        .iter()
        .fold(ParallelCoordinates::new("cars", axes), |chart, name| {
            chart.record(
                *name,
                [
                    28.0 + rng() * 40.0,
                    1.4 + rng() * 1.2,
                    250.0 + rng() * 350.0,
                    20.0 + rng() * 40.0,
                    (4.0 + rng() * 3.0).floor(),
                ],
            )
        });
    section(
        "NetworkGraph / ForceGraph / ChordDiagram / ParallelCoordinates",
        "People and who they work with, settling as forces pull and push: drag a node, hover one to light its ties. Trade between regions around a circle, and cars on five measures at once.",
        cx,
    )
    .child(probe("network", div().w(px(720.)).child(graph)))
    .child(
        div()
            .flex()
            .gap_6()
            .child(probe("chord", div().w(px(360.)).child(ChordDiagram::new("trade", regions, trade))))
            .child(probe("parallel", div().w(px(420.)).child(cars))),
    )
}

pub fn targets(cx: &mut App) -> impl IntoElement + use<> {
    let bullets = BulletChart::new("quarter")
        .bullet(Bullet::new("Revenue", 268.0, 250.0).bands([150.0, 225.0, 300.0]))
        .bullet(Bullet::new("Profit", 22.0, 27.0).bands([20.0, 25.0, 30.0]))
        .bullet(Bullet::new("New customers", 1680.0, 2000.0).bands([1000.0, 1500.0, 2500.0]));
    let rings = ProgressChart::new("goals")
        .goal("Move", 480.0, 600.0)
        .goal("Exercise", 27.0, 30.0)
        .goal("Stand", 9.0, 12.0);
    section(
        "Bullet Chart / ProgressChart",
        "A quarter's measures against their targets over bands from poor to good, and a day's goals as rings that fill toward them.",
        cx,
    )
    .child(probe("bullets", div().w(px(720.)).child(bullets)))
    .child(probe("rings", rings))
}
