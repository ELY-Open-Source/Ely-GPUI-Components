//! Ely's core: the theme, motion, i18n and the general components, chapters 1–14.
//!
//! Every component in Ely runs on macOS, Windows, Linux and the web, unless its doc says `Platforms:` and names fewer.

#![deny(missing_docs)]

mod assets;
/// Buttons and actions: Button, IconButton, groups, segmented controls, copy and share.
pub mod buttons;
/// Read-only displays: badges, tags, avatars, statistics, gauges, timelines and codes.
pub mod data_display;
/// What happened and where things stand: alerts, toasts, notifications, empty and error states.
pub mod feedback;
/// Text fields, choices, pickers, sliders and the form that holds them.
pub mod forms;
/// Messages by key, locales and right-to-left layout.
pub mod i18n;
/// Boxes, stacks, cards, panes, docks, sheets and the splits between them.
pub mod layout;
/// Lists and trees, selectable rows and titled sections.
pub mod lists;
/// Dropdown, context and menu-bar menus.
pub mod menus;
/// Durations, easings, springs and the effects built on them.
pub mod motion;
/// Tabs, breadcrumbs, palettes, pagination, steps and wizards.
pub mod navigation;
/// Dialogs, popovers, hover cards, lightboxes and tours.
pub mod overlays;
mod platform;
/// Icons, focus, tooltips and the parts every component shares.
pub mod primitives;
/// The window: title bars, toolbars, status bars, columns and window management.
pub mod shell;
/// Data tables, pivots, spreadsheets and their filters.
pub mod tables;
/// Palettes, tokens and the theme every component reads.
pub mod theme;
/// Text, headings, links, keys, numbers and formats.
pub mod typography;

pub use assets::{Assets, Before};

use anyhow::Context as _;
use gpui::{App, KeyBinding};

use crate::primitives::{FOCUS_CONTEXT, FocusNext, FocusPrev};

/// Loads fonts and the theme, binds Tab and the core's keys. Call once, first.
pub fn init(cx: &mut App) -> anyhow::Result<()> {
    let missing = assets::missing(cx.asset_source().as_ref());
    if let Some(first) = missing.first() {
        anyhow::bail!(
            "ely: the asset source lacks {} of Ely's files, first {first}; pass `ely_core::Assets`, or `Assets::before` your own, to `Application::with_assets`",
            missing.len()
        );
    }
    log::info!("assets: the app's source loads all of Ely's files");
    assets::load_fonts(cx).context("ely: embedded fonts failed to register")?;
    if !cx
        .text_system()
        .all_font_names()
        .iter()
        .any(|name| name == "IBM Plex Sans")
    {
        anyhow::bail!(
            "ely: the platform has no text system, so nothing would draw words; turn on ely-core's `platform` feature, or gpui_platform's `font-kit` (macOS) and `wayland` and `x11` (Linux)"
        );
    }
    setup(cx);
    Ok(())
}

/// `init` without assets or fonts, for app tests that render.
#[cfg(any(test, feature = "test-support"))]
pub fn init_for_tests(cx: &mut App) {
    setup(cx);
}

/// The theme and key bindings.
fn setup(cx: &mut App) {
    theme::Theme::init(cx);
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, Some(FOCUS_CONTEXT)),
        KeyBinding::new("shift-tab", FocusPrev, Some(FOCUS_CONTEXT)),
        // Out of a field that keeps Tab, such as a document.
        KeyBinding::new("f6", FocusNext, Some(FOCUS_CONTEXT)),
        KeyBinding::new("shift-f6", FocusPrev, Some(FOCUS_CONTEXT)),
    ]);
    forms::bind_keys(cx);
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{AppContext, TestAppContext};

    #[gpui::test]
    #[should_panic(expected = "leaked handles")]
    fn a_leaked_entity_fails_its_test(cx: &mut TestAppContext) {
        std::mem::forget(cx.new(|_| ()));
    }
}
