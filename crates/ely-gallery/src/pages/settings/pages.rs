use ely_gpui_component::{
    forms::{Choice, Combobox, FontPicker},
    settings::{
        AccentColorPicker, Changed, DensitySelector, DeveloperModeToggle, FeatureFlags,
        FontSizeControl, ImportExportSettings, KeyboardShortcutsList, NotificationSettings,
        PrivacySettings, ProxySettings, ResetToDefault, SettingsRow, SettingsSection, Shortcut,
        StartupSettings, StorageSettings, ThemeSelector,
    },
    theme::{ActiveTheme, HUE_NAMES},
};
use gpui::{AnyElement, App, Entity, IntoElement, ParentElement, Styled, div, px};

use super::window::{Fields, Prefs, SIZE};
use crate::ui::change;

/// The open section's page.
pub(super) fn page(prefs: &Entity<Prefs>, fields: &Fields, section: &str, cx: &App) -> AnyElement {
    match section {
        "appearance" => appearance(prefs, fields, cx),
        "keyboard" => keyboard(prefs, fields, cx),
        "network" => network(prefs, cx),
        "privacy" => privacy(prefs, cx),
        "notifications" => notifications(prefs, cx),
        "startup" => startup(prefs, cx),
        "storage" => storage(prefs, cx),
        "advanced" => advanced(prefs, fields, cx),
        other => panic!("settings demo: no page {other}"),
    }
}

fn appearance(prefs: &Entity<Prefs>, fields: &Fields, cx: &App) -> AnyElement {
    let now = prefs.read(cx);
    let presets = HUE_NAMES
        .iter()
        .take(6)
        .enumerate()
        .map(|(ix, name)| (*name, cx.theme().colors.hue(ix, "accent presets")));
    let [theme, accent, size, density, font] = [(); 5].map(|_| prefs.clone());
    SettingsSection::new("Appearance")
        .description("How the app looks.")
        .row(SettingsRow::new("Theme").control(ThemeSelector::new(
            "settings-theme",
            now.appearance,
            move |next, _, cx| change(&theme, cx, |prefs| prefs.appearance = next),
        )))
        .row(
            SettingsRow::new("Accent color")
                .description("Buttons, links and what is chosen.")
                .control(AccentColorPicker::new(
                    "settings-accent",
                    now.accent,
                    presets,
                    move |next, _, cx| change(&accent, cx, |prefs| prefs.accent = next),
                )),
        )
        .row(SettingsRow::new("Font size").control(FontSizeControl::new(
            "settings-size",
            now.size,
            SIZE,
            (11.0, 20.0),
            move |next, _, cx| change(&size, cx, |prefs| prefs.size = next),
        )))
        .row(
            SettingsRow::new("Density")
                .description("How close things sit.")
                .control(DensitySelector::new(
                    "settings-density",
                    now.density,
                    move |next, _, cx| change(&density, cx, |prefs| prefs.density = next),
                )),
        )
        .row(
            SettingsRow::new("Font")
                .description("FontFamilySelect → forms::FontPicker")
                .control(
                    div().w(px(220.)).child(
                        FontPicker::new("settings-font", &fields.font)
                            .selected(now.family.clone())
                            .on_change(move |next, _, cx| {
                                change(&font, cx, |prefs| prefs.family = next.clone())
                            }),
                    ),
                ),
        )
        .into_any_element()
}

fn shortcut(group: &str, name: &str, keys: &str) -> Shortcut {
    Shortcut {
        group: group.to_string().into(),
        name: name.to_string().into(),
        keys: keys.to_string().into(),
    }
}

fn keyboard(prefs: &Entity<Prefs>, fields: &Fields, cx: &App) -> AnyElement {
    let language = prefs.clone();
    SettingsSection::new("Keyboard and language")
        .description("ShortcutEditor → editor::KeybindingsEditor")
        .row(
            SettingsRow::new("Language")
                .description("LanguageSelector → forms::Combobox::languages")
                .control(
                    div().w(px(220.)).child(
                        Combobox::languages("settings-language", &fields.language)
                            .selected(prefs.read(cx).language.clone())
                            .on_change(move |next, _, cx| {
                                change(&language, cx, |prefs| prefs.language = next.clone())
                            }),
                    ),
                ),
        )
        .row(KeyboardShortcutsList::new(
            [
                shortcut("General", "Command palette", "cmd-shift-p"),
                shortcut("General", "Settings", "cmd-,"),
                shortcut("Editing", "Undo", "cmd-z"),
                shortcut("Editing", "Find", "cmd-f"),
                shortcut("Window", "New window", "cmd-shift-n"),
            ],
            &fields.shortcuts,
        ))
        .into_any_element()
}

