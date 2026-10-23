use ely_gpui_component::{
    Assets,
    buttons::Button,
    primitives::FocusScope,
    theme::{ActiveTheme, Density, Elevation, Mode, Radius, TextSize, Theme},
    typography::Paragraph,
};
use gpui::{
    App, AppContext, Context, FocusHandle, IntoElement, ParentElement, Render, Styled, Window,
    WindowOptions, div, rgb,
};

struct Tokens {
    focus: FocusHandle,
}

impl Render for Tokens {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let dark = theme.is_dark();
        let contrast = theme.high_contrast();
        let compact = theme.density == Density::Compact;
        let card = div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .bg(theme.colors.surface)
            .border_1()
            .border_color(theme.colors.border)
            .rounded(theme.radius(Radius::Lg))
            .shadow(theme.elevation(Elevation::Floating))
            .text_size(theme.text_size(TextSize::Lg))
            .text_color(theme.colors.fg)
            .child("A card made of tokens")
            .child(Paragraph::new(
                "Its colors, radius, shadow and text size come from the theme.",
            ));
        let controls = div()
            .flex()
            .flex_wrap()
            .gap_2()
            .child(
                Button::new("mode", if dark { "Light mode" } else { "Dark mode" }).on_click(
                    move |_, _, cx| {
                        Theme::set_mode(if dark { Mode::Light } else { Mode::Dark }, cx)
                    },
                ),
            )
            .child(
                Button::new(
                    "contrast",
                    if contrast {
                        "Usual contrast"
                    } else {
                        "High contrast"
                    },
                )
                .on_click(move |_, _, cx| Theme::set_high_contrast(!contrast, cx)),
            )
            .child(
                Button::new(
                    "density",
                    if compact {
                        "Standard density"
                    } else {
                        "Compact"
                    },
                )
                .on_click(move |_, _, cx| {
                    Theme::update(cx, |theme| {
                        theme.density = if compact {
                            Density::Standard
                        } else {
                            Density::Compact
                        };
                    })
                }),
            )
            .child(Button::new("accent", "Green accent").on_click(|_, _, cx| {
                let mode = cx.theme().mode();
                let mut palette = cx.theme().palette();
                *palette.token_mut("accent") = rgb(0x2f7d4f).into();
                Theme::set_palette(mode, Some(palette), cx);
            }));
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .gap_6()
            .p_8()
            .bg(theme.colors.bg)
            .child(card)
            .child(controls)
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            ely_gpui_component::init(cx).expect("Ely failed to start");
            cx.open_window(WindowOptions::default(), |window, cx| {
                cx.new(|cx| {
                    let focus = cx.focus_handle();
                    window.focus(&focus, cx);
                    Tokens { focus }
                })
            })
            .expect("window failed to open");
            cx.activate(true);
        });
}
