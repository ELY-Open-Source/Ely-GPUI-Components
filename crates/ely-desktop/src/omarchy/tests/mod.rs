#[cfg(all(not(target_family = "wasm"), feature = "test-support"))]
mod follow;

use std::{fs, path::PathBuf};

use gpui::Hsla;

use super::{OmarchyTheme, colors::hex, palette::wash, parse, read_omarchy_theme};
use crate::theme::{Mode, Palette, contrast, over};

const THEMES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/omarchy/themes");
const TOKYO_NIGHT: &str = include_str!("../themes/tokyo-night.toml");
const LATTE: &str = include_str!("../themes/catppuccin-latte.toml");
/// Tokyo Night as `omarchy-theme-colors-from-alacritty` writes it: ANSI names alone.
const GENERATED: &str = "accent = \"#7aa2f7\"\nselection = \"#292e42\"\nbackground = \"#1a1b26\"\nforeground = \"#a9b1d6\"\ncolor0 = \"#1a1b26\"\ncolor1 = \"#f7768e\"\ncolor2 = \"#9ece6a\"\ncolor3 = \"#e0af68\"\ncolor4 = \"#7aa2f7\"\ncolor5 = \"#ad8ee6\"\ncolor6 = \"#449dab\"\ncolor7 = \"#a9b1d6\"\ncolor8 = \"#414868\"\ncolor9 = \"#ff7a93\"\ncolor10 = \"#b9f27c\"\ncolor11 = \"#ff9e64\"\ncolor12 = \"#7da6ff\"\ncolor13 = \"#bb9af7\"\ncolor14 = \"#0db9d7\"\ncolor15 = \"#c0caf5\"\n";

fn bundled() -> Vec<(String, String)> {
    let mut themes: Vec<_> = fs::read_dir(THEMES)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .map(|path| {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            (name, fs::read_to_string(path).unwrap())
        })
        .collect();
    themes.sort();
    themes
}

fn value_of(text: &str, key: &str) -> Option<Hsla> {
    let line = text
        .lines()
        .find(|line| line.starts_with(&format!("{key} =")))?;
    Some(hex(line.split('"').nth(1).unwrap()).unwrap())
}

/// A color as `#rrggbb`, in whole bytes.
fn shown(color: Hsla) -> String {
    crate::forms::hex(color.into())
}

fn rgb(hex: u32) -> Hsla {
    gpui::rgb(hex).into()
}

/// Ely's contrast rules, written out apart from `devtools::audit`.
fn assert_contract(name: &str, theme: &OmarchyTheme) {
    let p = &theme.colors;
    let surfaces = [p.bg, p.surface, p.overlay, p.sunken, p.hover, p.active];
    let on = |token: &str, color: Hsla, grounds: &[Hsla], need: f32| {
        for (ix, ground) in grounds.iter().enumerate() {
            let ratio = contrast(color, *ground);
            assert!(
                ratio >= need,
                "{name}: {token} on ground {ix} reads {ratio:.2}"
            );
        }
    };
    for (token, color) in [("fg", p.fg), ("fg_muted", p.fg_muted), ("link", p.link)] {
        on(token, color, &surfaces, 4.5);
    }
    for token in crate::theme::Syntax::NAMES {
        on(token, p.syntax.token(token), &surfaces, 4.5);
    }
    for (token, color, subtle) in [
        ("success", p.success, p.success_subtle),
        ("warning", p.warning, p.warning_subtle),
        ("danger", p.danger, p.danger_subtle),
        ("info", p.info, p.info_subtle),
    ] {
        on(token, color, &[p.bg, subtle], 4.5);
    }
    on("on_accent", p.on_accent, &[p.accent, p.accent_hover], 4.5);
    on("tooltip_fg", p.tooltip_fg, &[p.tooltip_bg], 4.5);
    on("focus", p.focus, &[p.bg, p.surface], 3.0);
    for (ix, hue) in p.chart.iter().enumerate() {
        on(&format!("chart {ix}"), *hue, &[p.bg], 3.0);
    }
}

#[test]
fn every_bundled_theme_keeps_its_colors_and_reads_on_every_surface() {
    let themes = bundled();
    assert_eq!(themes.len(), 22);
    for (name, text) in themes {
        let theme =
            parse(&text, Some(name.clone()), false).unwrap_or_else(|e| panic!("{name}: {e:#}"));
        let p = &theme.colors;
        let given = |key: &str| value_of(&text, key).unwrap();
        let mode = text.contains("mode = \"light\"");
        assert_eq!(theme.mode == Mode::Light, mode, "{name}");
        assert!(theme.missing.is_empty(), "{name}: {:?}", theme.missing);
        assert_eq!(p.bg, given("background"), "{name}");
        assert_eq!(p.accent, given("accent"), "{name}");
        assert_eq!(
            shown(over(p.selection, p.bg)),
            shown(given("selection")),
            "{name}"
        );
        let red = given("red");
        if contrast(red, p.bg) >= 4.5 && contrast(red, p.danger_subtle) >= 4.5 {
            assert_eq!(p.danger, red, "{name}: a red that reads is kept");
        }
        assert_contract(&name, &theme);
    }
}

