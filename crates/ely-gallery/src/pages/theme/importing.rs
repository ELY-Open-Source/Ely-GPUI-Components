use ely_gpui_component::{
    settings::{SyntaxThemePicker, ThemeImporter},
    theme::{ActiveTheme, Theme, syntax_themes},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::ui::{change, keep, section, specimen, specimens};

/// Which code palette the demo last chose.
struct Chosen(SharedString);

pub fn importing(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chosen = keep("theme-syntax", || Chosen("Ely".into()), window, cx);
    let name = chosen.read(cx).0.clone();
    section(
        "ThemeImporter · SyntaxThemes",
        "A VS Code color theme dropped here reads into Ely's palette, the colors it leaves out kept from Ely's; Apply sets it on the whole gallery. Code palettes choose the colors of code for the mode shown.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "ThemeImporter",
                div().w(px(360.)).child(ThemeImporter::new("theme-importer", |read, _, cx| {
                    Theme::set_palette(read.mode, Some(read.colors.clone()), cx);
                    Theme::set_mode(read.mode, cx);
                })),
                cx,
            ))
            .child(specimen(
                "SyntaxThemePicker",
                div().w(px(360.)).child(SyntaxThemePicker::new("theme-syntax", syntax_themes(), name, 
                    move |picked, _, cx| {
                        let theme = syntax_themes()
                            .into_iter()
                            .find(|theme| theme.name == picked)
                            .expect("a listed syntax theme");
                        let mode = cx.theme().mode();
                        let mut colors = cx.theme().palette();
                        colors.syntax = theme.of(mode);
                        Theme::set_palette(mode, Some(colors), cx);
                        change(&chosen, cx, |chosen| chosen.0 = picked.into());
                    },
                )),
                cx,
            )),
    )
}
