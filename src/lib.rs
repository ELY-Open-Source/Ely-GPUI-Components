pub mod account;
pub mod agent;
pub mod calendar;
pub mod canvas;
pub mod charts;
pub mod chat;
pub mod collab;
pub mod dashboard;
pub mod debug;
pub mod devtools;
pub mod documents;
pub mod editor;
pub mod files;
pub mod finance;
pub mod generative;
pub mod git;
pub mod interaction;
pub mod mail;
pub mod maps;
pub mod media;
pub mod messaging;
pub mod misc;
pub mod onboarding;
pub mod project;
pub mod rendering;
pub mod settings;
pub mod terminal;
pub mod tooling;

#[cfg(test)]
mod shipped;

pub use ely_core::{
    Assets, Before, buttons, data_display, feedback, forms, i18n, layout, lists, menus, motion,
    navigation, overlays, primitives, shell, tables, theme, typography,
};

use gpui::App;

/// Ely's core, then the verticals' keys. Call once, first.
pub fn init(cx: &mut App) -> anyhow::Result<()> {
    ely_core::init(cx)?;
    bind_keys(cx);
    Ok(())
}

/// `init` without assets or fonts, for app tests that render.
#[cfg(any(test, feature = "test-support"))]
pub fn init_for_tests(cx: &mut App) {
    ely_core::init_for_tests(cx);
    bind_keys(cx);
}

fn bind_keys(cx: &mut App) {
    editor::bind_keys(cx);
    #[cfg(not(target_family = "wasm"))]
    terminal::bind_keys(cx);
    documents::bind_keys(cx);
}
