pub mod account;
pub mod agent;
pub mod calendar;
pub mod canvas;
pub mod chat;
pub mod collab;
pub mod dashboard;
pub mod devtools;
pub mod documents;
pub mod files;
pub mod generative;
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
pub mod tooling;

#[cfg(test)]
mod shipped;

pub use ely_charts as charts;
pub use ely_core::{
    Assets, Before, buttons, data_display, feedback, forms, i18n, layout, lists, menus, motion,
    navigation, overlays, primitives, shell, tables, theme, typography,
};
pub use ely_debug as debug;
pub use ely_editor as editor;
pub use ely_finance as finance;
pub use ely_git as git;
pub use ely_terminal as terminal;

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
