use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, TestAppContext,
    Window, div, px,
};

use super::super::ScrollArea;
use crate::theme::Theme;

/// A full-width row in an area; tall when it scrolls.
struct Rows(f32);

impl Render for Rows {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        ScrollArea::new("rows")
            .vertical()
            .w(px(300.))
            .h(px(200.))
            .child(div().debug_selector(|| "row".into()).w_full().h(px(self.0)))
    }
}

/// The row's right edge and the area's, when bars stay.
fn edges(cx: &mut TestAppContext, tall: f32) -> (f32, f32) {
    cx.update(Theme::init);
    cx.update(|cx| Theme::update(cx, |theme| theme.scrollbars_shown = true));
    let (_, cx) = cx.add_window_view(move |_, _| Rows(tall));
    cx.run_until_parked();
    let row = cx.debug_bounds("row").expect("the row");
    if tall > 200. {
        let thumb = cx.debug_bounds("scrollbar-thumb").expect("a shown thumb");
        assert!(row.right() <= thumb.left(), "the thumb covers the row");
    }
    (f32::from(row.right()), f32::from(row.left()) + 300.)
}

#[gpui::test]
fn a_shown_bar_never_covers_the_rows(cx: &mut TestAppContext) {
    let (row, area) = edges(cx, 2000.);
    assert_eq!(row, area - 12., "the row leaves the track's strip");
}

#[gpui::test]
fn rows_keep_their_width_when_nothing_scrolls(cx: &mut TestAppContext) {
    let (row, area) = edges(cx, 50.);
    assert_eq!(row, area - 12., "the width jumps at the overflow");
}
