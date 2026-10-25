#![cfg_attr(target_family = "wasm", no_main)]
// A binary: nothing it holds is reachable from outside.
#![allow(unreachable_pub)]

/// A file under the gallery's assets, by name.
#[cfg(not(target_family = "wasm"))]
macro_rules! asset {
    ($name:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $name)
    };
}

/// On the web, the build names the pictures' address.
#[cfg(target_family = "wasm")]
macro_rules! asset {
    ($name:literal) => {
        concat!(env!("ELY_GALLERY_ASSETS"), $name)
    };
}

mod assets;
#[cfg(windows)]
mod audit;
#[cfg(not(target_family = "wasm"))]
mod capture;
mod pages;
mod probe;
#[cfg(not(target_family = "wasm"))]
mod script;
mod shell;
// Native capture alone plays the scripts.
#[cfg_attr(target_family = "wasm", allow(dead_code))]
mod step;
#[cfg(not(target_family = "wasm"))]
mod swipe;
mod ui;
#[cfg(target_family = "wasm")]
mod web;

#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;

#[cfg(not(target_family = "wasm"))]
use anyhow::bail;
use anyhow::{Context as _, Result};
#[cfg(not(target_family = "wasm"))]
use assets::GalleryAssets;
use ely_gpui_component::theme::{Mode, Theme};
use gpui::{
    App, AppContext, Bounds, KeyBinding, Menu, MenuItem, SharedString, TitlebarOptions,
    WindowBounds, WindowHandle, WindowOptions, actions, point, px, size,
};

actions!(gallery, [Quit, About, LightMode, DarkMode, NewWindow]);

/// The gallery's menu bar, drawn natively and by MenuBar.
pub fn app_menus() -> Vec<Menu> {
    let view = Menu {
        name: "View".into(),
        items: vec![
            MenuItem::action("Light", LightMode),
            MenuItem::action("Dark", DarkMode),
        ],
        disabled: false,
    };
    // A page opens no second window and cannot quit.
    if cfg!(target_family = "wasm") {
        return vec![view];
    }
    let ely = Menu {
        name: "Ely".into(),
        items: vec![
            MenuItem::action("About Ely", About),
            MenuItem::separator(),
            MenuItem::action("Quit Ely", Quit),
        ],
        disabled: false,
    };
    vec![ely, view]
}

/// Menu bar and Dock menu, set before the window opens.
fn menus(cx: &mut App) {
    cx.set_menus(app_menus());
    cx.set_dock_menu(vec![MenuItem::action("New Window", NewWindow)]);
}

/// What the menus' actions do; the theme ones need the window.
fn menu_actions(window: WindowHandle<shell::Gallery>, cx: &mut App) {
    cx.on_action(|_: &About, cx| pages::open_about(cx));
    cx.on_action(|_: &NewWindow, cx| pages::open_managed(cx));
    cx.on_action(move |_: &LightMode, cx| choose(window, shell::Choice::Light, cx));
    cx.on_action(move |_: &DarkMode, cx| choose(window, shell::Choice::Dark, cx));
}

fn choose(window: WindowHandle<shell::Gallery>, choice: shell::Choice, cx: &mut App) {
    if let Err(error) = window.update(cx, |gallery, window, cx| gallery.choose(choice, window, cx))
    {
        log::error!("gallery: menu could not reach the window: {error:#}");
    }
}

/// What the gallery opens on.
#[derive(Default)]
pub struct Start {
    pub page: usize,
    /// One section, by title or slug, drawn alone.
    pub story: Option<SharedString>,
    pub choice: shell::Choice,
    /// Narrow check width, if any.
    pub narrow: Option<f32>,
    /// The window's size when not the usual 1280 by 820.
    pub window: Option<gpui::Size<gpui::Pixels>>,
    /// Why the start could not be read, shown instead.
    pub failure: Option<SharedString>,
}

/// The window's size: the one asked for, else 1280 by 820.
fn asked(window: Option<gpui::Size<gpui::Pixels>>) -> gpui::Size<gpui::Pixels> {
    window.unwrap_or(size(px(1280.0), px(820.0)))
}

