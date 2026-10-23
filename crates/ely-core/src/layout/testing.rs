use gpui::{
    AnyElement, App, Context, IntoElement, ParentElement, Pixels, Render, Styled, TestAppContext,
    Window, div, px,
};

type Build = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// One element in a column that a padded block measures by content.
struct Narrow(Build);

impl Render for Narrow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().w(px(280.0)).flex().flex_col().child(
            div().p_5().child(
                div()
                    .flex()
                    .flex_col()
                    .child((self.0)(window, cx))
                    .child("A line long enough to wrap in a narrow card, so the column fills it."),
            ),
        )
    }
}

/// The width of `selector` in that column; a root that fills it takes all 240px.
#[doc(hidden)]
pub fn narrow_width(
    cx: &mut TestAppContext,
    selector: &'static str,
    build: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
) -> Pixels {
    let (_, cx) = cx.add_window_view(|_, _| Narrow(Box::new(build)));
    cx.run_until_parked();
    cx.debug_bounds(selector).expect(selector).size.width
}
