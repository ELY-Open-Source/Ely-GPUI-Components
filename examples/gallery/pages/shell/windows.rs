use ely_gpui_component::{
    buttons::{Button, ButtonVariant, IconButton},
    primitives::{IconName, Image},
    shell::{
        AboutDialog, Corner, MiniWindow, SplashScreen, TitleBar, WindowManager, WindowSwitcher,
    },
    theme::{ActiveTheme, ControlSize, Radius, TextSize},
    typography::{Caption, Label, Title},
};
use gpui::{
    App, AppContext, Context, IntoElement, ObjectFit, ParentElement, Render, SharedString, Styled,
    Window, div, prelude::*, px, size,
};

use super::chrome::window_frame;
use crate::{
    probe::{Opened, probe},
    ui::{code, native_only, picture, row, section, specimen},
};

const DUNES: &str = asset!("dunes.jpg");
const ATRIUM: &str = asset!("atrium.jpg");

fn about() -> AboutDialog {
    AboutDialog::new("Ely", "0.1.0 (2026.09)")
        .description("A component library for GPUI, in light and dark.")
        .link(
            "Website",
            "https://github.com/ELY-Open-Source/Ely-GPUI-Components",
        )
        .link(
            "License",
            "https://github.com/ELY-Open-Source/Ely-GPUI-Components#license",
        )
        .copyright("© 2026 Ely contributors")
}

fn splash() -> SplashScreen {
    SplashScreen::new("Ely")
        .tagline("Components for GPUI")
        .status("Loading workspace…")
}

/// Opens a window and records it for scripts, or logs why it failed.
fn opened<V: 'static>(
    key: &'static str,
    result: anyhow::Result<gpui::WindowHandle<V>>,
    cx: &mut App,
) {
    match result {
        Ok(handle) => Opened::insert(key, handle.into(), cx),
        Err(error) => log::error!("gallery: {key} window failed: {error:#}"),
    }
}

fn trigger(
    key: &'static str,
    label: &'static str,
    open: impl Fn(&mut App) + 'static,
) -> impl IntoElement {
    probe(
        key,
        Button::new(key, label)
            .variant(ButtonVariant::Outline)
            .icon(IconName::AppWindow)
            .on_click(move |_, _, cx| open(cx)),
    )
}

pub fn splash_screen(cx: &App) -> impl IntoElement + use<> {
    section(
        "SplashScreen",
        "Mark and name rise in; the bar reports progress or sweeps.",
        cx,
    )
    .child(specimen(
        "Inline, at 62%",
        window_frame(480.0, 280.0, cx)
            .bg(cx.theme().colors.bg)
            .child(splash().progress(Some(0.62))),
        cx,
    ))
    .child(native_only(trigger(
        "splash-open",
        "Show splash window",
        |cx| {
            let result = splash().open(cx);
            opened("splash", result, cx)
        },
    )))
}

pub fn open_about(cx: &mut App) {
    let result = about().open(cx);
    opened("about", result, cx)
}

pub fn about_dialog(cx: &App) -> impl IntoElement + use<> {
    section(
        "AboutDialog",
        "Name, version to copy, links, fine print.",
        cx,
    )
    .child(specimen(
        "Inline",
        window_frame(360.0, 360.0, cx)
            .bg(cx.theme().colors.bg)
            .pt_6()
            .child(about()),
        cx,
    ))
    .child(native_only(trigger(
        "about-open",
        "Open About window",
        open_about,
    )))
}

struct Managed {
    title: SharedString,
    asked: bool,
}

impl Render for Managed {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let draft = cx.entity().downgrade();
        let theme = cx.theme();
        let asked = self.asked;
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.bg)
            .text_color(theme.colors.fg)
            .font_family(theme.font_family.clone())
            .child({
                let bar = TitleBar::new("draft-bar").title(self.title.clone());
                if asked {
                    bar
                } else {
                    bar.on_close(move |_, cx| {
                        let asked = draft.update(cx, |draft, cx| {
                            draft.asked = true;
                            cx.notify();
                        });
                        if let Err(error) = asked {
                            log::error!("gallery: draft gone before close: {error:#}");
                        }
                    })
                }
            })
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .child(Title::new(self.title.clone()))
                    .child(Caption::new(if asked {
                        "The title bar caught the close. The next one goes through."
                    } else {
                        "Opened by WindowManager. Its close button asks first."
                    }))
                    .when(asked, |body| {
                        body.child(
                            Button::new("draft-close", "Close anyway")
                                .size(ControlSize::Sm)
                                .on_click(|_, window, _| window.remove_window()),
                        )
                    }),
            )
    }
}

