use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    settings::{ThemeDraft, ThemeEditor},
    theme::{ActiveTheme, Density, Mode, Theme},
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::ui::section;

/// Applies a draft to the app's theme: its measures and switches, and its colors as the shown mode's own.
fn apply(draft: &ThemeDraft, cx: &mut App) {
    let now = ThemeDraft::of(cx.theme());
    Theme::update(cx, |theme| {
        theme.density = draft.density;
        theme.radius_scale = draft.radius_scale;
        theme.font_scale = draft.font_scale;
        theme.reduced_motion = draft.reduced_motion;
    });
    if draft.high_contrast != now.high_contrast {
        Theme::set_high_contrast(draft.high_contrast, cx);
    }
    if draft.colors != now.colors {
        Theme::set_palette(cx.theme().mode(), Some(draft.colors.clone()), cx);
    }
}

/// Ely's own theme again, in both modes.
fn restore(cx: &mut App) {
    Theme::update(cx, |theme| {
        theme.density = Density::Standard;
        theme.radius_scale = 1.0;
        theme.font_scale = 1.0;
    });
    Theme::set_palette(Mode::Light, None, cx);
    Theme::set_palette(Mode::Dark, None, cx);
    Theme::set_high_contrast(false, cx);
}

pub fn editor(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let draft = ThemeDraft::of(cx.theme());
    section(
        "ThemeEditor · DensityProvider · RadiusProvider · FontProvider · HighContrastMode · ReducedMotionProvider",
        "The theme laid open: every change here applies to the whole gallery at once, the colors as the shown mode's own. The theme switch above is settings::ThemeSelector.",
        cx,
    )
    .child(
        div()
            .w(px(520.))
            .flex()
            .flex_col()
            .gap_4()
            .child(ThemeEditor::new("theme-editor", draft, |draft, _, cx| apply(draft, cx)))
            .child(
                div().flex().child(
                    Button::new("theme-restore", "Back to Ely's theme")
                        .variant(ButtonVariant::Secondary)
                        .on_click(|_, _, cx| restore(cx)),
                ),
            ),
    )
}
