//! Five icons: its release binary holds their SVGs and no other.

use ely_core::{
    Assets,
    primitives::{Icon, IconName},
    theme::ActiveTheme,
};
use gpui::{
    App, AppContext, Context, IntoElement, ParentElement, Render, Styled, Window, WindowOptions,
    div,
};

const SHOWN: [IconName; 5] = [
    IconName::Check,
    IconName::Search,
    IconName::Settings,
    IconName::Star,
    IconName::X,
];

struct Five;

impl Render for Five {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .gap_4()
            .bg(cx.theme().colors.bg)
            .children(SHOWN.map(Icon::new))
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            ely_core::init(cx).expect("Ely failed to start");
            cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Five))
                .expect("window failed to open");
            cx.activate(true);
        });
}