fn network(prefs: &Entity<Prefs>, cx: &App) -> AnyElement {
    let proxy = prefs.clone();
    SettingsSection::new("Network")
        .row(ProxySettings::new(
            "settings-proxy",
            prefs.read(cx).proxy.clone(),
            move |next, _, cx| change(&proxy, cx, |prefs| prefs.proxy = next),
        ))
        .into_any_element()
}

fn privacy(prefs: &Entity<Prefs>, cx: &App) -> AnyElement {
    let privacy = prefs.clone();
    PrivacySettings::new(
        "settings-privacy",
        prefs.read(cx).privacy,
        move |next, _, cx| change(&privacy, cx, |prefs| prefs.privacy = next),
    )
    .into_any_element()
}

fn notifications(prefs: &Entity<Prefs>, cx: &App) -> AnyElement {
    let notices = prefs.clone();
    NotificationSettings::new(
        "settings-notices",
        prefs.read(cx).notices.clone(),
        move |next, _, cx| change(&notices, cx, |prefs| prefs.notices = next),
    )
    .into_any_element()
}

fn startup(prefs: &Entity<Prefs>, cx: &App) -> AnyElement {
    let startup = prefs.clone();
    let pages = [
        Choice::new("last", "Where I left off"),
        Choice::new("home", "The home page"),
        Choice::new("blank", "A blank window"),
    ];
    StartupSettings::new(
        "settings-startup",
        prefs.read(cx).startup.clone(),
        pages,
        move |next, _, cx| change(&startup, cx, |prefs| prefs.startup = next),
    )
    .into_any_element()
}

fn storage(prefs: &Entity<Prefs>, cx: &App) -> AnyElement {
    let now = prefs.read(cx);
    let [cleared, reset, all] = [(); 3].map(|_| prefs.clone());
    let changed = (now.size != SIZE).then(|| Changed {
        key: "size".into(),
        name: "Font size".into(),
        now: format!("{} pt", now.size).into(),
        default: format!("{SIZE} pt").into(),
    });
    div()
        .flex()
        .flex_col()
        .gap_8()
        .child(
            SettingsSection::new("Storage")
                .description("What the app keeps on this device.")
                .row(StorageSettings::new(
                    "settings-storage",
                    now.stores.clone(),
                    move |key, _, cx| {
                        change(&cleared, cx, |prefs| {
                            prefs
                                .stores
                                .iter_mut()
                                .filter(|store| store.key == *key)
                                .for_each(|store| store.bytes = 0)
                        })
                    },
                )),
        )
        .child(
            SettingsSection::new("Import and export").row(ImportExportSettings::new(
                "settings-files",
                |_, _| log::info!("gallery: settings exported"),
                |path, _, _| log::info!("gallery: settings imported from {}", path.display()),
            )),
        )
        .child(
            SettingsSection::new("Defaults")
                .description("A larger font size shows here.")
                .row(ResetToDefault::new(
                    "settings-reset",
                    changed,
                    move |_, _, cx| change(&reset, cx, |prefs| prefs.size = SIZE),
                    move |_, cx| change(&all, cx, |prefs| prefs.size = SIZE),
                )),
        )
        .into_any_element()
}

fn advanced(prefs: &Entity<Prefs>, fields: &Fields, cx: &App) -> AnyElement {
    let now = prefs.read(cx);
    let [toggled, developer] = [(); 2].map(|_| prefs.clone());
    div()
        .flex()
        .flex_col()
        .gap_8()
        .child(
            SettingsSection::new("Features")
                .description("AdvancedSettings / FeatureFlags")
                .row(FeatureFlags::new(
                    "settings-flags",
                    now.flags.clone(),
                    &fields.flags,
                    move |key, on, _, cx| {
                        change(&toggled, cx, |prefs| {
                            prefs
                                .flags
                                .iter_mut()
                                .filter(|flag| flag.key == *key)
                                .for_each(|flag| flag.on = on)
                        })
                    },
                )),
        )
        .child(DeveloperModeToggle::new(
            "settings-developer",
            now.developer,
            move |on, _, cx| change(&developer, cx, |prefs| prefs.developer = on),
        ))
        .into_any_element()
}
