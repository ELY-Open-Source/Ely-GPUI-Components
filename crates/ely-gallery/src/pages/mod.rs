mod account;
mod agent;
mod buttons;
mod calendar;
mod canvas;
mod charts;
mod chat;
mod collab;
mod dashboard;
mod data;
mod debug;
mod devtools;
mod documents;
mod editor;
mod feedback;
mod files;
mod finance;
mod forms;
mod generative;
mod git;
mod i18n;
mod interaction;
mod layout;
mod lists;
mod mail;
mod maps;
mod media;
mod menus;
mod messaging;
mod misc;
mod motion;
mod navigation;
mod onboarding;
mod overlays;
mod primitives;
mod project;
mod rendering;
mod settings;
mod shell;
mod tables;
mod terminal;
mod theme;
mod tooling;
mod typography;

use gpui::{AnyElement, App, Window};

use crate::step::Step;

pub struct Page {
    pub number: u8,
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub render: fn(&mut Window, &mut App) -> AnyElement,
    /// Interactions shot after the static pages.
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    pub script: &'static [Step],
}

pub const ALL: &[Page] = &[
    primitives::PAGE,
    typography::PAGE,
    layout::PAGE,
    shell::PAGE,
    buttons::PAGE,
    forms::PAGE,
    navigation::PAGE,
    menus::PAGE,
    overlays::PAGE,
    feedback::PAGE,
    motion::PAGE,
    data::PAGE,
    lists::PAGE,
    tables::PAGE,
    charts::PAGE,
    finance::PAGE,
    editor::PAGE,
    terminal::PAGE,
    git::PAGE,
    debug::PAGE,
    documents::PAGE,
    collab::PAGE,
    chat::PAGE,
    agent::PAGE,
    generative::PAGE,
    media::PAGE,
    files::PAGE,
    messaging::PAGE,
    mail::PAGE,
    calendar::PAGE,
    project::PAGE,
    canvas::PAGE,
    devtools::PAGE,
    dashboard::PAGE,
    settings::PAGE,
    account::PAGE,
    onboarding::PAGE,
    interaction::PAGE,
    theme::PAGE,
    i18n::PAGE,
    maps::PAGE,
    misc::PAGE,
    tooling::PAGE,
    rendering::PAGE,
];

pub use i18n::install as install_i18n;
pub use interaction::bind_keys;
pub use shell::{open_about, open_managed};

pub fn find(slug: &str) -> Option<usize> {
    ALL.iter().position(|page| page.slug == slug)
}