/// Ghostty's palette from `default/themed/ghostty.conf.tpl`.
#[test]
fn the_terminal_colors_are_the_ones_omarchy_gives_ghostty() {
    for (name, text) in bundled() {
        let ansi = parse(&text, None, false).unwrap().colors.ansi;
        let keys = [
            "background",
            "red",
            "green",
            "yellow",
            "blue",
            "magenta",
            "cyan",
            "foreground",
            "muted",
            "bright_red",
            "bright_green",
            "bright_yellow",
            "bright_blue",
            "bright_magenta",
            "bright_cyan",
            "bright_foreground",
        ];
        for (ix, key) in keys.iter().enumerate() {
            if let Some(color) = value_of(&text, key) {
                assert_eq!(ansi[ix], color, "{name}: palette {ix} is {key}");
            }
        }
    }
}

#[test]
fn a_theme_whose_text_already_reads_keeps_its_foreground() {
    let theme = parse(TOKYO_NIGHT, None, false).unwrap();
    assert_eq!(theme.colors.fg, rgb(0xa9b1d6));
}

#[test]
fn a_generated_theme_with_ansi_names_alone_gets_its_hues_through_omarchys_aliases() {
    let theme = parse(GENERATED, None, false).unwrap();
    let p = &theme.colors;
    assert_eq!(theme.mode, Mode::Dark);
    assert!(theme.missing.is_empty(), "{:?}", theme.missing);
    assert_eq!(p.danger, rgb(0xf7768e));
    assert_eq!(p.success, rgb(0x9ece6a));
    assert_eq!(p.chart[4], rgb(0xad8ee6));
    assert_eq!(p.ansi[8], rgb(0x414868), "color8 stands for muted");
    assert_eq!(p.ansi[9], rgb(0xff7a93));
    assert_eq!(p.ansi[15], rgb(0xc0caf5));
    assert_contract("generated", &theme);
}

#[test]
fn bright_hues_and_orange_follow_omarchys_derivations_in_whole_bytes() {
    let bare = "background = \"#101010\"\nforeground = \"#e0e0e0\"\nred = \"#c04040\"\nyellow = \"#c0a040\"\n";
    let p = parse(bare, None, false).unwrap().colors;
    assert_eq!(p.ansi[9], rgb(0xcd6666), "mix_color's #cd6666");
    assert_eq!(p.ansi[8], rgb(0xe0e0e0), "muted falls to the foreground");
    assert_eq!(p.chart[6], rgb(0xc0a040), "orange falls to yellow");
}

#[test]
fn a_theme_that_lacks_a_hue_is_told_apart_from_one_that_has_it() {
    let no_red = TOKYO_NIGHT.replace("red = \"#f7768e\"", "");
    let theme = parse(&no_red, None, false).unwrap();
    assert_eq!(theme.missing, ["red"]);
    assert_eq!(theme.colors.danger, Palette::dark(false).danger);

    let bare = parse(
        "background = \"#101010\"\nforeground = \"#e0e0e0\"\n",
        None,
        false,
    )
    .unwrap();
    assert_eq!(
        bare.missing,
        [
            "red", "green", "yellow", "blue", "magenta", "cyan", "accent"
        ]
    );
}

#[test]
fn the_reader_takes_omarchys_loose_syntax_and_skips_what_omarchy_skips() {
    let loose = "# a comment\n\"background\" = '#1a1b26'\nforeground=#a9b1d6\n  accent   =   \"#7aa2f7\"  # note\nmode = \"dark\"\nodd key! = \"#ffffff\"\nname = \"a;b\"\n[table]\n";
    let p = parse(loose, None, false).unwrap().colors;
    assert_eq!(p.bg, rgb(0x1a1b26));
    assert_eq!(p.accent, rgb(0x7aa2f7));
    let unended = "background = \"#101010\"\nforeground = \"#e0e0e0\"\nmode = \"light\"";
    assert_eq!(
        parse(unended, None, false).unwrap().mode,
        Mode::Dark,
        "bash's read drops a last line without its newline"
    );
    assert_eq!(p.fg, rgb(0xa9b1d6));
}

#[test]
fn mode_comes_from_mode_then_theme_type_then_the_light_file_then_brightness() {
    let colors = |extra: &str, bg: &str| {
        format!("{extra}\nbackground = \"{bg}\"\nforeground = \"#808080\"\n")
    };
    let mode = |text: String, light_file: bool| parse(&text, None, light_file).unwrap().mode;
    assert_eq!(
        mode(colors("mode = \"light\"", "#000000"), false),
        Mode::Light
    );
    assert_eq!(
        mode(colors("theme_type = \"light\"", "#000000"), false),
        Mode::Light
    );
    assert_eq!(
        mode(
            colors("mode = \"dark\"\ntheme_type = \"light\"", "#ffffff"),
            false
        ),
        Mode::Dark
    );
    assert_eq!(mode(colors("", "#000000"), true), Mode::Light);
    assert_eq!(mode(colors("mode = \"dark\"", "#ffffff"), true), Mode::Dark);
    assert_eq!(
        mode(colors("", "#7f8080"), false),
        Mode::Light,
        "a sum of 383"
    );
    assert_eq!(
        mode(colors("", "#7f7f80"), false),
        Mode::Dark,
        "a sum of 382"
    );
}

