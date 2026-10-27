//! Ely GPUI Component: every Ely crate under its chapter's name, and `init`, which binds every crate's keys.
#![deny(missing_docs)]

#[cfg(test)]
mod containers;
#[cfg(test)]
mod shipped;

pub use ely_account as account;
pub use ely_agent as agent;
pub use ely_calendar as calendar;
pub use ely_canvas as canvas;
pub use ely_charts as charts;
pub use ely_chat as chat;
pub use ely_collab as collab;
pub use ely_core::{
    Assets, Before, buttons, data_display, feedback, forms, i18n, layout, lists, menus, motion,
    navigation, overlays, primitives, secondary, shell, tables, theme, typography,
};
pub use ely_dashboard as dashboard;
pub use ely_debug as debug;
pub use ely_desktop as desktop;
pub use ely_devtools as devtools;
pub use ely_documents as documents;
pub use ely_editor as editor;
pub use ely_files as files;
pub use ely_finance as finance;
pub use ely_generative as generative;
pub use ely_git as git;
pub use ely_interaction as interaction;
pub use ely_mail as mail;
pub use ely_maps as maps;
pub use ely_media as media;
pub use ely_messaging as messaging;
pub use ely_misc as misc;
pub use ely_onboarding as onboarding;
pub use ely_project as project;
pub use ely_rendering as rendering;
pub use ely_settings as settings;
pub use ely_terminal as terminal;
pub use ely_tooling as tooling;

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
