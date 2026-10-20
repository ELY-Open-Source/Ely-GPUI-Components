use gpui::{FontStyle, HighlightStyle, Pixels, StrikethroughStyle, UnderlineStyle};

use super::HEAVY;
use crate::{documents::tree::Style, theme::Palette};

/// How one inline style draws.
pub(super) fn highlight(style: &Style, colors: &Palette, rule: Pixels) -> HighlightStyle {
    match style {
        Style::Strong => HighlightStyle {
            font_weight: Some(HEAVY),
            ..HighlightStyle::default()
        },
        Style::Emphasis => HighlightStyle {
            font_style: Some(FontStyle::Italic),
            ..HighlightStyle::default()
        },
        Style::Strike => HighlightStyle {
            strikethrough: Some(StrikethroughStyle {
                thickness: rule,
                color: Some(colors.fg_muted),
            }),
            ..HighlightStyle::default()
        },
        Style::Code | Style::Math => HighlightStyle {
            background_color: Some(colors.sunken),
            color: Some(if *style == Style::Math {
                colors.syntax.function
            } else {
                colors.fg
            }),
            ..HighlightStyle::default()
        },
        Style::Link => HighlightStyle {
            color: Some(colors.link),
            underline: Some(UnderlineStyle {
                thickness: rule,
                color: Some(colors.link.opacity(0.4)),
                wavy: false,
            }),
            ..HighlightStyle::default()
        },
        Style::Footnote => HighlightStyle {
            color: Some(colors.fg_subtle),
            ..HighlightStyle::default()
        },
        Style::Underline => HighlightStyle {
            underline: Some(UnderlineStyle {
                thickness: rule,
                color: None,
                wavy: false,
            }),
            ..HighlightStyle::default()
        },
        Style::Mark => HighlightStyle {
            background_color: Some(colors.warning_subtle),
            ..HighlightStyle::default()
        },
    }
}
