use std::time::Duration;

use anyhow::anyhow;
use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    feedback::{
        ConfirmationCard, ConnectionStatus, Countdown, ErrorBoundary, ErrorView, InlineMessage,
        ResultView, SaveState, SavingIndicator, SyncState, SyncStatus, Timer,
    },
    primitives::Severity,
    shell::Connectivity,
    theme::{ActiveTheme, Radius, TextSize},
    typography::Caption,
};
use gpui::{App, Div, IntoElement, ParentElement, Styled, Window, div, prelude::FluentBuilder};
use web_time::Instant;

use crate::{
    probe::probe,
    ui::{keep, row, section, set, specimen, specimens},
};

fn frame(cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .flex_1()
        .min_w_0()
        .h_80()
        .flex()
        .flex_col()
        .justify_center()
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border)
}

pub fn errors(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "ErrorView / NotFound / NoPermission / Maintenance / Offline",
        "A view that could not show what was asked says what happened and offers a way on.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_3()
            .child(frame(cx).child(
                ErrorView::not_found("error-404").action(Button::new("error-home", "Go home")),
            ))
            .child(
                frame(cx).child(
                    ErrorView::no_permission("error-403")
                        .action(Button::new("error-ask", "Request access").primary()),
                ),
            ),
    )
    .child(
        div()
            .flex()
            .gap_3()
            .child(frame(cx).child(ErrorView::maintenance("error-maintenance")))
            .child(frame(cx).child(
                ErrorView::offline("error-offline").action(Button::new("error-retry", "Try again")),
            )),
    )
}

pub fn boundary(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let broken = keep("boundary-broken", || false, window, cx);
    let (now, toggle, retry) = (*broken.read(cx), broken.clone(), broken.clone());
    let theme = cx.theme();
    let content = if now {
        Err(anyhow!("the export service answered 503").context("Revenue, third quarter"))
    } else {
        Ok(div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .text_color(theme.colors.fg_muted)
            .child("Revenue, third quarter: 1.2 M"))
    };
    section(
        "ErrorBoundary",
        "Shows its content, or the error it holds, logged once. It renders a Result; it never catches a panic.",
        cx,
    )
    .child(
        div().flex().child(
            frame(cx).child(
                ErrorBoundary::new("boundary", content)
                    .on_retry(move |_, cx| set(&retry, false, cx)),
            ),
        ),
    )
    .child(
        row().child(probe(
            "boundary-break",
            Button::new("boundary-toggle", if now { "Fix it" } else { "Break it" })
                .on_click(move |_, _, cx| set(&toggle, !now, cx)),
        )),
    )
}

pub fn results(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let turn = keep("result-turn", || 0usize, window, cx);
    let (now, replay) = (*turn.read(cx), turn.clone());
    section(
        "Result",
        "Where a task ended. The ring and its mark draw themselves once.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_3()
            .child(
                frame(cx).child(
                    ResultView::success(("result-ok", now), "Payment sent")
                        .body("€240.00 to Studio Ottanta. A receipt is on its way.")
                        .action(Button::new("result-done", "Done").primary())
                        .action(Button::new("result-receipt", "View receipt")),
                ),
            )
            .child(
                frame(cx).child(
                    ResultView::failure(("result-no", now), "Payment declined")
                        .body("Your bank declined the charge. No money moved.")
                        .action(Button::new("result-again", "Try again")),
                ),
            ),
    )
    .child(
        row().child(probe(
            "result-replay",
            Button::new("result-replay", "Replay")
                .variant(ButtonVariant::Ghost)
                .on_click(move |_, _, cx| set(&replay, now + 1, cx)),
        )),
    )
}

pub fn confirmations(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let said = keep("confirm-said", || None::<&'static str>, window, cx);
    let theme = cx.theme();
    let answer = |what: &'static str| {
        let said = said.clone();
        move |_: &mut Window, cx: &mut App| set(&said, Some(what), cx)
    };
    let command = div()
        .px_3()
        .py_2()
        .rounded(theme.radius(Radius::Md))
        .bg(theme.colors.sunken)
        .font_family(theme.mono_family.clone())
        .text_size(theme.text_size(TextSize::Sm))
        .child("cargo clean && cargo build --release");
    section(
        "ConfirmationCard",
        "Asks in place before an action runs, with what it will do.",
        cx,
    )
    .child(
        div()
            .w_128()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                ConfirmationCard::new("confirm-run", "Run this command?")
                    .body("The agent wants to rebuild the project from scratch.")
                    .confirm("Run")
                    .on_confirm(answer("Ran the command"))
                    .on_cancel(answer("Kept the build"))
                    .child(command),
            )
            .child(
                ConfirmationCard::new("confirm-delete", "Delete 3 branches?")
                    .body("feature/toasts, feature/menus and old/draft go for good.")
                    .confirm("Delete")
                    .destructive()
                    .on_confirm(answer("Deleted 3 branches"))
                    .on_cancel(answer("Kept the branches")),
            )
            .when_some(*said.read(cx), |cards, what| {
                cards.child(InlineMessage::new(Severity::Info, what))
            }),
    )
}

