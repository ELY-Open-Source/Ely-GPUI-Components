//! Ely's terminal for GPUI: a pseudo-terminal drawn cell by cell, colored output, command blocks, logs and processes.
#![deny(missing_docs)]

use ely_core::{buttons, i18n, menus, motion, navigation, primitives, tables, theme, typography};
#[cfg(not(target_family = "wasm"))]
use ely_editor as editor;

mod ansi;
mod block;
mod colors;
#[cfg(not(target_family = "wasm"))]
mod frame;
mod history;
#[cfg(not(target_family = "wasm"))]
mod input;
#[cfg(not(target_family = "wasm"))]
mod keys;
mod links;
mod logs;
#[cfg(not(target_family = "wasm"))]
mod paint;
mod processes;
#[cfg(not(target_family = "wasm"))]
mod pty;
#[cfg(not(target_family = "wasm"))]
mod search;
mod shells;
#[cfg(all(test, feature = "test-support"))]
mod tests;
#[cfg(not(target_family = "wasm"))]
mod view;

pub use ansi::AnsiText;
pub use block::{CommandBlock, CommandState};
pub use history::{CommandHistory, PastCommand};
#[cfg(not(target_family = "wasm"))]
pub use input::bind_keys;
pub use links::{Target, links};
pub use logs::{LogLevel, LogLine, LogViewer};
pub use processes::{Process, ProcessList};
#[cfg(not(target_family = "wasm"))]
pub use pty::Launch;
pub use shells::{ShellChoice, ShellSelector, TerminalToolbar};
#[cfg(not(target_family = "wasm"))]
pub use view::{Terminal, TerminalEvent};
