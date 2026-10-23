//! Settings, chapter 35: pages, sections and rows, appearance, themes, privacy, notices, startup, network, storage and shortcuts.
#![deny(missing_docs)]

use ely_canvas as canvas;
use ely_core::{
    buttons, data_display, feedback, forms, i18n, lists, primitives, theme, typography,
};

mod advanced;
mod appearance;
mod editor;
mod importer;
mod network;
mod preferences;
mod search;
mod section;
mod shortcuts;
mod storage;
mod syntax;
mod vscode;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use advanced::{DeveloperModeToggle, FeatureFlags, Flag, Stage};
pub use appearance::{
    AccentColorPicker, Appearance, DensitySelector, FontSizeControl, ThemeSelector,
};
pub use editor::{ThemeDraft, ThemeEditor};
pub use importer::ThemeImporter;
pub use network::{Proxy, ProxySettings, manual};
pub use preferences::{
    Notices, NotificationSettings, Privacy, PrivacySettings, Startup, StartupSettings,
};
pub use search::{SettingEntry, SettingsSearch, matching};
pub use section::{SettingsLayout, SettingsRow, SettingsSection};
pub use shortcuts::{KeyboardShortcutsList, Shortcut};
pub use storage::{Changed, ImportExportSettings, ResetToDefault, StorageSettings, Store};
pub use syntax::SyntaxThemePicker;
pub use vscode::{VsCodeTheme, read_vscode_theme};
