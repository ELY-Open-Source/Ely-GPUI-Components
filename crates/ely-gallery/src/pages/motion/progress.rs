use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::anyhow;
use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    feedback::AsyncView,
    forms::{Upload, UploadList, UploadState},
    motion::{
        LazyLoad, LoadingOverlay, ProgressBar, ProgressRing, Refresh, RefreshIndicator,
        TypingIndicator,
    },
    theme::{ActiveTheme, Radius, TextSize},
    typography::Caption,
};
use gpui::{
    App, Entity, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use super::loading::later;
use crate::{
    probe::probe,
    ui::{keep, live, row, section, set, specimen, specimens},
};

const STEPS: [f32; 5] = [0.0, 0.35, 0.72, 1.0, 0.5];

pub fn bars(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let at = keep("progress-at", || 1usize, window, cx);
    let (now, step) = (*at.read(cx), at.clone());
    let value = STEPS[now];
    section(
        "ProgressBar / ProgressCircle / ProgressRing",
        "A track that fills and glides to each new value; a sweep when none is known; a buffer ahead; steps as segments; the same in a ring.",
        cx,
    )
    .child(
        div()
            .w_128()
            .flex()
            .flex_col()
            .gap_5()
            .child(specimen("determinate, gliding", ProgressBar::new("bar-value", value), cx))
            .child(specimen("buffered", ProgressBar::new("bar-buffer", 0.4).buffer(0.7), cx))
            .child(specimen("segmented, four steps", ProgressBar::new("bar-steps", 0.62).segments(4), cx))
            .child(specimen("indeterminate", ProgressBar::indeterminate("bar-sweep"), cx)),
    )
    .child(
        specimens()
            .child(specimen("gliding", ProgressRing::new("ring-value", value).percent(), cx))
            .child(specimen("a quarter", ProgressRing::new("ring-quarter", 0.25), cx))
            .child(specimen("done", ProgressRing::new("ring-done", 1.0).percent(), cx)),
    )
    .child(row().child(probe(
        "progress-step",
        Button::new("progress-step", "Step")
            .variant(ButtonVariant::Ghost)
            .on_click(move |_, _, cx| set(&step, (now + 1) % STEPS.len(), cx)),
    )))
}

pub fn overlay(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let busy = keep("overlay-busy", || false, window, cx);
    let (loading, start) = (*busy.read(cx), busy.clone());
    let theme = cx.theme();
    section(
        "LoadingOverlay",
        "A veil and a spinner over what is reloading. Presses stop at the veil; it fades in and out.",
        cx,
    )
    .child(
        LoadingOverlay::new("overlay", loading).w_96().label("Refreshing prices…").child(
            div()
                .w_96()
                .rounded(theme.radius(Radius::Lg))
                .border_1()
                .border_color(theme.colors.border)
                .children(["Oat milk · 2.40", "Espresso beans · 14.90", "Rye bread · 4.20"].map(
                    |line| div().px_4().py_3().border_b_1().border_color(theme.colors.border).child(line),
                )),
        ),
    )
    .child(row().child(probe(
        "overlay-reload",
        Button::new("overlay-reload", "Reload")
            .variant(ButtonVariant::Ghost)
            .on_click(move |_, _, cx| {
                set(&start, true, cx);
                later(&start, false, Duration::from_millis(1400), cx);
            }),
    )))
}

pub fn lazy(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let rows = (0..12).map(|ix| {
        LazyLoad::new(
            SharedString::from(format!("lazy-{ix}")),
            px(36.0),
            move |_, _| {
                div()
                    .h(px(36.0))
                    .px_3()
                    .flex()
                    .items_center()
                    .child(format!("Row {} of 12", ix + 1))
            },
        )
    });
    section(
        "LazyLoad",
        "Rows build only once they scroll into view; a skeleton holds each place until then. Scroll the list.",
        cx,
    )
    .child(
        div()
            .id("lazy-scroll")
            .w_96()
            .h_48()
            .overflow_y_scroll()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .child(div().flex().flex_col().gap_1().p_1().children(rows)),
    )
}

/// A report that takes a moment to fetch, and can be told to fail.
struct Report {
    fail: Arc<AtomicBool>,
}

pub fn suspense(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let section = section(
        "Suspense / AsyncView",
        "A spinner while a value loads off the main thread, then the value as it fades in, or the error with Try again.",
        cx,
    );
    if !live("Suspense / AsyncView", cx) {
        return section;
    }
    let (view, breaker) = report(window, cx);
    let theme = cx.theme();
    section
        .child(
            div()
                .w_128()
                .min_h_40()
                .flex()
                .items_center()
                .justify_center()
                .px_4()
                .rounded(theme.radius(Radius::Lg))
                .border_1()
                .border_color(theme.colors.border)
                .child(view.clone()),
        )
        .child(
            row()
                .child(
                    Button::new("report-reload", "Reload")
                        .variant(ButtonVariant::Ghost)
                        .on_click({
                            let view = view.clone();
                            move |_, _, cx| view.update(cx, |view, cx| view.reload(cx))
                        }),
                )
                .child(
                    Button::new("report-fail", "Reload and fail")
                        .variant(ButtonVariant::Ghost)
                        .on_click({
                            let view = view.clone();
                            move |_, _, cx| {
                                breaker.store(true, Ordering::Relaxed);
                                view.update(cx, |view, cx| view.reload(cx))
                            }
                        }),
                ),
        )
}

/// The report's view and its fail switch.
fn report(window: &mut Window, cx: &mut App) -> (Entity<AsyncView<String>>, Arc<AtomicBool>) {
    let report = keep(
        "report",
        || Report {
            fail: Arc::new(AtomicBool::new(false)),
        },
        window,
        cx,
    );
    let fail = report.read(cx).fail.clone();
    let executor = cx.background_executor().clone();
    let view: Entity<AsyncView<String>> =
        window.use_keyed_state("report-view", cx, move |_, cx| {
            AsyncView::new(
                "report",
                move || {
                    let (executor, fail) = (executor.clone(), fail.clone());
                    async move {
                        executor.timer(Duration::from_millis(1500)).await;
                        if fail.swap(false, Ordering::Relaxed) {
                            return Err(anyhow!("the report service answered 503"));
                        }
                        Ok("Revenue rose 12% on the quarter; churn held at 2.1%.".to_string())
                    }
                },
                |text, _, cx| {
                    div()
                        .py_6()
                        .text_size(cx.theme().text_size(TextSize::Base))
                        .child(text.clone())
                },
                cx,
            )
        });
    let breaker = report.read(cx).fail.clone();
    (view, breaker)
}

pub fn refresh(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let phase = keep("refresh-phase", || Refresh::Idle, window, cx);
    let (now, go) = (*phase.read(cx), phase.clone());
    section(
        "RefreshIndicator / TypingIndicator",
        "An arrow that turns with a pull, a spinner while it works, a check when done. Someone typing, as three dots in a bubble.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("pulling", RefreshIndicator::new("refresh-pull", Refresh::Pulling(0.3)), cx))
            .child(specimen("almost", RefreshIndicator::new("refresh-almost", Refresh::Pulling(0.85)), cx))
            .child(specimen("refreshing", RefreshIndicator::new("refresh-busy", Refresh::Refreshing), cx))
            .child(specimen("done", RefreshIndicator::new("refresh-done", Refresh::Done), cx)),
    )
    .child(
        row()
            .child(probe(
                "refresh-run",
                Button::new("refresh-run", "Refresh").on_click(move |_, _, cx| {
                    set(&go, Refresh::Refreshing, cx);
                    later(&go, Refresh::Done, Duration::from_millis(1200), cx);
                    later(&go, Refresh::Idle, Duration::from_millis(2600), cx);
                }),
            ))
            .child(div().w_48().child(RefreshIndicator::new("refresh-live", now))),
    )
    .child(
        specimens()
            .child(specimen("named", TypingIndicator::new("typing-mia").who("Mia"), cx))
            .child(specimen("alone", TypingIndicator::new("typing"), cx)),
    )
}

pub fn uploads(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let files = keep(
        "uploads",
        || {
            vec![
                Upload {
                    name: "brand-guidelines.pdf".into(),
                    bytes: 8_400_000,
                    state: UploadState::Uploading(0.62),
                },
                Upload {
                    name: "hero@2x.png".into(),
                    bytes: 2_430_000,
                    state: UploadState::Done,
                },
                Upload {
                    name: "interview.mov".into(),
                    bytes: 2_600_000_000,
                    state: UploadState::Failed("Over the 2 GB limit".into()),
                },
                Upload {
                    name: "notes.md".into(),
                    bytes: 12_800,
                    state: UploadState::Uploading(0.12),
                },
            ]
        },
        window,
        cx,
    );
    let list = files.read(cx).clone();
    let (cancel, retry, remove, tick) =
        (files.clone(), files.clone(), files.clone(), files.clone());
    let edit = |files: &Entity<Vec<Upload>>,
                ix: usize,
                cx: &mut App,
                change: fn(&mut Vec<Upload>, usize)| {
        let mut next = files.read(cx).clone();
        change(&mut next, ix);
        set(files, next, cx);
    };
    let theme = cx.theme();
    section(
        "UploadList",
        "Files on their way up: the percent and a bar that glides while they go, then their size and a check, or why they failed and a way to retry.",
        cx,
    )
    .child(
        div()
            .w_128()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .child(
                UploadList::new("uploads", list)
                    .on_cancel(move |ix, _, cx| edit(&cancel, ix, cx, |files, ix| { files.remove(ix); }))
                    .on_retry(move |ix, _, cx| {
                        edit(&retry, ix, cx, |files, ix| files[ix].state = UploadState::Uploading(0.0))
                    })
                    .on_remove(move |ix, _, cx| edit(&remove, ix, cx, |files, ix| { files.remove(ix); })),
            ),
    )
    .child(row().child(probe(
        "uploads-tick",
        Button::new("uploads-tick", "Send more")
            .variant(ButtonVariant::Ghost)
            .on_click(move |_, _, cx| {
                let next = tick
                    .read(cx)
                    .iter()
                    .cloned()
                    .map(|mut file| {
                        if let UploadState::Uploading(share) = file.state {
                            file.state = if share >= 0.8 { UploadState::Done } else { UploadState::Uploading(share + 0.2) };
                        }
                        file
                    })
                    .collect();
                set(&tick, next, cx)
            }),
    )))
    .child(Caption::new(
        "StepProgress is navigation::Steps, chapter 7. LoadingScreen is shell::SplashScreen, which also renders in place, chapter 4.",
    ))
}
