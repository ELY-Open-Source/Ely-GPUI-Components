use ely_core::primitives;

mod surface;
#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use surface::RenderSurface;
