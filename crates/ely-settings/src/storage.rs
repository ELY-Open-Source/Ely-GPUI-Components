use std::{path::PathBuf, rc::Rc};

use gpui::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use crate::{
    buttons::{Button, ButtonVariant, ConfirmButton, ConfirmMode, IconButton},
    data_display::UsageBar,
    forms::DropZone,
    primitives::IconName,
    theme::{ActiveTheme, TextSize},
    typography::{Ellipsis, format},
};

/// A store on disk: its key and name, its size in bytes, and whether it may be cleared.
#[derive(Clone, Debug, PartialEq)]
pub struct Store {
    /// Tells it apart from the others.
    pub key: SharedString,
    /// What it shows.
    pub name: SharedString,
    /// Its size, in bytes.
    pub bytes: u64,
    /// Whether it may be cleared.
    pub clearable: bool,
}

type OnKey = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// What an app keeps on disk: a bar of each store's share, then each store with its size and, where it may go, a Clear that asks twice.
///
/// ```
/// use ely_settings::{StorageSettings, Store};
///
/// let cache = Store { key: "cache".into(), name: "Cache".into(), bytes: 48 << 20, clearable: true };
/// let settings = StorageSettings::new("storage", [cache], |_key, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct StorageSettings {
    id: ElementId,
    stores: Vec<Store>,
    on_clear: OnKey,
}

impl StorageSettings {
    /// Settings under `id` of `stores`; `on_clear` gets the key of the store to clear.
    pub fn new(
        id: impl Into<ElementId>,
        stores: impl IntoIterator<Item = Store>,
        on_clear: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            stores: stores.into_iter().collect(),
            on_clear: Rc::new(on_clear),
        }
    }
}

impl RenderOnce for StorageSettings {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_clear = self.on_clear;
        let theme = cx.theme();
        let total: u64 = self.stores.iter().map(|store| store.bytes).sum();
        let bar = self.stores.iter().fold(
            UsageBar::new(total.max(1) as f64)
                .amounts(|bytes| format::file_size(bytes as u64, true)),
            |bar, store| bar.part(store.name.clone(), store.bytes as f64),
        );
        let rows = self.stores.iter().map(|store| {
            let (key, on_clear) = (store.key.clone(), on_clear.clone());
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap_2()
                .py_2()
                .border_b_1()
                .border_color(theme.colors.border)
                .child(
                    div()
                        .flex_1()
                        .min_w(theme.label_width())
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(theme.text_size(TextSize::Sm))
                                .child(Ellipsis::new(store.name.clone())),
                        )
                        .child(
                            div()
                                .text_size(theme.text_size(TextSize::Xs))
                                .text_color(theme.colors.fg_muted)
                                .child(format::file_size(store.bytes, true)),
                        ),
                )
                .children(store.clearable.then(|| {
                    ConfirmButton::new(
                        (self.id.clone(), format!("clear-{}", store.key)),
                        "Clear",
                        ConfirmMode::Twice,
                    )
                    .on_confirm(move |window, cx| {
                        log::info!("storage: clear {key}");
                        on_clear(&key, window, cx);
                    })
                }))
        });
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_size(theme.text_size(TextSize::Sm))
                    .text_color(theme.colors.fg_muted)
                    .child(format!("{} in all", format::file_size(total, true))),
            )
            .child(bar)
            .child(div().flex().flex_col().children(rows))
    }
}

/// A setting changed from its default: its key and name, and both values as words.
#[derive(Clone, Debug, PartialEq)]
pub struct Changed {
    /// Tells it apart from the others.
    pub key: SharedString,
    /// What it shows.
    pub name: SharedString,
    /// Its value now, as words.
    pub now: SharedString,
    /// Its default, as words.
    pub default: SharedString,
}

type Run = Rc<dyn Fn(&mut Window, &mut App)>;

/// The settings changed from their defaults, each with what it is and what it was, and a way back for each; Reset all asks twice. With none changed it says so.
///
/// ```
/// use ely_settings::{Changed, ResetToDefault};
///
/// let font = Changed { key: "font".into(), name: "Font size".into(), now: "16 px".into(), default: "14 px".into() };
/// let reset = ResetToDefault::new("reset", [font], |_key, _, _| {}, |_, _| {});
/// ```
#[derive(IntoElement)]
pub struct ResetToDefault {
    id: ElementId,
    changed: Vec<Changed>,
    on_reset: OnKey,
    on_reset_all: Run,
}

