use ely_gpui_component::{
    forms::TextInput,
    primitives::IconName,
    settings::{
        Appearance, Flag, Notices, Privacy, Proxy, SettingEntry, SettingsLayout, SettingsSearch,
        Stage, Startup, Store,
    },
    theme::{ActiveTheme, Density},
};
use gpui::{App, Entity, Hsla, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::pages;
use crate::{
    probe::probe,
    ui::{change, keep, section},
};

pub(super) const SECTIONS: [(&str, &str, IconName); 8] = [
    ("appearance", "Appearance", IconName::Palette),
    ("keyboard", "Keyboard", IconName::Keyboard),
    ("network", "Network", IconName::Globe),
    ("privacy", "Privacy", IconName::Shield),
    ("notifications", "Notifications", IconName::Bell),
    ("startup", "Startup", IconName::Power),
    ("storage", "Storage", IconName::HardDrive),
    ("advanced", "Advanced", IconName::Wrench),
];

/// Everything the settings window holds.
pub(super) struct Prefs {
    pub section: SharedString,
    pub appearance: Appearance,
    pub accent: Hsla,
    pub size: f32,
    pub density: Density,
    pub family: SharedString,
    pub language: SharedString,
    pub proxy: Proxy,
    pub privacy: Privacy,
    pub notices: Notices,
    pub startup: Startup,
    pub stores: Vec<Store>,
    pub flags: Vec<Flag>,
    pub developer: bool,
}

pub(super) const SIZE: f32 = 14.0;

fn flag(key: &str, name: &str, description: &str, stage: Stage, on: bool) -> Flag {
    Flag {
        key: key.to_string().into(),
        name: name.to_string().into(),
        description: description.to_string().into(),
        stage,
        on,
    }
}

fn pair(kind: &str, channel: &str) -> (SharedString, SharedString) {
    (kind.to_string().into(), channel.to_string().into())
}

impl Prefs {
    fn new(accent: Hsla) -> Self {
        Self {
            section: "appearance".into(),
            appearance: Appearance::System,
            accent,
            size: SIZE,
            density: Density::Standard,
            family: "Inter".into(),
            language: "en".into(),
            proxy: Proxy::System,
            privacy: Privacy {
                telemetry: true,
                crash_reports: true,
                personalization: false,
                history: true,
            },
            notices: Notices {
                channels: vec!["App".into(), "Email".into(), "Push".into()],
                kinds: vec![
                    pair("mentions", "Mentions"),
                    pair("replies", "Replies"),
                    pair("digest", "Weekly digest"),
                ],
                on: vec![
                    pair("mentions", "App"),
                    pair("mentions", "Push"),
                    pair("replies", "App"),
                    pair("digest", "Email"),
                ],
                quiet: false,
            },
            startup: Startup {
                at_login: false,
                restore: true,
                opens: "last".into(),
                updates: true,
            },
            stores: vec![
                Store {
                    key: "cache".into(),
                    name: "Cache".into(),
                    bytes: 312 << 20,
                    clearable: true,
                },
                Store {
                    key: "logs".into(),
                    name: "Logs".into(),
                    bytes: 48 << 20,
                    clearable: true,
                },
                Store {
                    key: "documents".into(),
                    name: "Documents".into(),
                    bytes: 1_240 << 20,
                    clearable: false,
                },
            ],
            flags: vec![
                flag(
                    "tabs",
                    "Vertical tabs",
                    "Tabs down the side of the window.",
                    Stage::Beta,
                    false,
                ),
                flag(
                    "sync",
                    "Settings sync",
                    "The same settings on every device.",
                    Stage::Stable,
                    true,
                ),
                flag(
                    "suggest",
                    "Inline suggestions",
                    "The rest of a line, offered as you type.",
                    Stage::Experimental,
                    false,
                ),
            ],
            developer: false,
        }
    }
}

/// The window's text fields, each kept across frames.
pub(super) struct Fields {
    pub font: Entity<TextInput>,
    pub language: Entity<TextInput>,
    pub shortcuts: Entity<TextInput>,
    pub flags: Entity<TextInput>,
}

fn field(
    key: &'static str,
    hint: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TextInput> {
    window.use_keyed_state(key, cx, move |window, cx| {
        TextInput::new(window, cx).placeholder(hint)
    })
}

fn entry(section: &str, key: &str, name: &str, words: &[&str]) -> SettingEntry {
    let (_, title, _) = SECTIONS
        .iter()
        .find(|(each, _, _)| *each == section)
        .expect("a settings section");
    SettingEntry {
        key: key.to_string().into(),
        section: (*title).into(),
        name: name.to_string().into(),
        words: words
            .iter()
            .map(|word| SharedString::from(word.to_string()))
            .collect(),
    }
}

/// The section each searchable setting lives in, by the setting's key.
fn home(key: &str) -> &'static str {
    match key {
        "theme" | "accent" | "size" | "density" | "font" => "appearance",
        "language" | "shortcuts" => "keyboard",
        "proxy" => "network",
        "telemetry" => "privacy",
        "quiet" => "notifications",
        "login" => "startup",
        "cache" => "storage",
        "developer" => "advanced",
        other => panic!("settings demo: no home for {other}"),
    }
}

pub fn settings(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let accent = cx.theme().colors.accent;
    let prefs = keep("settings-prefs", move || Prefs::new(accent), window, cx);
    let search = field("settings-search", "Search settings", window, cx);
    let fields = Fields {
        font: field("settings-font", "Font", window, cx),
        language: field("settings-language", "Language", window, cx),
        shortcuts: field("settings-shortcuts", "Find a shortcut", window, cx),
        flags: field("settings-flags", "Find a feature", window, cx),
    };
    let entries = [
        entry("appearance", "theme", "Theme", &["dark", "light", "mode"]),
        entry("appearance", "accent", "Accent color", &["color", "hue"]),
        entry("appearance", "size", "Font size", &["text", "zoom"]),
        entry("appearance", "density", "Density", &["compact", "spacing"]),
        entry("appearance", "font", "Font", &["typeface", "family"]),
        entry(
            "keyboard",
            "language",
            "Language",
            &["locale", "translation"],
        ),
        entry(
            "keyboard",
            "shortcuts",
            "Keyboard shortcuts",
            &["keys", "bindings"],
        ),
        entry("network", "proxy", "Proxy", &["http", "socks"]),
        entry(
            "privacy",
            "telemetry",
            "Usage data",
            &["telemetry", "analytics"],
        ),
        entry(
            "notifications",
            "quiet",
            "Hold notices",
            &["do not disturb", "quiet"],
        ),
        entry("startup", "login", "Open at login", &["launch", "boot"]),
        entry("storage", "cache", "Clear cache", &["disk", "space"]),
        entry(
            "advanced",
            "developer",
            "Developer mode",
            &["debug", "inspector"],
        ),
    ];
    let selected = prefs.read(cx).section.clone();
    let page = pages::page(&prefs, &fields, &selected, cx);
    let [found, picked] = [(); 2].map(|_| prefs.clone());
    section(
        "SettingsLayout · SettingsSection · SettingsRow · SettingsSearch",
        "A settings window: sections down the side and the open one beside them, the list going above in a narrow window. The search finds a setting by its name or the words it answers to, and Enter opens its section.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(div().w(px(360.)).child(SettingsSearch::new("settings-search", entries, &search, 
                move |key, _, cx| change(&found, cx, |prefs| prefs.section = home(key).into()),
            )))
            .child(probe(
                "settings-layout",
                div().w(px(880.)).child(
                    SettingsLayout::new("settings-layout", SECTIONS, selected, move |key, _, cx| change(&picked, cx, |prefs| prefs.section = key.clone()))
                        .page(page),
                ),
            )),
    )
}
