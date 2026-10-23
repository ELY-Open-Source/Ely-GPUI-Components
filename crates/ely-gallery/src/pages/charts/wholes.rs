use ely_gpui_component::charts::{
    CalendarHeatmap, HeatmapChart, PieChart, RadarChart, Series, Slice, Sunburst, Treemap,
};
use gpui::{App, IntoElement, ParentElement, Styled, div, px};
use jiff::civil::date;

use crate::{
    probe::probe,
    ui::{noise, section},
};

pub fn pies(cx: &mut App) -> impl IntoElement + use<> {
    let traffic = [
        ("Search", 412.0),
        ("Direct", 288.0),
        ("Social", 164.0),
        ("Email", 91.0),
    ];
    let pie = traffic
        .iter()
        .fold(PieChart::new("sources"), |pie, (name, value)| {
            pie.slice(*name, *value)
        });
    let donut = [("Design", 38.0), ("Engineering", 54.0), ("Research", 12.0)]
        .iter()
        .fold(PieChart::new("time"), |pie, (name, hours)| {
            pie.slice(*name, *hours)
        })
        .donut("104 h", "This week");
    section(
        "PieChart / DonutChart",
        "Parts of a whole, clockwise from twelve. Hover a slice: it lifts and reads its share; the donut reads it in the middle.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_6()
            .child(probe("pie", div().w(px(360.)).child(pie)))
            .child(probe("donut", div().w(px(360.)).child(donut))),
    )
}

pub fn radar(cx: &mut App) -> impl IntoElement + use<> {
    let axes = ["Speed", "Quality", "Price", "Support", "Docs", "Reach"];
    section(
        "RadarChart",
        "Two products on six measures. Hover a spoke to read both; press a name to hide it.",
        cx,
    )
    .child(probe(
        "radar",
        div().w(px(480.)).child(
            RadarChart::new("compare", axes)
                .series(Series::new("Ely", [86.0, 92.0, 70.0, 81.0, 88.0, 64.0]))
                .series(Series::new("Others", [72.0, 68.0, 84.0, 60.0, 55.0, 79.0])),
        ),
    ))
}

pub fn parts(cx: &mut App) -> impl IntoElement + use<> {
    let spend = [
        ("Payroll", 612.0),
        ("Cloud", 184.0),
        ("Office", 96.0),
        ("Travel", 58.0),
        ("Software", 72.0),
        ("Events", 31.0),
        ("Legal", 22.0),
    ];
    let treemap = spend
        .iter()
        .fold(Treemap::new("spend"), |map, (name, value)| {
            map.tile(*name, *value)
        })
        .format(|value| format!("${value:.0}k"));
    let teams = [
        Slice::new("Product", 0.0).children([
            Slice::new("Design", 14.0),
            Slice::new("Research", 6.0),
            Slice::new("Writing", 4.0),
        ]),
        Slice::new("Engineering", 0.0).children([
            Slice::new("Platform", 0.0)
                .children([Slice::new("Runtime", 9.0), Slice::new("Build", 5.0)]),
            Slice::new("Apps", 18.0),
        ]),
        Slice::new("Go to market", 0.0)
            .children([Slice::new("Sales", 11.0), Slice::new("Support", 8.0)]),
    ];
    section(
        "Treemap / Sunburst",
        "A year's spend as tiles whose areas compare, and a company as rings: teams inside groups inside the whole. Hover to read a part.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_6()
            .child(probe("treemap", div().w(px(420.)).child(treemap)))
            .child(probe("sunburst", div().w(px(300.)).child(Sunburst::new("teams", teams)))),
    )
}

pub fn heatmaps(cx: &mut App) -> impl IntoElement + use<> {
    let hours = ["0", "3", "6", "9", "12", "15", "18", "21"];
    let mut rng = noise(31);
    let busy = |day: usize, hour: usize, rng: &mut dyn FnMut() -> f64| {
        let work = if day < 5 && (3..7).contains(&hour) {
            60.0
        } else {
            12.0
        };
        work + rng() * 30.0
    };
    let week = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .iter()
        .enumerate()
        .fold(HeatmapChart::new("load", hours), |chart, (day, name)| {
            chart.row(
                *name,
                (0..8)
                    .map(|hour| busy(day, hour, &mut rng))
                    .collect::<Vec<_>>(),
            )
        });
    let mut rng = noise(37);
    let commits: Vec<f64> = (0..365)
        .map(|day| {
            if day % 7 >= 5 {
                (rng() * 3.0).floor()
            } else {
                (rng() * 12.0).floor()
            }
        })
        .collect();
    section(
        "HeatmapChart / CalendarHeatmap",
        "Load by weekday and hour, and a year of commits a day, Monday first. Hover a cell to read it.",
        cx,
    )
    .child(probe("load", div().w(px(560.)).child(week)))
    .child(probe("year", div().w(px(720.)).child(CalendarHeatmap::new("commits", date(2026, 1, 1), commits))))
}
