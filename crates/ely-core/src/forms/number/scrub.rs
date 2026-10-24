//! A number field with a label you drag.

use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window};

use super::NumberInput;

/// A label you drag sideways to change a number, beside its field.
///
/// ```
/// use ely_core::forms::ScrubInput;
///
/// let width = ScrubInput::new("width", "W", 240.0)
///     .range(0.0, 4096.0)
///     .on_change(|_value, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct ScrubInput {
    field: NumberInput,
}

impl ScrubInput {
    /// A number field under `id` beside `label`; a drag across the label scrubs `value`.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, value: f64) -> Self {
        let label = label.into();
        let mut field = NumberInput::new(id, value);
        field.label = label.clone();
        field.scrub_label = Some(label);
        Self { field }
    }

    /// Its least and greatest value. Panics on an empty range.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.field = self.field.range(min, max);
        self
    }

    /// How far a step, an arrow key or a scrub moves it. Panics unless positive.
    pub fn step(mut self, step: f64) -> Self {
        self.field = self.field.step(step);
        self
    }

    /// Places after the point.
    pub fn precision(mut self, places: usize) -> Self {
        self.field = self.field.precision(places);
        self
    }

    /// Runs with each new value, typed ones included.
    pub fn on_change(mut self, handler: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.field = self.field.on_change(handler);
        self
    }
}

impl RenderOnce for ScrubInput {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.field
    }
}

crate::capable!(ScrubInput: change(f64));
