//! Tooling, chapter 43: a playground, an inspector, render and frame meters, an event logger, a contrast checker and the design tokens.
#![deny(missing_docs)]

#[cfg(debug_assertions)]
use ely_core::buttons;
use ely_core::{data_display, forms, layout, primitives, theme, typography};
use ely_devtools as devtools;

mod a11y;
mod events;
#[cfg(debug_assertions)]
mod inspector;
mod perf;
mod playground;
mod renders;
#[cfg(all(test, feature = "test-support"))]
mod tests;
mod tokens;

pub use a11y::A11yChecker;
pub use events::EventLogger;
#[cfg(debug_assertions)]
pub use inspector::install_inspector;
pub use perf::FpsMeter;
pub use playground::{Knob, Playground, Setting, Settings};
pub use renders::RenderCounter;
pub use tokens::DesignTokenViewer;
