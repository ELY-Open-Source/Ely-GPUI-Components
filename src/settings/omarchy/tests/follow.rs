use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use gpui::{App, TestAppContext};

use super::super::{follow::Following, follow_omarchy, read_omarchy_theme, stop_omarchy};
use crate::theme::{ActiveTheme, Mode, Palette, Theme};

const LATTE: &str = include_str!("../themes/catppuccin-latte.toml");
const NIGHT: &str = include_str!("../themes/tokyo-night.toml");

/// A fresh folder under the system's temporary one.
fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ely-follow-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Sets a theme as `omarchy-theme-set` does: stage, remove, move in, name.
fn install(current: &Path, name: &str, colors: Option<&str>) {
    let (next, theme) = (current.join("next-theme"), current.join("theme"));
    fs::create_dir_all(&next).unwrap();
    if let Some(colors) = colors {
        fs::write(next.join("colors.toml"), colors).unwrap();
    }
    if theme.exists() {
        fs::remove_dir_all(&theme).unwrap();
    }
    fs::rename(&next, &theme).unwrap();
    fs::write(current.join("theme.name"), format!("{name}\n")).unwrap();
}

fn colors_of(tag: &str, text: &str) -> Palette {
    let current = folder(&format!("colors-{tag}"));
    install(&current, "probe", Some(text));
    let colors = read_omarchy_theme(&current).unwrap().unwrap().colors;
    fs::remove_dir_all(&current).unwrap();
    colors
}

fn start(cx: &mut TestAppContext) {
    cx.update(|cx| {
        Theme::init(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
    });
    cx.executor().allow_parking();
}

fn wearing(cx: &mut TestAppContext) -> (Mode, Palette) {
    cx.update(|cx| (cx.theme().mode(), cx.theme().palette()))
}

/// Runs the watcher's thread and the settle until `done`, or fails after five seconds.
fn until(cx: &mut TestAppContext, what: &str, done: impl Fn(&App) -> bool) {
    let deadline = web_time::Instant::now() + Duration::from_secs(5);
    while web_time::Instant::now() < deadline {
        cx.executor().advance_clock(Duration::from_millis(300));
        cx.run_until_parked();
        if cx.update(|cx| done(cx)) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} within five seconds");
}

/// Gives the watcher a second to act on what it should not act on.
fn idle(cx: &mut TestAppContext) {
    for _ in 0..25 {
        cx.executor().advance_clock(Duration::from_millis(300));
        cx.run_until_parked();
        std::thread::sleep(Duration::from_millis(40));
    }
}

#[gpui::test]
fn following_wears_the_theme_at_once_and_each_switch_omarchy_makes(cx: &mut TestAppContext) {
    start(cx);
    let current = folder("switch");
    install(&current, "catppuccin-latte", Some(LATTE));
    let (latte, night) = (
        colors_of("switch-latte", LATTE),
        colors_of("switch-night", NIGHT),
    );
    cx.update(|cx| follow_omarchy(current.clone(), cx)).unwrap();
    assert_eq!(wearing(cx), (Mode::Light, latte.clone()));
    assert_eq!(
        cx.update(|cx| cx.theme().colors.clone()),
        latte,
        "no fade at first"
    );

    install(&current, "tokyo-night", Some(NIGHT));
    until(cx, "tokyo night is worn", |cx| {
        cx.theme().mode() == Mode::Dark && cx.theme().palette() == night
    });
    cx.update(|cx| Theme::set_mode_now(Mode::Light, cx));
    assert_eq!(
        wearing(cx).1,
        Palette::light(false),
        "the other mode is Ely's"
    );
    cx.update(stop_omarchy);
    fs::remove_dir_all(&current).unwrap();
}

#[gpui::test]
fn a_theme_that_cannot_be_read_keeps_the_colors_and_the_next_good_one_is_worn(
    cx: &mut TestAppContext,
) {
    start(cx);
    let current = folder("bad");
    install(&current, "catppuccin-latte", Some(LATTE));
    let latte = colors_of("bad-latte", LATTE);
    cx.update(|cx| follow_omarchy(current.clone(), cx)).unwrap();

    install(&current, "broken", Some("mode = \"dark\"\n"));
    idle(cx);
    assert_eq!(wearing(cx), (Mode::Light, latte));

    install(&current, "tokyo-night", Some(NIGHT));
    let night = colors_of("bad-night", NIGHT);
    until(cx, "tokyo night is worn", |cx| {
        cx.theme().palette() == night
    });
    cx.update(stop_omarchy);
    fs::remove_dir_all(&current).unwrap();
}

#[gpui::test]
fn a_theme_without_colors_brings_elys_palettes_back(cx: &mut TestAppContext) {
    start(cx);
    let current = folder("bare");
    install(&current, "tokyo-night", Some(NIGHT));
    cx.update(|cx| follow_omarchy(current.clone(), cx)).unwrap();
    assert_eq!(wearing(cx).0, Mode::Dark);

    install(&current, "plain", None);
    until(cx, "Ely's dark palette shows", |cx| {
        cx.theme().palette() == Palette::dark(false)
    });
    cx.update(stop_omarchy);
    fs::remove_dir_all(&current).unwrap();
}

#[gpui::test]
fn stopping_brings_elys_palettes_back_and_a_later_switch_changes_nothing(cx: &mut TestAppContext) {
    start(cx);
    let current = folder("stop");
    install(&current, "catppuccin-latte", Some(LATTE));
    cx.update(|cx| follow_omarchy(current.clone(), cx)).unwrap();
    cx.update(stop_omarchy);
    assert_eq!(wearing(cx), (Mode::Light, Palette::light(false)));

    install(&current, "tokyo-night", Some(NIGHT));
    idle(cx);
    assert_eq!(wearing(cx), (Mode::Light, Palette::light(false)));
    cx.update(stop_omarchy);
    fs::remove_dir_all(&current).unwrap();
}

/// Lets the watcher's thread deliver, then moves the executor's clock by `ms`.
fn pump(cx: &mut TestAppContext, ms: u64) {
    std::thread::sleep(Duration::from_millis(100));
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_millis(ms));
    cx.run_until_parked();
}