/// Opens the next numbered draft window.
pub fn open_managed(cx: &mut App) {
    let number = WindowManager::windows(cx).len() + 1;
    let title = SharedString::from(format!("Draft {number}"));
    let result = WindowManager::open(title.clone(), size(px(420.0), px(280.0)), cx, |_, cx| {
        cx.new(|_| Managed {
            title,
            asked: false,
        })
    });
    opened("managed", result, cx)
}

pub fn window_manager(cx: &mut App) -> impl IntoElement + use<> {
    let open = WindowManager::windows(cx);
    section(
        "MultiWindow Manager",
        "Each new window cascades from the last; the manager knows them by name.",
        cx,
    )
    .child(native_only(trigger(
        "managed-open",
        "New window",
        open_managed,
    )))
    .child(
        row().children(open.into_iter().enumerate().map(|(ix, (handle, title))| {
            Button::new(("managed-focus", ix), title)
                .size(ControlSize::Sm)
                .variant(ButtonVariant::Subtle)
                .on_click(move |_, _, cx| {
                    if let Err(error) = WindowManager::focus(handle, cx) {
                        log::error!("gallery: focus failed: {error:#}");
                    }
                })
        })),
    )
}

pub fn window_switcher(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let shown = window.use_keyed_state("switcher-shown", cx, |_, _| false);
    let open = *shown.read(cx);
    let (show, hide) = (shown.clone(), shown);
    section(
        "WindowSwitcher",
        "Managed windows over a scrim. Arrows or Tab move, Enter goes.",
        cx,
    )
    .child(probe(
        "switcher-open",
        Button::new("switcher-open", "Switch windows")
            .variant(ButtonVariant::Outline)
            .on_click(move |_, _, cx| {
                show.update(cx, |shown, cx| {
                    *shown = true;
                    cx.notify();
                })
            }),
    ))
    .when(open, |block| {
        block.child(WindowSwitcher::new("switcher", move |_, cx| {
            hide.update(cx, |shown, cx| {
                *shown = false;
                cx.notify();
            })
        }))
    })
}

pub fn quick_launcher(cx: &App) -> impl IntoElement + use<> {
    section(
        "QuickLauncher",
        "A search palette in a window above all others.",
        cx,
    )
    .child(Caption::new(
        "It is `navigation::QuickLauncher`; the Navigation page, chapter 7, opens one.",
    ))
}

struct Player {
    playing: bool,
}

impl Render for Player {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let playing = self.playing;
        div()
            .size_full()
            .flex()
            .items_center()
            .gap_3()
            .px_3()
            .child(
                div()
                    .size(theme.control_height(ControlSize::Lg) * 2.0)
                    .rounded(theme.radius(Radius::Md))
                    .overflow_hidden()
                    .child(
                        Image::new("player-art", picture(DUNES))
                            .fit(ObjectFit::Cover)
                            .size_full(),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(Label::new("Quiet Hours"))
                    .child(Caption::new("Low Tide · 2:41")),
            )
            .child(
                IconButton::new(
                    "player-toggle",
                    if playing {
                        IconName::Pause
                    } else {
                        IconName::Play
                    },
                )
                .on_click(cx.listener(|player, _, _, cx| {
                    player.playing = !player.playing;
                    cx.notify();
                })),
            )
    }
}

struct Picture;

impl Render for Picture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .relative()
            .size_full()
            .child(
                Image::new("pip-frame", picture(ATRIUM))
                    .fit(ObjectFit::Cover)
                    .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .left_3()
                    .bottom_3()
                    .px_2()
                    .rounded(theme.radius(Radius::Sm))
                    .bg(theme.colors.tooltip_bg.opacity(0.85))
                    .text_color(theme.colors.tooltip_fg)
                    .text_size(theme.text_size(TextSize::Xs))
                    .child("12:04 / 48:10"),
            )
    }
}

pub fn mini_windows(cx: &App) -> impl IntoElement + use<> {
    section(
        "MiniWindow / CompactMode / PictureInPicture",
        "Small windows above the rest. Hover for the bar; expand returns to the app.",
        cx,
    )
    .child(native_only(
        row()
            .child(trigger("mini-open", "Mini player", |cx| {
                let view = cx.new(|_| Player { playing: true });
                let result = MiniWindow::open(
                    "Now playing",
                    size(px(320.0), px(96.0)),
                    Some(Corner::BottomRight),
                    view.into(),
                    cx,
                );
                opened("mini", result, cx)
            }))
            .child(trigger("pip-open", "Picture in picture", |cx| {
                let view = cx.new(|_| Picture);
                let result = MiniWindow::open(
                    "Picture in picture",
                    size(px(384.0), px(216.0)),
                    Some(Corner::TopRight),
                    view.into(),
                    cx,
                );
                opened("pip", result, cx)
            })),
    ))
    .child(code(
        "AlwaysOnTop is gpui's WindowOptions { kind: WindowKind::PopUp, .. }",
        cx,
    ))
}
