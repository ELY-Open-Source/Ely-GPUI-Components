use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    desktop::{TrayIcon, TrayItem, TrayLook, dock_badge, set_dock_badge},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Window};

use crate::{
    probe::probe,
    ui::{blocked, row, section},
};

const ITEMS: [&str; 3] = ["Open gallery", "Toggle theme", "Quit"];

pub fn tray(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tray = window.use_keyed_state("tray", cx, |_, _| None::<TrayIcon>);
    let last = window.use_keyed_state("tray-last", cx, |_, _| SharedString::from("nothing yet"));
    let shown = tray.read(cx).is_some();
    let note = last.read(cx).clone();
    let (toggle, pick) = (tray.clone(), tray);
    section(
        "TrayIcon / TrayMenu",
        "An icon in the menu bar or tray, with a menu.",
        cx,
    )
    .child(
        row()
            .child(probe(
                "tray-show",
                Button::new(
                    "tray-show",
                    if shown {
                        "Remove tray icon"
                    } else {
                        "Show tray icon"
                    },
                )
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| {
                    let last = last.clone();
                    let next = match toggle.read(cx) {
                        Some(_) => None,
                        None => {
                            let items = vec![
                                TrayItem::Action(ITEMS[0].into()),
                                TrayItem::Action(ITEMS[1].into()),
                                TrayItem::Separator,
                                TrayItem::Action(ITEMS[2].into()),
                            ];
                            let look = TrayLook {
                                symbol: "circle.hexagongrid",
                                png: include_bytes!("../../../assets/stickers/olive.png"),
                                name: "Gallery".into(),
                            };
                            let made = TrayIcon::new(
                                look,
                                items,
                                move |index, cx| {
                                    let label = match index {
                                        0 => ITEMS[0],
                                        1 => ITEMS[1],
                                        3 => ITEMS[2],
                                        other => panic!("tray has no action at {other}"),
                                    };
                                    last.update(cx, |last, cx| {
                                        *last = label.into();
                                        cx.notify();
                                    })
                                },
                                cx,
                            );
                            match made {
                                Ok(icon) => Some(icon),
                                Err(error) => {
                                    log::error!("gallery: tray failed: {error:#}");
                                    None
                                }
                            }
                        }
                    };
                    toggle.update(cx, |tray, cx| {
                        *tray = next;
                        cx.notify();
                    })
                }),
            ))
            .child(probe(
                "tray-pick",
                Button::new("tray-pick", "Pick “Open gallery”")
                    .variant(ButtonVariant::Ghost)
                    .disabled(!shown)
                    .on_click(move |_, _, cx| {
                        if let Some(icon) = pick.read(cx) {
                            icon.pick(0);
                        }
                    }),
            ))
            .child(Caption::new(format!("Last pick: {note}"))),
    )
}

pub fn dock(cx: &mut App) -> impl IntoElement + use<> {
    let reading = match dock_badge() {
        Ok(Some(label)) => format!("The Dock shows “{label}”."),
        Ok(None) => "The Dock shows no badge.".to_string(),
        Err(error) => format!("{error:#}"),
    };
    let badge = |label: Option<&'static str>| {
        move |_: &gpui::ClickEvent, window: &mut Window, _: &mut App| {
            if let Err(error) = set_dock_badge(label) {
                log::error!("gallery: badge failed: {error:#}");
            }
            window.refresh();
        }
    };
    section(
        "DockBadge",
        "A count on the app's Dock icon. macOS only.",
        cx,
    )
    .child(
        row()
            .child(probe(
                "badge-set",
                Button::new("badge-set", "Badge 3")
                    .variant(ButtonVariant::Outline)
                    .on_click(badge(Some("3"))),
            ))
            .child(probe(
                "badge-clear",
                Button::new("badge-clear", "Clear")
                    .variant(ButtonVariant::Ghost)
                    .on_click(badge(None)),
            ))
            .child(Caption::new(reading)),
    )
}

pub fn notification(cx: &App) -> impl IntoElement + use<> {
    section(
        "SystemNotification",
        "Notices from the system's own center.",
        cx,
    )
    .child(blocked(
        if cfg!(target_family = "wasm") {
            "Not shown: gpui on the web posts no system notification."
        } else {
            "Not yet shown: gpui posts a system notification only from an app bundle, and the gallery runs unbundled."
        },
        cx,
    ))
}
