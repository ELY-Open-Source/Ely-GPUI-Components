use std::time::Duration;

use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    charts::{
        AreaChart, BarChart, ChartTooltip, LineChart, Points, RealtimeChart, ScatterChart, Series,
    },
    theme::ActiveTheme,
    typography::Caption,
};
use gpui::{App, ClipboardItem, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, live, noise, row, section, set},
};

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn lines(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let turn = keep("lines-turn", || 0usize, window, cx);
    let (now, next) = (*turn.read(cx), turn.clone());
    let copied = keep("lines-copied", || None::<&'static str>, window, cx);
    let (told, tell) = (*copied.read(cx), copied.clone());
    let mut rng = noise(21 + now as u64);
    let revenue: Vec<f64> = (0..12)
        .map(|month| 40.0 + month as f64 * 4.0 + rng() * 12.0)
        .collect();
    let costs: Vec<f64> = (0..12)
        .map(|month| 30.0 + month as f64 * 2.5 + rng() * 8.0)
        .collect();
    let chart = || {
        LineChart::new("revenue", MONTHS)
            .series(Series::new("Revenue", revenue.clone()))
            .series(Series::new("Costs", costs.clone()))
            .smooth()
            .rule(70.0, "Target")
            .note(8, "Launch")
            .zoom()
            .format(|value| {
                if value == 0.0 {
                    "$0".into()
                } else {
                    format!("${value:.0}k")
                }
            })
    };
    let (for_svg, for_csv) = (chart(), chart());
    section(
        "LineChart / ChartAxis / ChartGrid / ChartLegend / ChartTooltip / ChartCrosshair / ChartAnnotation / ChartZoom / ChartExport",
        "Hover for a crosshair and each series' value. Press a name in the legend to hide it. Drag across the chart to zoom into a span; a double press shows the whole year. The target and the launch are annotations. Copy the chart as SVG or its data as CSV.",
        cx,
    )
    .child(probe("revenue", div().w(px(720.)).child(chart())))
    .child(
        row()
            .child(Button::new("lines-next", "New data").variant(ButtonVariant::Ghost).on_click(move |_, _, cx| set(&next, now + 1, cx)))
            .child(Button::new("lines-svg", "Copy SVG").variant(ButtonVariant::Ghost).on_click({
                let tell = tell.clone();
                move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(for_svg.svg(720.0, 240.0, cx)));
                    set(&tell, Some("SVG"), cx)
                }
            }))
            .child(Button::new("lines-csv", "Copy CSV").variant(ButtonVariant::Ghost).on_click(move |_, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(for_csv.csv()));
                set(&tell, Some("CSV"), cx)
            })),
    )
    .children(told.map(|what| Caption::new(format!("Copied the chart as {what}."))))
}

pub fn areas_and_bars(cx: &mut App) -> impl IntoElement + use<> {
    let quarters = ["Q1", "Q2", "Q3", "Q4"];
    let mut rng = noise(5);
    let mut series = |name: &'static str, base: f64| {
        Series::new(
            name,
            (0..4)
                .map(|q| base + q as f64 * 6.0 + rng() * 10.0)
                .collect::<Vec<_>>(),
        )
    };
    let (web, ios, android) = (
        series("Web", 30.0),
        series("iOS", 22.0),
        series("Android", 16.0),
    );
    section(
        "AreaChart / StackedArea / BarChart",
        "Areas stack each platform on the ones before it. Bars stand grouped, stacked, or run sideways.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_6()
            .child(div().w(px(360.)).child(AreaChart::new("areas", quarters).series(web.clone()).series(ios.clone()).series(android.clone()).stacked().smooth()))
            .child(probe("bars", div().w(px(360.)).child(BarChart::new("bars", quarters).series(web.clone()).series(ios.clone()).series(android.clone())))),
    )
    .child(
        div()
            .flex()
            .gap_6()
            .child(div().w(px(360.)).child(BarChart::new("stacked", quarters).series(web.clone()).series(ios.clone()).series(android.clone()).stacked()))
            .child(div().w(px(360.)).child(BarChart::new("sideways", ["Search", "Direct", "Social", "Email"]).series(Series::new("Visits", [412.0, 288.0, 164.0, 91.0])).horizontal())),
    )
}

pub fn scatter(cx: &mut App) -> impl IntoElement + use<> {
    let mut rng = noise(9);
    let cloud = |name: &'static str, cx0: f64, cy0: f64, rng: &mut dyn FnMut() -> f64| {
        Points::new(
            name,
            (0..28)
                .map(|_| (cx0 + (rng() - 0.5) * 40.0, cy0 + (rng() - 0.5) * 30.0))
                .collect::<Vec<_>>(),
        )
    };
    let (a, b) = (
        cloud("Trial", 40.0, 60.0, &mut rng),
        cloud("Paid", 75.0, 35.0, &mut rng),
    );
    let cities = Points::new(
        "Cities",
        [
            (12.0, 70.0),
            (35.0, 52.0),
            (58.0, 81.0),
            (80.0, 44.0),
            (94.0, 66.0),
        ],
    )
    .sizes([3.2, 8.9, 1.4, 12.5, 5.6]);
    section(
        "ScatterChart / BubbleChart",
        "Points placed by two values; sizes turn them into bubbles whose areas compare.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_6()
            .child(
                div().w(px(360.)).child(
                    ScatterChart::new("scatter")
                        .points(a)
                        .points(b)
                        .axes("Sessions", "Retention"),
                ),
            )
            .child(probe(
                "bubbles",
                div().w(px(360.)).child(
                    ScatterChart::new("bubbles")
                        .points(cities)
                        .axes("Price", "Rating"),
                ),
            )),
    )
}

pub fn realtime(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let stream = keep(
        "stream",
        || {
            let mut rng = noise(17);
            (
                (0..40).map(|_| 50.0 + rng() * 20.0).collect::<Vec<f64>>(),
                40u64,
            )
        },
        window,
        cx,
    );
    let ticking = keep("stream-tick", || false, window, cx);
    if !*ticking.read(cx) && !cx.theme().reduced_motion && live("RealtimeChart", cx) {
        set(&ticking, true, cx);
        let stream = stream.clone();
        cx.spawn(async move |cx| {
            let mut rng = noise(29);
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(900))
                    .await;
                cx.update(|cx| {
                    stream.update(cx, |(values, pushed), cx| {
                        let last = values.last().copied().unwrap_or(60.0);
                        values.push((last + (rng() - 0.5) * 10.0).clamp(20.0, 100.0));
                        values.remove(0);
                        *pushed += 1;
                        cx.notify();
                    })
                });
            }
        })
        .detach();
    }
    let (values, pushed) = stream.read(cx).clone();
    let latest = values.last().copied().unwrap_or_default();
    section(
        "RealtimeChart",
        "A new reading every second; the line slides left as each arrives.",
        cx,
    )
    .child(
        div()
            .w(px(720.))
            .child(RealtimeChart::new("stream", values, pushed)),
    )
    .child(div().w(px(200.)).child(ChartTooltip::new("Latest").row(
        Some(cx.theme().colors.chart[0]),
        "Requests per second",
        SharedString::from(format!("{latest:.1}")),
    )))
}
