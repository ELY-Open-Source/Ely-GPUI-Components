use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled,
    TestAppContext, VisualTestContext, Window, div, px,
};

use super::super::{Input, TextInput};
use super::setup;

struct Narrow(Entity<TextInput>);

impl Render for Narrow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(120.)).child(Input::new(&self.0))
    }
}

fn long_field(cx: &mut TestAppContext) -> (Entity<TextInput>, &mut VisualTestContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|window, cx| {
        let field = cx.new(|cx| TextInput::new(window, cx));
        Narrow(field)
    });
    let field = view.read_with(cx, |view, _| view.0.clone());
    let text = "word ".repeat(60);
    field.update(cx, |input, cx| input.set_text(text, cx));
    cx.run_until_parked();
    (field, cx)
}

fn scrolled(field: &Entity<TextInput>, cx: &mut VisualTestContext) -> Pixels {
    field.read_with(cx, |input, _| input.scroll.x)
}

#[gpui::test]
fn an_unfocused_field_shows_a_selection_past_its_edge(cx: &mut TestAppContext) {
    let (field, cx) = long_field(cx);
    field.update(cx, |input, cx| {
        let end = input.text().len();
        input.select(end - 4..end, cx);
    });
    cx.run_until_parked();
    assert!(scrolled(&field, cx) > px(0.), "the selection stays hidden");
}

#[gpui::test]
fn an_unfocused_field_without_a_selection_shows_its_start(cx: &mut TestAppContext) {
    let (field, cx) = long_field(cx);
    field.update(cx, |input, cx| {
        let end = input.text().len();
        input.select(end..end, cx);
    });
    cx.run_until_parked();
    assert_eq!(scrolled(&field, cx), px(0.));
}
