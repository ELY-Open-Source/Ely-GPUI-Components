use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    shell::{
        Connectivity, CrashReporter, OfflineIndicator, UpdateBanner, UpdateDialog, UpdateState,
        ZoomControl,
    },
    theme::ActiveTheme,
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div};

use super::chrome::window_frame;
use crate::{
    probe::{Opened, probe},
    ui::{native_only, row, section, specimen},
};

fn log_action(name: &'static str) -> impl Fn(&mut Window, &mut App) + 'static {
    move |_, _| log::info!("gallery: update {name}")
}

pub fn update_banner(cx: &App) -> impl IntoElement + use<> {
    let banner = |id: &'static str, state| {
        UpdateBanner::new(id, "0.2.0", state)
            .on_download(log_action("download"))
            .on_restart(log_action("restart"))
            .on_later(log_action("later"))
    };
    section(
        "UpdateBanner",
        "Offer, download, restart. One slim bar, three states.",
        cx,
    )
    .child(
        window_frame(560.0, 126.0, cx)
            .bg(cx.theme().colors.bg)
            .child(banner("banner-available", UpdateState::Available))
            .child(banner("banner-downloading", UpdateState::Downloading(0.42)))
            .child(banner("banner-ready", UpdateState::Ready)),
    )
}

fn update_dialog() -> UpdateDialog {
    UpdateDialog::new("Ely", "0.1.0", "0.2.0", UpdateState::Available)
        .note("Window chrome for macOS, Windows and Linux.")
        .note("Docks remember their layout across launches.")
        .note("Sticky headers stay inside their scroll area.")
        .note("Drawers follow the pointer outside their bounds.")
        .on_download(log_action("download"))
        .on_later(|window, _| window.remove_window())
}

pub fn update_window(cx: &App) -> impl IntoElement + use<> {
    section("UpdateDialog", "What changed, then one clear action.", cx)
        .child(specimen(
            "Inline",
            window_frame(400.0, 380.0, cx)
                .bg(cx.theme().colors.bg)
                .pt_6()
                .child(update_dialog().on_later(log_action("later"))),
            cx,
        ))
        .child(native_only(probe(
            "update-open",
            Button::new("update-open", "Open update window")
                .variant(ButtonVariant::Outline)
                .on_click(|_, _, cx| match update_dialog().open(cx) {
                    Ok(handle) => Opened::insert("update", handle.into(), cx),
                    Err(error) => log::error!("gallery: update window failed: {error:#}"),
                }),
        )))
}

const REPORT: &str = "Exception Type:  EXC_BAD_ACCESS (SIGSEGV)
Crashed Thread:  0  main
0   ely           0x0000000104a1c3f0 layout::dock::restore + 112
1   ely           0x0000000104a1b9d4 workspace::apply + 88
2   ely           0x0000000104a0f2a8 app::open_recent + 404
3   libdyld.dylib 0x000000018f2d4f28 start + 4";

pub fn crash_reporter(cx: &App) -> impl IntoElement + use<> {
    let reporter = |details: bool| {
        CrashReporter::new(
            "Ely",
            "The report says where it stopped. Sending it helps fix it.",
            REPORT,
        )
        .details(details)
        .on_send(|_, _| log::info!("gallery: crash report sent"))
        .on_dismiss(|_, _| log::info!("gallery: crash report dismissed"))
    };
    section(
        "CrashReporter",
        "Plain words first. The report waits behind a disclosure.",
        cx,
    )
    .child(specimen(
        "Inline, report shown",
        window_frame(420.0, 520.0, cx)
            .bg(cx.theme().colors.bg)
            .pt_6()
            .child(reporter(true)),
        cx,
    ))
    .child(native_only(probe(
        "crash-open",
        Button::new("crash-open", "Open crash window")
            .variant(ButtonVariant::Outline)
            .on_click(move |_, _, cx| match reporter(false).open(cx) {
                Ok(handle) => Opened::insert("crash", handle.into(), cx),
                Err(error) => log::error!("gallery: crash window failed: {error:#}"),
            }),
    )))
}

pub fn offline(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = window.use_keyed_state("connectivity", cx, |_, _| Connectivity::Online);
    let current = *state.read(cx);
    let choices = [
        ("offline-online", "Online", Connectivity::Online),
        ("offline-offline", "Offline", Connectivity::Offline),
        (
            "offline-reconnecting",
            "Reconnecting",
            Connectivity::Reconnecting,
        ),
    ];
    section(
        "OfflineIndicator",
        "Quiet while it matters, gone soon after the network returns.",
        cx,
    )
    .child(row().children(choices.map(|(key, label, choice)| {
        let state = state.clone();
        probe(
            key,
            Button::new(key, label)
                .size(ely_gpui_component::theme::ControlSize::Sm)
                .variant(if current == choice {
                    ButtonVariant::Subtle
                } else {
                    ButtonVariant::Ghost
                })
                .on_click(move |_, _, cx| {
                    state.update(cx, |state, cx| {
                        *state = choice;
                        cx.notify();
                    })
                }),
        )
    })))
    .child(
        div()
            .h(cx
                .theme()
                .control_height(ely_gpui_component::theme::ControlSize::Lg))
            .flex()
            .items_center()
            .child(OfflineIndicator::new("connectivity-pill", current)),
    )
}

pub fn zoom(cx: &App) -> impl IntoElement + use<> {
    section(
        "ZoomControl",
        "Scales the whole window through its rem size. Click the number to reset.",
        cx,
    )
    .child(probe("zoom", ZoomControl::new("zoom-control")))
}