impl ResetToDefault {
    /// A list under `id` of `changed`; `on_reset` gets the key to reset, `on_reset_all` the press on Reset all.
    pub fn new(
        id: impl Into<ElementId>,
        changed: impl IntoIterator<Item = Changed>,
        on_reset: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
        on_reset_all: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            changed: changed.into_iter().collect(),
            on_reset: Rc::new(on_reset),
            on_reset_all: Rc::new(on_reset_all),
        }
    }
}

impl RenderOnce for ResetToDefault {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_reset = self.on_reset;
        let on_reset_all = self.on_reset_all;
        let theme = cx.theme();
        if self.changed.is_empty() {
            return div()
                .text_size(theme.text_size(TextSize::Sm))
                .text_color(theme.colors.fg_muted)
                .child("Every setting is at its default.")
                .into_any_element();
        }
        let rows = self.changed.iter().map(|changed| {
            let (key, on_reset) = (changed.key.clone(), on_reset.clone());
            div()
                .flex()
                .items_center()
                .gap_2()
                .py_2()
                .border_b_1()
                .border_color(theme.colors.border)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(theme.text_size(TextSize::Sm))
                                .child(Ellipsis::new(changed.name.clone())),
                        )
                        .child(
                            div()
                                .text_size(theme.text_size(TextSize::Xs))
                                .text_color(theme.colors.fg_muted)
                                .child(Ellipsis::new(format!(
                                    "{} · was {}",
                                    changed.now, changed.default
                                ))),
                        ),
                )
                .child(
                    IconButton::new(
                        (self.id.clone(), format!("reset-{}", changed.key)),
                        IconName::RotateCcw,
                    )
                    .tooltip("Back to the default")
                    .on_click(move |_, window, cx| {
                        log::info!("reset: {key}");
                        on_reset(&key, window, cx);
                    }),
                )
        });
        let on_reset_all = on_reset_all;
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().flex().flex_col().children(rows))
            .child(
                div().flex().child(
                    ConfirmButton::new(
                        (self.id, "all"),
                        format!("Reset all {}", self.changed.len()),
                        ConfirmMode::Twice,
                    )
                    .on_confirm(move |window, cx| {
                        log::info!("reset: all");
                        on_reset_all(window, cx);
                    }),
                ),
            )
            .into_any_element()
    }
}

type OnFile = Rc<dyn Fn(PathBuf, &mut Window, &mut App)>;

/// Settings out to a file and back: Export asks the owner to write them, and a JSON file dropped or picked asks it to read one.
///
/// ```
/// use ely_settings::ImportExportSettings;
///
/// let settings = ImportExportSettings::new("files", |_, _| {}, |_path, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct ImportExportSettings {
    id: ElementId,
    on_export: Run,
    on_import: OnFile,
}

impl ImportExportSettings {
    /// Import and export under `id`; `on_export` gets the press on Export, `on_import` the file to read.
    pub fn new(
        id: impl Into<ElementId>,
        on_export: impl Fn(&mut Window, &mut App) + 'static,
        on_import: impl Fn(PathBuf, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            on_export: Rc::new(on_export),
            on_import: Rc::new(on_import),
        }
    }
}

impl RenderOnce for ImportExportSettings {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let on_export = self.on_export;
        let on_import = self.on_import;
        let (on_export, on_import) = (on_export, on_import);
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div().flex().child(
                    Button::new((self.id.clone(), "export"), "Export settings")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, window, cx| {
                            log::info!("settings: export");
                            on_export(window, cx);
                        }),
                ),
            )
            .child(DropZone::new((self.id, "import")).kinds(&["json"]).on_drop(
                move |paths, window, cx| {
                    let Some(path) = paths.first().cloned() else {
                        return;
                    };
                    log::info!("settings: import {}", path.display());
                    on_import(path, window, cx);
                },
            ))
    }
}