/// Each step 200 ms apart, under the settle: a reload in between would find no theme.
#[gpui::test]
fn a_slow_swap_shows_no_palette_between_the_two_themes(cx: &mut TestAppContext) {
    start(cx);
    let current = folder("slow");
    install(&current, "tokyo-night", Some(NIGHT));
    let (night, latte) = (
        colors_of("slow-night", NIGHT),
        colors_of("slow-latte", LATTE),
    );
    cx.update(|cx| follow_omarchy(current.clone(), cx)).unwrap();
    fs::create_dir(current.join("next-theme")).unwrap();
    fs::write(current.join("next-theme/colors.toml"), LATTE).unwrap();
    pump(cx, 200);
    fs::remove_dir_all(current.join("theme")).unwrap();
    pump(cx, 200);
    assert_eq!(wearing(cx), (Mode::Dark, night), "nothing between");
    fs::rename(current.join("next-theme"), current.join("theme")).unwrap();
    fs::write(current.join("theme.name"), "catppuccin-latte\n").unwrap();
    until(cx, "latte is worn", |cx| cx.theme().palette() == latte);
    cx.update(stop_omarchy);
    fs::remove_dir_all(&current).unwrap();
}

#[gpui::test]
fn following_puts_the_owners_palettes_aside_and_stopping_brings_elys(cx: &mut TestAppContext) {
    start(cx);
    let current = folder("owner");
    let mut owners = Palette::dark(false);
    owners.accent = gpui::rgb(0xff0000).into();
    cx.update(|cx| {
        Theme::set_mode_now(Mode::Dark, cx);
        Theme::set_palette(Mode::Dark, Some(owners.clone()), cx);
        follow_omarchy(current.clone(), cx).unwrap();
    });
    assert_eq!(wearing(cx).1, Palette::dark(false), "no colors yet: Ely's");
    cx.update(|cx| Theme::set_palette(Mode::Dark, Some(owners), cx));
    cx.update(stop_omarchy);
    assert_eq!(wearing(cx).1, Palette::dark(false));
    fs::remove_dir_all(&current).unwrap();
}

/// Inotify reports reads, the watcher's own too, so a read that counted would read again.
#[gpui::test]
fn reading_the_theme_is_no_switch(cx: &mut TestAppContext) {
    start(cx);
    let current = folder("reads");
    install(&current, "catppuccin-latte", Some(LATTE));
    cx.update(|cx| follow_omarchy(current.clone(), cx)).unwrap();
    idle(cx);
    let reads = |cx: &mut TestAppContext| cx.update(|cx| cx.global::<Following>().reads);
    let before = reads(cx);
    fs::read_to_string(current.join("theme/colors.toml")).unwrap();
    fs::read_to_string(current.join("theme.name")).unwrap();
    idle(cx);
    assert_eq!(reads(cx), before);
    cx.update(stop_omarchy);
    fs::remove_dir_all(&current).unwrap();
}

#[gpui::test]
fn following_twice_or_a_missing_folder_fails_naming_the_folder(cx: &mut TestAppContext) {
    start(cx);
    let current = folder("twice");
    cx.update(|cx| follow_omarchy(current.clone(), cx)).unwrap();
    let error = cx
        .update(|cx| follow_omarchy(current.clone(), cx))
        .unwrap_err();
    assert!(format!("{error:#}").contains("already"), "{error:#}");
    cx.update(stop_omarchy);

    let gone = current.join("gone");
    let error = cx
        .update(|cx| follow_omarchy(gone.clone(), cx))
        .unwrap_err();
    assert!(format!("{error:#}").contains("gone"), "{error:#}");
    fs::remove_dir_all(&current).unwrap();
}