/// Sets Ely up and opens the gallery's window.
fn launch(start: Start, cx: &mut App) -> Result<WindowHandle<shell::Gallery>> {
    ely_gpui_component::init(cx)?;
    #[cfg(debug_assertions)]
    ely_gpui_component::tooling::install_inspector(cx);
    pages::bind_keys(cx);
    pages::install_i18n(cx);
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    cx.on_action(|_: &Quit, cx| cx.quit());
    menus(cx);
    if start.choice == shell::Choice::Dark {
        Theme::set_mode_now(Mode::Dark, cx);
    }
    let bounds = Bounds::centered(None, asked(start.window), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Ely".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(16.0), px(18.0))),
        }),
        window_min_size: Some(size(px(880.0), px(560.0))),
        // Keeps drawing while a demo window is key.
        inactive_frame_interval: None,
        ..Default::default()
    };
    let window = cx
        .open_window(options, |window, cx| {
            cx.new(|cx| shell::Gallery::new(start, window, cx))
        })
        .context("gallery window failed to open")?;
    menu_actions(window, cx);
    cx.activate(true);
    Ok(window)
}

#[cfg(not(target_family = "wasm"))]
struct Args {
    start: Start,
    /// The page asked for, which a capture shoots alone.
    page: Option<usize>,
    capture: Option<PathBuf>,
    /// Where the accessibility audit writes its report.
    audit: Option<PathBuf>,
    /// Holds every demo that runs on its own at its start.
    still: bool,
}

#[cfg(not(target_family = "wasm"))]
fn parse_args() -> Result<Args> {
    let mut args = Args {
        start: Start::default(),
        page: None,
        capture: None,
        audit: None,
        still: false,
    };
    let mut iter = std::env::args().skip(1);
    while let Some(flag) = iter.next() {
        if flag == "--still" {
            args.still = true;
            continue;
        }
        let value = iter
            .next()
            .with_context(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--page" => {
                args.page = Some(pages::find(&value).with_context(|| format!("no page {value}"))?)
            }
            "--story" => args.start.story = Some(value.into()),
            "--capture" => args.capture = Some(PathBuf::from(value)),
            "--audit" if cfg!(windows) => args.audit = Some(PathBuf::from(value)),
            // A size every runner's screen holds, so the system never trims the window.
            "--window" => {
                let (width, height) = value
                    .split_once('x')
                    .with_context(|| format!("--window takes WIDTHxHEIGHT, not {value}"))?;
                let parse = |side: &str| {
                    side.parse::<f32>()
                        .with_context(|| format!("bad size {value}"))
                };
                args.start.window = Some(size(px(parse(width)?), px(parse(height)?)));
            }
            "--narrow" => {
                args.start.narrow = Some(
                    value
                        .parse()
                        .with_context(|| format!("bad width {value}"))?,
                )
            }
            other => bail!("unknown flag {other}"),
        }
    }
    if args.start.story.is_some() && args.page.is_none() {
        bail!("--story needs --page");
    }
    args.start.page = args.page.unwrap_or(0);
    Ok(args)
}

#[cfg(not(target_family = "wasm"))]
fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let Args {
        start,
        page,
        capture,
        audit,
        still,
    } = parse_args()?;
    // A script aims across its page at 1280 wide; stories hold part. Its input is posted through AppKit, so only macOS plays one.
    let scripted = start.narrow.is_none()
        && start.story.is_none()
        && start.window.is_none()
        && cfg!(target_os = "macos");
    if capture.is_some() && cfg!(not(target_os = "macos")) {
        log::warn!("capture: scripted states need macOS, so only the pages are shot");
    } else if capture.is_some() && start.window.is_some() {
        log::warn!(
            "capture: scripts aim at the usual 1280 by 820 window, so only the pages are shot"
        );
    }
    // Clocks, dates and "minutes ago" draw one moment, so two captures compare pixel for pixel.
    if capture.is_some() {
        ely_gpui_component::typography::format::freeze_now(capture::AT.parse()?);
    }
    gpui_platform::application()
        .with_assets(GalleryAssets)
        .run(move |cx: &mut App| {
            if still {
                cx.set_global(ui::Still);
            }
            let size = asked(start.window);
            let window = match launch(start, cx) {
                Ok(window) => window,
                Err(error) => {
                    log::error!("gallery: {error:#}");
                    std::process::exit(1);
                }
            };
            if let Some(dir) = capture {
                capture::run(window, size, dir, page, scripted, cx);
            }
            #[cfg(windows)]
            if let Some(report) = audit {
                audit::run(window, report, page, cx);
            }
            #[cfg(not(windows))]
            let _ = audit;
        });
    Ok(())
}
