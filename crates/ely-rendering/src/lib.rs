//! Rendering, chapter 44: a surface for pixels the host draws.
#![deny(missing_docs)]

use ely_core::primitives;

mod surface;
#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use surface::RenderSurface;
