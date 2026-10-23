use std::rc::Rc;

use ely_gpui_component::{
    debug::{Flamegraph, NetworkInspector, ProfileFrame, Request, Span, TimelineProfiler},
    forms::{Input, TextInput},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::ui::{keep, section, set};

fn frame(name: &str, own: u64, calls: Vec<ProfileFrame>) -> ProfileFrame {
    ProfileFrame {
        name: name.to_string().into(),
        own,
        calls,
    }
}

/// A frame's worth of an app's time, sampled.
fn profile() -> ProfileFrame {
    frame(
        "main",
        4,
        vec![
            frame(
                "run_loop",
                6,
                vec![
                    frame(
                        "draw",
                        10,
                        vec![
                            frame(
                                "layout",
                                38,
                                vec![
                                    frame("measure_text", 22, vec![]),
                                    frame("taffy::compute", 30, vec![]),
                                ],
                            ),
                            frame(
                                "paint",
                                26,
                                vec![
                                    frame("shape_line", 18, vec![]),
                                    frame("rasterize", 12, vec![]),
                                ],
                            ),
                        ],
                    ),
                    frame("dispatch_event", 14, vec![frame("on_click", 9, vec![])]),
                ],
            ),
            frame("io_thread", 20, vec![frame("read", 16, vec![])]),
        ],
    )
}

fn spans() -> Rc<Vec<Span>> {
    let span = |track, name: &str, start, end| Span {
        track,
        name: name.to_string().into(),
        start,
        end,
    };
    Rc::new(vec![
        span(0, "event", 0.0, 1.2),
        span(0, "layout", 1.2, 5.6),
        span(0, "paint", 5.6, 9.1),
        span(0, "event", 16.6, 17.3),
        span(0, "layout", 17.3, 20.4),
        span(0, "paint", 20.4, 23.0),
        span(1, "shape", 2.0, 4.8),
        span(1, "shape", 18.0, 19.6),
        span(2, "gc", 9.5, 12.0),
        span(3, "read", 0.5, 14.0),
        span(3, "parse", 14.0, 15.2),
    ])
}

pub fn profiles(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let focus = keep("debug-focus", Vec::<usize>::new, window, cx);
    let hovered = keep("debug-hovered", || None::<Vec<usize>>, window, cx);
    let range = keep("debug-range", || (0.0_f64, 25.0_f64), window, cx);
    let picked = keep("debug-span", || Some(1usize), window, cx);
    let (now_focus, now_hovered, now_range, now_picked) = (
        focus.read(cx).clone(),
        hovered.read(cx).clone(),
        *range.read(cx),
        *picked.read(cx),
    );
    let (zoom, hover, pan, pick) = (
        focus.clone(),
        hovered.clone(),
        range.clone(),
        picked.clone(),
    );
    let timeline = TimelineProfiler::new(
        "debug-timeline",
        ["Main", "Text", "Memory", "IO"],
        spans(),
        now_range,
    )
    .on_range(move |next, _, cx| set(&pan, next, cx))
    .on_select(move |ix, _, cx| set(&pick, Some(ix), cx));
    let timeline = match now_picked {
        Some(ix) => timeline.selected(ix),
        None => timeline,
    };
    section(
        "PerformanceProfiler / Flamegraph / TimelineProfiler",
        "Where the time went: each function as wide as its samples with its callees under it; press a frame to zoom in and a caller to zoom back out. Work over time on tracks; the wheel zooms around the pointer and a sideways wheel pans.",
        cx,
    )
    .child(
        div().w(px(840.)).child(
            Flamegraph::new("debug-flame", profile())
                .focus(now_focus)
                .hovered(now_hovered)
                .on_focus(move |path, _, cx| set(&zoom, path, cx))
                .on_hover(move |path, _, cx| {
                    if *hover.read(cx) != path {
                        set(&hover, path, cx)
                    }
                }),
        ),
    )
    .child(div().w(px(840.)).child(timeline))
}

fn requests() -> Vec<Request> {
    let request = |method: &'static str,
                   url: &'static str,
                   status,
                   kind: &'static str,
                   size,
                   start,
                   duration,
                   body: Option<&'static str>| Request {
        method: method.into(),
        url: url.into(),
        status,
        kind: kind.into(),
        size,
        start,
        duration,
        headers: vec![
            (
                "content-type".into(),
                if kind == "fetch" {
                    "application/json".into()
                } else {
                    "text/html".into()
                },
            ),
            ("cache-control".into(), "no-store".into()),
        ],
        body: body.map(Into::into),
    };
    vec![
        request(
            "GET",
            "https://ely.dev/",
            Some(200),
            "document",
            18_400,
            0.0,
            120.0,
            None,
        ),
        request(
            "GET",
            "https://ely.dev/static/app.js",
            Some(200),
            "script",
            212_000,
            130.0,
            240.0,
            None,
        ),
        request(
            "GET",
            "https://ely.dev/static/inter.woff2",
            Some(304),
            "font",
            0,
            140.0,
            60.0,
            None,
        ),
        request(
            "GET",
            "https://api.ely.dev/v1/quotes?symbol=ELY",
            Some(200),
            "fetch",
            2_300,
            380.0,
            95.0,
            Some("{ \"symbol\": \"ELY\", \"price\": 128.4, \"change\": 0.012 }"),
        ),
        request(
            "POST",
            "https://api.ely.dev/v1/orders",
            Some(503),
            "fetch",
            180,
            420.0,
            310.0,
            Some("{ \"error\": \"orders are paused\" }"),
        ),
        request(
            "GET",
            "https://api.ely.dev/v1/stream",
            None,
            "fetch",
            0,
            500.0,
            400.0,
            None,
        ),
    ]
}

pub fn network(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let picked = keep("debug-request", || Some(4usize), window, cx);
    let query = window.use_keyed_state("debug-network-query", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Filter addresses")
    });
    let now = *picked.read(cx);
    let text: SharedString = query.read(cx).text().to_string().into();
    let pick = picked.clone();
    let inspector = NetworkInspector::new("debug-network", requests())
        .query(text)
        .on_select(move |ix, _, cx| set(&pick, Some(ix), cx));
    let inspector = match now {
        Some(ix) => inspector.selected(ix),
        None => inspector,
    };
    section(
        "NetworkInspector",
        "Requests in the order they started, with status, kind, size and time, and a waterfall of when each ran; a filter keeps matching addresses. Press one to read its headers and body.",
        cx,
    )
    .child(div().w(px(320.)).child(Input::new(&query)))
    .child(div().w(px(840.)).child(inspector))
}