#[test]
fn the_cause_is_named_for_a_missing_color_a_bad_color_a_bad_mode_and_text_that_cannot_read() {
    let say = |text: &str| format!("{:#}", parse(text, None, false).unwrap_err());
    let without = |key: &str| {
        TOKYO_NIGHT
            .lines()
            .filter(|line| !line.starts_with(key))
            .map(|line| format!("{line}\n"))
            .collect::<String>()
    };
    assert!(say(&without("foreground")).contains("\"foreground\" is missing"));
    assert!(say(&without("background")).contains("\"background\" is missing"));
    assert!(
        say(&TOKYO_NIGHT.replace("\"#7aa2f7\"\nselection", "\"blue\"\nselection"))
            .contains("\"accent\"")
    );
    assert!(say(&TOKYO_NIGHT.replace("#f7768e", "#+12345")).contains("not a hex color"));
    assert!(say(&TOKYO_NIGHT.replace("#f7768e", "#f77")).contains("not #rrggbb"));
    assert!(
        say(&TOKYO_NIGHT.replace("mode = \"dark\"", "mode = \"dim\""))
            .contains("not dark or light")
    );
    assert!(say("not toml at all").contains("\"background\" is missing"));
    assert!(say("background = \"#777777\"\nforeground = \"#000000\"\n").contains("needs 4.5:1"));
}

#[test]
fn a_hue_that_reads_only_toward_the_other_extreme_is_lifted_there() {
    let gray = "background = \"#767676\"\nforeground = \"#000000\"\ngreen = \"#00ff00\"\n";
    let p = parse(gray, None, false)
        .unwrap_or_else(|e| panic!("{e:#}"))
        .colors;
    assert!(
        contrast(p.success, p.success_subtle) >= 4.5,
        "{:?}",
        p.success
    );
    assert!(contrast(p.success, p.bg) >= 4.5);
}

#[test]
fn a_selection_becomes_the_clearest_wash_that_shows_it_over_the_page() {
    let (page, picked) = (rgb(0x1a1b26), rgb(0x292e42));
    let washed = wash(picked, page);
    assert!(washed.a < 0.2, "{washed:?}");
    assert_eq!(shown(over(washed, page)), "#292e42");
    let latte = wash(rgb(0xccd0da), rgb(0xeff1f5));
    assert_eq!(shown(over(latte, rgb(0xeff1f5))), "#ccd0da");
    assert_eq!(
        wash(page, page).a,
        0.0,
        "a selection the page's color shows nothing"
    );
}

/// A fresh folder under the system's temporary one.
fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ely-omarchy-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_folder_gives_the_theme_its_name_and_light_marker_and_none_without_colors() {
    let current = folder("read");
    assert!(read_omarchy_theme(&current).unwrap().is_none());

    let theme_dir = current.join("theme");
    fs::create_dir(&theme_dir).unwrap();
    fs::write(theme_dir.join("colors.toml"), LATTE).unwrap();
    assert_eq!(read_omarchy_theme(&current).unwrap().unwrap().name, None);
    fs::write(current.join("theme.name"), "catppuccin-latte\n").unwrap();
    let theme = read_omarchy_theme(&current).unwrap().unwrap();
    assert_eq!(theme.name.as_deref(), Some("catppuccin-latte"));
    assert_eq!(theme.mode, Mode::Light);

    let plain = "background = \"#000000\"\nforeground = \"#ffffff\"\n";
    fs::write(theme_dir.join("colors.toml"), plain).unwrap();
    assert_eq!(
        read_omarchy_theme(&current).unwrap().unwrap().mode,
        Mode::Dark
    );
    fs::create_dir(theme_dir.join("light.mode")).unwrap();
    let marked = read_omarchy_theme(&current).unwrap().unwrap();
    assert_eq!(marked.mode, Mode::Dark, "Omarchy's -f takes a file alone");
    fs::remove_dir(theme_dir.join("light.mode")).unwrap();
    fs::write(theme_dir.join("light.mode"), "").unwrap();
    assert_eq!(
        read_omarchy_theme(&current).unwrap().unwrap().mode,
        Mode::Light
    );

    fs::write(theme_dir.join("colors.toml"), "mode = \"dark\"").unwrap();
    let error = format!("{:#}", read_omarchy_theme(&current).unwrap_err());
    assert!(
        error.contains("colors.toml") && error.contains("\"background\" is missing"),
        "{error}"
    );
    fs::remove_dir_all(&current).unwrap();
}

#[test]
fn a_name_that_cannot_be_read_fails_naming_its_file() {
    let current = folder("name");
    fs::create_dir(current.join("theme")).unwrap();
    fs::write(current.join("theme/colors.toml"), LATTE).unwrap();
    fs::create_dir(current.join("theme.name")).unwrap();
    let error = format!("{:#}", read_omarchy_theme(&current).unwrap_err());
    assert!(error.contains("theme.name"), "{error}");
    fs::remove_dir_all(&current).unwrap();
}
