//! Desktop: the tray icon, the Dock badge and Omarchy's theme.
#![deny(missing_docs)]

use ely_core::{forms, theme};
use ely_devtools as devtools;

mod native;
mod omarchy;
mod platform;

pub use native::{TrayIcon, TrayItem, TrayLook, dock_badge, set_dock_badge};
pub use omarchy::{OmarchyTheme, omarchy_current_dir, read_omarchy_theme};
#[cfg(not(target_family = "wasm"))]
pub use omarchy::{follow_omarchy, stop_omarchy};
