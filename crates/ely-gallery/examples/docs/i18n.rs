use ely_gpui_component::{
    Assets,
    buttons::Button,
    i18n::{Direction, I18n},
    primitives::FocusScope,
    theme::ActiveTheme,
};
use gpui::{
    App, AppContext, Context, FocusHandle, IntoElement, ParentElement, Render, Styled, Window,
    WindowOptions, div,
};

struct Words {
    focus: FocusHandle,
}

impl Render for Words {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let i18n = cx.global::<I18n>();
        let locale = i18n.locale();
        let greeting = i18n.text("greeting", &[("name", "Ada")]);
        let files = i18n.text("files", &[("count", &locale.number(1234.0, 0))]);
        let next = if locale.direction == Direction::Rtl {
            "en-US"
        } else {
            "he-IL"
        };
        let row = locale
            .direction
            .row(div())
            .gap_4()
            .child(greeting)
            .child(files);
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .bg(theme.colors.bg)
            .text_color(theme.colors.fg)
            .child(row)
            .child(
                Button::new("locale", i18n.text("switch", &[]))
                    .on_click(move |_, _, cx| I18n::set_locale(next, cx)),
            )
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            ely_gpui_component::init(cx).expect("Ely failed to start");
            cx.set_global(
                I18n::new("en-US")
                    .catalog(
                        "en-US",
                        &[
                            ("greeting", "Hello, {name}"),
                            ("files", "{count} files"),
                            ("switch", "עברית"),
                        ],
                    )
                    .catalog(
                        "he-IL",
                        &[
                            ("greeting", "שלום, {name}"),
                            ("files", "{count} קבצים"),
                            ("switch", "English"),
                        ],
                    ),
            );
            cx.open_window(WindowOptions::default(), |window, cx| {
                cx.new(|cx| {
                    let focus = cx.focus_handle();
                    window.focus(&focus, cx);
                    Words { focus }
                })
            })
            .expect("window failed to open");
            cx.activate(true);
        });
}
