use ely_gpui_component::{
    dashboard::{
        AlertList, AlertState, Check, EventStream, Health, HealthCheck, Incident, IncidentCard,
        IncidentPhase, IncidentUpdate, MonitorAlert, Service, ServiceStatus, StreamEvent,
        UptimeBar,
    },
    data_display::Tone,
    primitives::Severity,
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};
use jiff::{SignedDuration, Timestamp, civil::date};

use crate::ui::{change, keep, section};

fn now() -> Timestamp {
    "2026-09-27T12:00:00Z".parse().expect("a time")
}

fn ago(minutes: i64) -> Timestamp {
    now() - SignedDuration::from_mins(minutes)
}

pub fn health(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let days = |dip: usize| {
        (0..60).map(move |ix| {
            if ix == dip {
                Some(0.982)
            } else if ix == dip + 3 {
                Some(0.9975)
            } else if ix < 4 {
                None
            } else {
                Some(1.0)
            }
        })
    };
    section(
        "UptimeBar · ServiceStatus · HealthCheck Indicator",
        "Sixty days of a service, a bar a day in the tone of its uptime; services under a line for the whole system; and checks with what each found, when it ran and how long it took.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(
                div()
                    .w(px(420.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(UptimeBar::new("dashboard-uptime-api", "API", date(2026, 7, 30), days(41)))
                    .child(UptimeBar::new("dashboard-uptime-web", "Website", date(2026, 7, 30), days(12))),
            )
            .child(
                div().w(px(360.)).child(ServiceStatus::new([
                    Service { name: "API".into(), health: Health::Operational, uptime: Some(0.9998) },
                    Service { name: "Website".into(), health: Health::Operational, uptime: Some(0.9992) },
                    Service { name: "Payments".into(), health: Health::Degraded, uptime: Some(0.9971) },
                    Service { name: "Email".into(), health: Health::Maintenance, uptime: None },
                ])),
            )
            .child(
                div()
                    .w(px(320.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(HealthCheck::new("GET /health", Check::Passing).ran(ago(1), now()).took(42))
                    .child(HealthCheck::new("Database", Check::Passing).ran(ago(1), now()).took(8))
                    .child(HealthCheck::new("Queue depth", Check::Warning).ran(ago(2), now()))
                    .child(HealthCheck::new("Payments provider", Check::Failing).ran(ago(3), now()).took(3_020))
                    .child(HealthCheck::new("Search index", Check::Pending)),
            ),
    )
}

pub fn incidents(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let alerts = keep(
        "dashboard-alerts",
        || {
            let alert =
                |key: &str, title: &str, source: &str, severity, state, minutes| MonitorAlert {
                    key: key.to_string().into(),
                    title: title.to_string().into(),
                    source: source.to_string().into(),
                    severity,
                    state,
                    since: ago(minutes),
                };
            vec![
                alert(
                    "a1",
                    "Payments p95 over 2s",
                    "payments · prometheus",
                    Severity::Danger,
                    AlertState::Firing,
                    12,
                ),
                alert(
                    "a2",
                    "Queue depth rising",
                    "worker · prometheus",
                    Severity::Warning,
                    AlertState::Firing,
                    25,
                ),
                alert(
                    "a3",
                    "Disk 85% on db-2",
                    "db-2 · node exporter",
                    Severity::Warning,
                    AlertState::Acknowledged,
                    95,
                ),
                alert(
                    "a4",
                    "Certificate renewed",
                    "edge · cert-manager",
                    Severity::Info,
                    AlertState::Resolved,
                    300,
                ),
            ]
        },
        window,
        cx,
    );
    let list = alerts.read(cx).clone();
    let incident = Incident {
        title: "Card payments fail for some customers".into(),
        severity: Severity::Danger,
        services: vec!["Payments".into(), "Checkout".into()],
        started: ago(48),
        updates: vec![
            IncidentUpdate {
                phase: IncidentPhase::Investigating,
                text: "Payments time out for about one in ten cards.".into(),
                at: ago(46),
            },
            IncidentUpdate {
                phase: IncidentPhase::Identified,
                text: "The provider's EU region is slow; traffic moves to the US region.".into(),
                at: ago(30),
            },
            IncidentUpdate {
                phase: IncidentPhase::Monitoring,
                text: "Payments pass again; we watch error rates.".into(),
                at: ago(9),
            },
        ],
    };
    section(
        "AlertList · IncidentCard",
        "Alerts with their severity, what raised them and since when; a firing one is acknowledged from its row. An incident with its severity and phase, what it touches, and its updates newest first.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(
                div().w(px(460.)).child(
                    AlertList::new("dashboard-alert-list", list, now()).on_acknowledge(move |key, _, cx| {
                        change(&alerts, cx, |alerts| {
                            alerts.iter_mut().find(|alert| alert.key == *key).expect("a listed alert").state = AlertState::Acknowledged
                        })
                    }),
                ),
            )
            .child(div().w(px(420.)).child(IncidentCard::new(incident, now()))),
    )
}

pub fn stream(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let event = |key: &str, kind: &str, text: &str, tone, minutes| StreamEvent {
        key: key.to_string().into(),
        kind: kind.to_string().into(),
        text: text.to_string().into(),
        tone,
        at: ago(minutes),
    };
    let events = vec![
        event(
            "e1",
            "deploy",
            "web 1.8.1 rolled out to production",
            Tone::Info,
            58,
        ),
        event(
            "e2",
            "scale",
            "worker scaled from 4 to 6 replicas",
            Tone::Neutral,
            41,
        ),
        event("e3", "alert", "Payments p95 over 2s", Tone::Danger, 12),
        event(
            "e4",
            "config",
            "feature flag new-checkout at 20%",
            Tone::Accent,
            7,
        ),
        event(
            "e5",
            "deploy",
            "web 1.8.2 rolled out to production",
            Tone::Info,
            2,
        ),
    ];
    section(
        "EventStream · LogStream → terminal::LogViewer · TraceWaterfall → debug::TimelineProfiler · SystemMonitor → devtools::ResourceMonitor · ProcessTable → terminal::ProcessList · NetworkGraph (throughput) → charts::RealtimeChart · MapView / GeoHeatmap → maps::MapView, maps::GeoHeatmap · WorldMap → maps::WorldMap",
        "Events as they come, newest first, kept by kind, with Pause to hold the list while new ones count up. Logs, traces, a machine's readings, its processes and live throughput have their homes in the terminal, agent, dev tools and charts chapters; the maps live in the maps chapter.",
        cx,
    )
    .child(div().w(px(620.)).child(EventStream::new("dashboard-events", events, now())))
}