pub fn statuses(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    const SAVES: [SaveState; 3] = [SaveState::Saving, SaveState::Saved, SaveState::Failed];
    const SYNCS: [SyncState; 4] = [
        SyncState::Syncing(12),
        SyncState::Synced,
        SyncState::Paused,
        SyncState::Failed,
    ];
    const LINKS: [Connectivity; 3] = [
        Connectivity::Online,
        Connectivity::Reconnecting,
        Connectivity::Offline,
    ];
    let at = keep("status-states", || [0usize; 3], window, cx);
    let now = *at.read(cx);
    let next = |slot: usize, count: usize| {
        let at = at.clone();
        move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
            let mut then = now;
            then[slot] = (now[slot] + 1) % count;
            set(&at, then, cx)
        }
    };
    let retry = at.clone();
    section(
        "SavingIndicator / SyncStatus / ConnectionStatus",
        "Quiet lines for work in the background. Each rises in when its state changes; sync arrows turn while they work.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "saving",
                row()
                    .child(probe(
                        "saving-next",
                        Button::new("saving-next", "Next").on_click(next(0, SAVES.len())),
                    ))
                    .child(
                        SavingIndicator::new("saving", SAVES[now[0]]).on_retry(move |_, cx| {
                            let mut then = now;
                            then[0] = 0;
                            set(&retry, then, cx)
                        }),
                    ),
                cx,
            ))
            .child(specimen(
                "sync",
                row()
                    .child(probe(
                        "sync-next",
                        Button::new("sync-next", "Next").on_click(next(1, SYNCS.len())),
                    ))
                    .child(SyncStatus::new("sync", SYNCS[now[1]])),
                cx,
            ))
            .child(specimen(
                "connection",
                row()
                    .child(probe(
                        "connection-next",
                        Button::new("connection-next", "Next").on_click(next(2, LINKS.len())),
                    ))
                    .child(ConnectionStatus::new("connection", LINKS[now[2]])),
                cx,
            )),
    )
}

pub fn clocks(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let opened = keep("clock-opened", Instant::now, window, cx);
    let short = keep(
        "clock-short",
        || Instant::now() + Duration::from_secs(10),
        window,
        cx,
    );
    let stopped = keep("clock-stopped", || None::<Instant>, window, cx);
    let over = keep("clock-over", || false, window, cx);
    let (since, until, halt) = (*opened.read(cx), *short.read(cx), *stopped.read(cx));
    let (restart, toggle, done, reset) =
        (short.clone(), stopped.clone(), over.clone(), opened.clone());
    let reopen = over.clone();
    section(
        "Countdown / Timer",
        "Time left and time spent. Digits roll as they change.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "launch in",
                Countdown::new(
                    "countdown-launch",
                    since + Duration::from_secs(2 * 3_600 + 14 * 60 + 7),
                ),
                cx,
            ))
            .child(specimen(
                "ten seconds",
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        Countdown::new("countdown-short", until)
                            .on_done(move |_, cx| set(&done, true, cx)),
                    )
                    .child(probe(
                        "countdown-restart",
                        Button::new("countdown-restart", "Restart")
                            .variant(ButtonVariant::Ghost)
                            .on_click(move |_, _, cx| {
                                set(&restart, Instant::now() + Duration::from_secs(10), cx);
                                set(&reopen, false, cx);
                            }),
                    )),
                cx,
            ))
            .child(specimen(
                "on this page",
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(Timer::new("timer", since).stopped(halt))
                    .child(
                        Button::new(
                            "timer-toggle",
                            if halt.is_some() { "Reset" } else { "Stop" },
                        )
                        .variant(ButtonVariant::Ghost)
                        .on_click(move |_, _, cx| match halt {
                            Some(_) => {
                                set(&reset, Instant::now(), cx);
                                set(&toggle, None, cx);
                            }
                            None => set(&toggle, Some(Instant::now()), cx),
                        }),
                    ),
                cx,
            )),
    )
    .when(*over.read(cx), |section| {
        section.child(Caption::new("The ten seconds ran out; on_done ran once."))
    })
}
