//! Interaction, chapter 38: resizing, turning, roving focus, selection areas and synced scrolling.
#![deny(missing_docs)]

use ely_core::{i18n, layout, primitives, theme};

mod resizable;
mod rotatable;
mod roving;
mod selection;
mod sync;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use resizable::Resizable;
pub use rotatable::Rotatable;
pub use roving::RovingFocus;
pub use selection::SelectionArea;
pub use sync::ScrollSync;
