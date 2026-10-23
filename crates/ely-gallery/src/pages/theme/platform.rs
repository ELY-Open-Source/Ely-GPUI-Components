use ely_gpui_component::{
    buttons::{Button, SegmentedControl},
    files::FileIcon,
    forms::Switch,
    primitives::{IconName, IconTheme},
    shell::{Vibrancy, WindowControls},
    theme::{ActiveTheme, Platform, Theme},
    typography::{Kbd, KbdCombo},
};
use gpui::{
    App, AppContext, Bounds, Context, IntoElement, ParentElement, Render, Styled, TitlebarOptions,
    Window, WindowBackgroundAppearance, WindowBounds, WindowOptions, div, point, px, size,
};

use crate::{
    probe::{Opened, probe},
    ui::{change, keep, native_only, section, specimen, specimens},
};

const PLATFORMS: [(Platform, &str); 3] = [
    (Platform::Mac, "macOS"),
    (Platform::Windows, "Windows"),
    (Platform::Linux, "Linux"),
];

/// Whether the demo's own icon rules are on.
struct Owned(bool);

pub fn platform(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    section(
        "Platform Style · IconThemeProvider · VibrancyBackground",
        "The theme's platform decides window buttons and key caps, and starts as the system's. An icon theme lays an owner's rules over Ely's file icons. A frosted pane lets a blurred window's backdrop through; gpui blurs as vibrancy on macOS and acrylic on Windows, and sets Mica on Windows 11.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("Platform", platforms(cx), cx))
            .child(specimen("IconTheme", icons(window, cx), cx))
            .child(native_only(specimen(
                "Vibrancy",
                probe(
                        "frosted-open",
                        Button::new("frosted-open", "Open a frosted window")
                            .on_click(|_, _, cx| frosted(cx)),
                    ),
                cx,
            ))),
    )
}

fn platforms(cx: &App) -> impl IntoElement + use<> {
    let shown = cx.theme().platform;
    let label = PLATFORMS
        .iter()
        .find(|(platform, _)| *platform == shown)
        .map(|(_, label)| *label)
        .expect("a listed platform");
    let control = PLATFORMS.iter().fold(
        SegmentedControl::new("platform", label),
        |control, (_, label)| control.segment(*label, *label, None),
    );
    div()
        .w(px(320.))
        .flex()
        .flex_col()
        .gap_4()
        .child(control.on_change(|picked, _, cx| {
            let (platform, _) = PLATFORMS
                .iter()
                .find(|(_, label)| *label == picked.as_ref())
                .expect("a listed platform");
            Theme::update(cx, |theme| theme.platform = *platform);
        }))
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap_3()
                .child(WindowControls::new("platform-controls"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(Kbd::new("secondary-s"))
                        .child(KbdCombo::new("secondary-k secondary-s")),
                ),
        )
}

fn icons(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let owned = keep("icon-theme", || Owned(false), window, cx);
    let on = owned.read(cx).0;
    let muted = cx.theme().colors.fg_muted;
    let rows = ["Cargo.toml", "README.md", "main.rs", "logo.svg"].map(|name| {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(FileIcon::file(name))
            .child(div().text_color(muted).child(name))
    });
    div()
        .w(px(240.))
        .flex()
        .flex_col()
        .gap_2()
        .child(
            Switch::new("icon-theme", on)
                .label("The owner's icons")
                .on_change(move |on, _, cx| {
                    let theme = if on {
                        IconTheme::default()
                            .name("Cargo.toml", IconName::Package)
                            .name("README.md", IconName::BookOpen)
                            .extension("rs", IconName::Braces)
                    } else {
                        IconTheme::default()
                    };
                    theme.apply(cx);
                    change(&owned, cx, |owned| owned.0 = on);
                }),
        )
        .children(rows)
}

fn frosted(cx: &mut App) {
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(560.), px(340.)),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("Frosted".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(16.0), px(18.0))),
        }),
        window_background: WindowBackgroundAppearance::Blurred,
        ..Default::default()
    };
    match cx.open_window(options, |_, cx| cx.new(|_| Frosted)) {
        Ok(handle) => Opened::insert("frosted", handle.into(), cx),
        Err(error) => log::error!("gallery: frosted window failed: {error:#}"),
    }
}

/// A window whose sidebar lets the desktop through.
struct Frosted;

impl Render for Frosted {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        div()
            .size_full()
            .flex()
            .font_family(theme.font_family.clone())
            .text_color(colors.fg)
            .child(
                Vibrancy::new()
                    .w(px(180.))
                    .h_full()
                    .pt_12()
                    .px_3()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(
                        ["Library", "Recents", "Shared"]
                            .map(|name| div().px_2().py_1().child(name)),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .p_6()
                    .pt_12()
                    .bg(colors.bg)
                    .border_l_1()
                    .border_color(colors.border)
                    .min_w_0()
                    .text_color(colors.fg_muted)
                    .child(
                        div().child("The sidebar lets the desktop through; this pane stays solid."),
                    ),
            )
    }
}
