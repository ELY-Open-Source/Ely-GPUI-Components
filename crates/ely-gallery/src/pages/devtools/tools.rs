use ely_gpui_component::devtools::{
    ColorContrastChecker, Container, ContainerList, ContainerState, Encoder, Pod, PodList,
    PodPhase, Reading, RegexTester, ResourceMonitor,
};
use gpui::{App, Hsla, IntoElement, ParentElement, Styled, Window, div, px, rgb};
use jiff::Timestamp;

use crate::ui::{change, keep, section};

pub fn text(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let colors = keep(
        "devtools-contrast",
        || (Hsla::from(rgb(0x6b6b6b)), Hsla::from(rgb(0xf4efe6))),
        window,
        cx,
    );
    let (fore, back) = *colors.read(cx);
    section(
        "RegexTester · Base64 / Encoder · ColorContrastChecker",
        "A pattern tried against a sample with each match washed and its groups listed; text encoded or decoded as Base64, for a URL or as hex; and text on a background with the contrast between them and the WCAG levels it meets.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(
                div().w(px(420.)).child(RegexTester::new(
                    "devtools-regex",
                    r"(?P<user>[\w.]+)@(?P<host>[\w.]+)",
                    "Write to ada@example.com or grace.h@navy.mil.\nNot a@ or @b.",
                )),
            )
            .child(div().w(px(380.)).child(Encoder::new("devtools-encoder", "Hello, 世界")))
            .child(
                div().w(px(360.)).child(
                    ColorContrastChecker::new("devtools-contrast", fore, back, move |fore, back, _, cx| change(&colors, cx, |colors| *colors = (fore, back))),
                ),
            ),
    )
}

fn containers() -> Vec<Container> {
    let container = |key: &str, image: &str, state, ports: &[&str], cpu, memory| Container {
        key: key.to_string().into(),
        name: key.to_string().into(),
        image: image.to_string().into(),
        state,
        ports: ports.iter().map(|port| port.to_string().into()).collect(),
        cpu,
        memory,
    };
    vec![
        container(
            "web",
            "registry.example.com/web:1.8.2",
            ContainerState::Running,
            &["8080:80"],
            0.18,
            184 << 20,
        ),
        container(
            "worker",
            "registry.example.com/worker:1.8.2",
            ContainerState::Running,
            &[],
            0.42,
            312 << 20,
        ),
        container(
            "postgres",
            "postgres:17",
            ContainerState::Paused,
            &["5432:5432"],
            0.0,
            96 << 20,
        ),
        container(
            "migrate",
            "registry.example.com/web:1.8.2",
            ContainerState::Exited(1),
            &[],
            0.0,
            0,
        ),
    ]
}

fn pods(now: Timestamp) -> Vec<Pod> {
    let ago = |minutes: i64| now - jiff::SignedDuration::from_mins(minutes);
    let pod = |key: &str, namespace: &str, phase, ready, restarts, minutes, node: &str| Pod {
        key: key.to_string().into(),
        name: key.to_string().into(),
        namespace: namespace.to_string().into(),
        phase,
        ready,
        restarts,
        started: ago(minutes),
        node: node.to_string().into(),
    };
    vec![
        pod(
            "web-7c9f-2kq4x",
            "shop",
            PodPhase::Running,
            (1, 1),
            0,
            3 * 1440,
            "node-a",
        ),
        pod(
            "web-7c9f-8zt6m",
            "shop",
            PodPhase::Running,
            (1, 1),
            2,
            3 * 1440,
            "node-b",
        ),
        pod(
            "worker-5d8b-q2n9",
            "shop",
            PodPhase::Pending,
            (0, 1),
            0,
            2,
            "node-b",
        ),
        pod(
            "migrate-29xk",
            "shop",
            PodPhase::Failed,
            (0, 1),
            4,
            45,
            "node-a",
        ),
        pod(
            "coredns-6f4b-hx8c",
            "kube-system",
            PodPhase::Running,
            (1, 1),
            0,
            21 * 1440,
            "node-a",
        ),
    ]
}

pub fn machines(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let now: Timestamp = "2026-09-27T12:00:00Z".parse().expect("a time");
    let held = keep("devtools-containers", containers, window, cx);
    let (toggled, restarted) = (held.clone(), held.clone());
    let list = held.read(cx).clone();
    let wave = |base: f32, swing: f32| {
        (0..24).map(move |ix| base + swing * (((ix as f32) * 0.6).sin() + ix as f32 / 24.0))
    };
    section(
        "ContainerList · PodList · ResourceMonitor",
        "Containers with where each stands and its load, started, stopped or restarted from its row; pods in a table narrowed by namespace; and a machine's readings, each with its latest value, its recent past and how full it is.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div().w(px(640.)).child(
                    ContainerList::new("devtools-containers", list)
                        .on_toggle(move |key, _, cx| {
                            change(&toggled, cx, |list| {
                                let container = list.iter_mut().find(|each| each.key == *key).expect("a listed container");
                                container.state = match container.state {
                                    ContainerState::Running => ContainerState::Exited(0),
                                    _ => ContainerState::Running,
                                };
                            })
                        })
                        .on_restart(move |key, _, cx| {
                            change(&restarted, cx, |list| {
                                list.iter_mut().find(|each| each.key == *key).expect("a listed container").state = ContainerState::Restarting
                            })
                        }),
                ),
            )
            .child(div().w(px(860.)).child(PodList::new("devtools-pods", pods(now), now)))
            .child(
                div().w(px(860.)).child(ResourceMonitor::new(
                    "devtools-monitor",
                    [
                        Reading::new("CPU", "cores", wave(3.1, 1.0)).limit(8.0),
                        Reading::new("Memory", "GB", wave(5.2, 0.6)).limit(16.0),
                        Reading::new("Network in", "MB/s", wave(12.0, 6.0)),
                        Reading::new("Disk", "GB", wave(310.0, 2.0)).limit(512.0),
                    ],
                )),
            ),
    )
}
