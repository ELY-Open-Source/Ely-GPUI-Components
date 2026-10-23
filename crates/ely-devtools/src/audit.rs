use gpui::Hsla;

use crate::theme::{HUE_NAMES, Palette, Syntax, contrast, over};

/// What body text needs on its surface, as WCAG 1.4.3 has it.
pub const TEXT_CONTRAST: f32 = 4.5;
/// What a mark needs beside its surface, as WCAG 1.4.11 has it.
pub const MARK_CONTRAST: f32 = 3.0;

/// The text colors Ely draws, each over the surfaces it sits on.
const TEXT_PAIRS: &[(&str, &[&str])] = &[
    (
        "fg",
        &["bg", "surface", "sunken", "overlay", "hover", "active"],
    ),
    (
        "fg_muted",
        &["bg", "surface", "sunken", "overlay", "hover", "active"],
    ),
    (
        "fg_subtle",
        &["bg", "surface", "sunken", "overlay", "hover", "active"],
    ),
    ("link", &["bg", "surface", "sunken"]),
    ("on_accent", &["accent", "accent_hover"]),
    ("tooltip_fg", &["tooltip_bg"]),
    ("success", &["bg", "success_subtle"]),
    ("warning", &["bg", "warning_subtle"]),
    ("danger", &["bg", "danger_subtle"]),
    ("info", &["bg", "info_subtle"]),
];

/// The surfaces code sits on: the editor, a code block, the editor's current line.
const CODE: [&str; 3] = ["surface", "sunken", "hover"];

/// A color, what it stands on, the contrast between them, and the contrast it needs.
#[derive(Clone, Debug)]
pub struct Check {
    /// The color's token, such as `fg_muted` or `syntax.keyword`.
    pub front: String,
    /// The token it stands on.
    pub back: &'static str,
    /// The color.
    pub fg: Hsla,
    /// What it stands on.
    pub bg: Hsla,
    /// The contrast between them.
    pub ratio: f32,
    /// The contrast it needs.
    pub need: f32,
}

impl Check {
    /// Whether it meets what it needs.
    pub fn passes(&self) -> bool {
        self.ratio >= self.need
    }

    /// Its WCAG grade: Fails, AA, or AAA for text at 7:1 or more.
    pub fn grade(&self) -> &'static str {
        match self.passes() {
            false => "Fails",
            true if self.need == TEXT_CONTRAST && self.ratio >= 7.0 => "AAA",
            true => "AA",
        }
    }
}

/// Every pair in `palette`: its text over its surfaces and its code colors over the code surfaces at 4.5:1, the focus ring and the chart hues beside the page at 3:1.
pub fn audit(palette: &Palette) -> Vec<Check> {
    let check = |front: String, fg: Hsla, back: &'static str, need: f32| {
        let bg = palette.token(back);
        Check {
            front,
            back,
            fg,
            bg,
            ratio: contrast(over(fg, bg), bg),
            need,
        }
    };
    let text = TEXT_PAIRS.iter().flat_map(|(front, backs)| {
        backs
            .iter()
            .map(move |back| check(front.to_string(), palette.token(front), back, TEXT_CONTRAST))
    });
    let code = Syntax::NAMES.iter().flat_map(|name| {
        CODE.iter().map(move |back| {
            check(
                format!("syntax.{name}"),
                palette.syntax.token(name),
                back,
                TEXT_CONTRAST,
            )
        })
    });
    let focus = ["bg", "surface"]
        .into_iter()
        .map(|back| check("focus".into(), palette.focus, back, MARK_CONTRAST));
    let hues = HUE_NAMES.iter().enumerate().map(|(ix, name)| {
        check(
            format!("chart {name}"),
            palette.hue(ix, "contrast audit"),
            "bg",
            MARK_CONTRAST,
        )
    });
    text.chain(code).chain(focus).chain(hues).collect()
}

#[cfg(test)]
mod tests {
    use super::{Check, audit};
    use crate::theme::Palette;

    #[test]
    fn every_ely_palette_passes_its_audit() {
        for high_contrast in [false, true] {
            for palette in [Palette::light(high_contrast), Palette::dark(high_contrast)] {
                let failing: Vec<String> = audit(&palette)
                    .iter()
                    .filter(|check| !check.passes())
                    .map(|check| format!("{} on {} {:.2}", check.front, check.back, check.ratio))
                    .collect();
                assert!(
                    failing.is_empty(),
                    "high contrast {high_contrast}: {failing:?}"
                );
            }
        }
    }

    #[test]
    fn a_pair_under_its_need_fails() {
        let mut palette = Palette::light(false);
        palette.fg_subtle = palette.bg;
        let checks = audit(&palette);
        let subtle: Vec<&Check> = checks.iter().filter(|c| c.front == "fg_subtle").collect();
        assert!(subtle.iter().all(|check| check.grade() == "Fails"));
        assert_eq!(
            checks.len(),
            81,
            "32 text pairs, 39 for code, 2 for the ring, 8 hues"
        );
        assert_eq!(subtle.len(), 6);
    }

    #[test]
    fn a_mark_needs_less_than_text() {
        let mut palette = Palette::light(false);
        let gray = gpui::rgb(0x8a8a8a).into();
        (palette.focus, palette.fg_muted) = (gray, gray);
        let checks = audit(&palette);
        let grade = |front: &str| {
            checks
                .iter()
                .find(|c| c.front == front && c.back == "bg")
                .map(Check::grade)
        };
        assert_eq!(grade("focus"), Some("AA"), "a ring at 3.3:1");
        assert_eq!(grade("fg_muted"), Some("Fails"), "text at 3.3:1");
    }

    #[test]
    fn code_and_hues_read_their_own_colors() {
        let mut palette = Palette::light(false);
        palette.syntax.comment = palette.surface;
        let checks = audit(&palette);
        let failing: Vec<&str> = checks
            .iter()
            .filter(|check| !check.passes())
            .map(|check| check.back)
            .collect();
        assert_eq!(
            failing,
            ["surface", "sunken", "hover"],
            "the comment over code"
        );
        for (ix, name) in crate::theme::HUE_NAMES.iter().enumerate() {
            let hue = checks.iter().find(|c| c.front == format!("chart {name}"));
            assert_eq!(hue.map(|check| check.fg), Some(palette.chart[ix]));
        }
    }

    #[test]
    fn translucent_text_counts_as_it_shows() {
        let mut palette = Palette::light(false);
        palette.fg_muted = palette.fg.alpha(0.2);
        let checks = audit(&palette);
        let muted = checks
            .iter()
            .find(|c| c.front == "fg_muted" && c.back == "bg");
        assert_eq!(
            muted.map(Check::grade),
            Some("Fails"),
            "a fifth of fg over the page"
        );
    }
}
