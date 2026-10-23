use ely_gpui_component::{
    Assets,
    buttons::Button,
    primitives::FocusScope,
    theme::{ActiveTheme, Mode, Theme},
    typography::{Heading, Paragraph},
};
use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, IntoElement, ParentElement, Render, Styled,
    TitlebarOptions, Window, WindowBounds, WindowOptions, px, size,
};

struct Start {
    focus: FocusHandle,
}

impl Render for Start {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let next = if theme.is_dark() {
            Mode::Light
        } else {
            Mode::Dark
        };
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .bg(theme.colors.bg)
            .child(Heading::h1("Start"))
            .child(Paragraph::new(
                "Every Ely component reads the theme, so one call turns them all.",
            ))
            .child(
                Button::new("mode", format!("{next:?} mode"))
                    .on_click(move |_, _, cx| Theme::set_mode(next, cx)),
            )
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            ely_gpui_component::init(cx).expect("Ely failed to start");
            Theme::set_mode_now(Mode::Dark, cx);
            let bounds = Bounds::centered(None, size(px(640.0), px(400.0)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Start".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };
            cx.open_window(options, |window, cx| {
                cx.new(|cx| {
                    let focus = cx.focus_handle();
                    window.focus(&focus, cx);
                    Start { focus }
                })
            })
            .expect("window failed to open");
            cx.activate(true);
        });
}
