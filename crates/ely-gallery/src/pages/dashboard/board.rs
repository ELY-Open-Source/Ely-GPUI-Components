use ely_gpui_component::{
    charts::{LineChart, Series},
    dashboard::{
        DashboardCard, DashboardFilter, DashboardFilterBar, DashboardGrid, RefreshIntervalSelector,
        Tile, TimeWindow,
    },
    data_display::{Gauge, Statistic, TrendIndicator},
    forms::Choice,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::Timestamp;

use crate::ui::{change, keep, section};

/// The board demo's tiles, window, filters and refresh.
struct Wall {
    tiles: Vec<Tile>,
    window: TimeWindow,
    environment: Option<SharedString>,
    service: Option<SharedString>,
    every: Option<u32>,
    /// Seconds since the last refresh, at the demo's fixed clock.
    since: i64,
}

impl Wall {
    fn new() -> Self {
        Self {
            tiles: vec![
                Tile::new("requests", (0, 0), (4, 2)),
                Tile::new("errors", (4, 0), (4, 2)),
                Tile::new("cpu", (8, 0), (4, 2)),
                Tile::new("latency", (0, 2), (8, 3)),
                Tile::new("saturation", (8, 2), (4, 3)),
            ],
            window: TimeWindow::Hour,
            environment: Some("production".into()),
            service: None,
            every: Some(30),
            since: 40,
        }
    }
}

pub fn grid(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let wall = keep("dashboard-wall", Wall::new, window, cx);
    let now = wall.read(cx);
    let (tiles, time, environment, service, every, since) = (
        now.tiles.clone(),
        now.window,
        now.environment.clone(),
        now.service.clone(),
        now.every,
        now.since,
    );
    let [moved, windowed, filtered, timed, refreshed] = [(); 5].map(|_| wall.clone());
    let clock: Timestamp = "2026-09-27T12:00:00Z".parse().expect("a time");
    let hours: Vec<String> = (0..24).map(|hour| format!("{hour:02}:00")).collect();
    let latency = Series::new(
        "p95",
        (0..24)
            .map(|ix| 120.0 + 40.0 * ((ix as f64) * 0.5).sin() + if ix > 18 { 60.0 } else { 0.0 }),
    );
    let filters = [
        DashboardFilter {
            key: "environment".into(),
            name: "Environment".into(),
            choices: vec![
                Choice::new("production", "Production"),
                Choice::new("staging", "Staging"),
            ],
            picked: environment,
        },
        DashboardFilter {
            key: "service".into(),
            name: "Service".into(),
            choices: vec![
                Choice::new("web", "Web"),
                Choice::new("api", "API"),
                Choice::new("worker", "Worker"),
            ],
            picked: service,
        },
    ];
    let grid = DashboardGrid::new("dashboard-grid", 12, tiles, move |next, _, cx| {
        change(&moved, cx, |wall| wall.tiles = next)
    })
    .card(
        "requests",
        DashboardCard::new("Requests").subtitle("per minute").body(
            Statistic::new("dashboard-requests", "This hour", 12_400.0)
                .trend(TrendIndicator::new(0.082)),
        ),
    )
    .card(
        "errors",
        DashboardCard::new("Errors").subtitle("5xx share").body(
            Statistic::new("dashboard-errors", "This hour", 0.42)
                .decimals(2)
                .suffix("%")
                .trend(TrendIndicator::new(-0.12)),
        ),
    )
    .card(
        "cpu",
        DashboardCard::new("CPU").body(Gauge::new(
            "dashboard-cpu",
            "percent busy",
            62.0,
            0.0,
            100.0,
        )),
    )
    .card(
        "latency",
        DashboardCard::new("Latency")
            .subtitle("p95, milliseconds")
            .body(LineChart::new("dashboard-latency", hours).series(latency)),
    )
    .card(
        "saturation",
        DashboardCard::new("Saturation").body(Gauge::new(
            "dashboard-memory",
            "memory in use",
            81.0,
            0.0,
            100.0,
        )),
    );
    section(
        "Dashboard Grid · Widget / DashboardCard · RefreshIntervalSelector · DashboardFilterBar · StatCard → data_display::Statistic · MetricsChart → charts::LineChart · ResourceGauge → data_display::Gauge",
        "Widgets on a grid of twelve columns: a tile's top strip drags it and its corner resizes it, a cell at a time, while the others make room and rise to fill gaps; a focused tile steps with the arrows. Over it, the window of time and the filters, and how often it refreshes.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .gap_3()
                    .child(
                        DashboardFilterBar::new("dashboard-filters", time, filters, move |next, _, cx| change(&windowed, cx, |wall| wall.window = next), move |key, value, _, cx| {
                                change(&filtered, cx, |wall| match key.as_ref() {
                                    "environment" => wall.environment = value,
                                    _ => wall.service = value,
                                })
                            }),
                    )
                    .child(
                        RefreshIntervalSelector::new("dashboard-refresh", every, clock - jiff::SignedDuration::from_secs(since), clock, move |next, _, cx| change(&timed, cx, |wall| wall.every = next), move |_, cx| change(&refreshed, cx, |wall| wall.since = 0)),
                    ),
            )
            .child(div().w(px(900.)).child(grid)),
    )
}
