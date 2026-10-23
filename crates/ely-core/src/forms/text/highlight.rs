use gpui::{FontWeight, Hsla};

/// Styles a span of the text: its color, an optional wash behind it, and its weight, slant and strike.
///
/// ```
/// use ely_core::forms::Highlight;
/// use gpui::{FontWeight, red};
///
/// let error = Highlight {
///     weight: Some(FontWeight::BOLD),
///     ..Highlight::new(red())
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Highlight {
    /// Its text color.
    pub color: Hsla,
    /// A wash behind it.
    pub background: Option<Hsla>,
    /// Its weight; the field's unless set.
    pub weight: Option<FontWeight>,
    /// Slanted.
    pub italic: bool,
    /// Struck through.
    pub strike: bool,
}

impl Highlight {
    /// A span in `color`, its font as the field's.
    pub fn new(color: Hsla) -> Self {
        Self {
            color,
            background: None,
            weight: None,
            italic: false,
            strike: false,
        }
    }
}
