use std::{cell::Cell, rc::Rc};

use gpui::{
    App, Axis, Context, IntoElement, Modifiers, MouseButton, ParentElement, Pixels, Point, Render,
    Styled, TestAppContext, Window, div, point, px,
};

use super::{ScrollModel, ScrollPlace, ScrollSource, Scrollbar};
use crate::theme::Theme;

/// A quarter shows; drags set `ratio`.
struct Owned {
    ratio: Cell<f32>,
    fits: bool,
}

impl ScrollModel for Owned {
    fn place(&self, _: Axis, _: &App) -> Option<ScrollPlace> {
        (!self.fits).then(|| ScrollPlace {
            span: px(400.),
            shown: 0.25,
            passed: self.ratio.get(),
        })
    }

    fn set(&self, _: Axis, ratio: f32, _: &mut App) {
        self.ratio.set(ratio);
    }

    fn offset(&self, _: &App) -> Point<Pixels> {
        point(px(self.ratio.get()), px(0.))
    }
}

struct Bar(Rc<Owned>);

impl Render for Bar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let source = ScrollSource::Model(self.0.clone());
        div()
            .relative()
            .w(px(400.))
            .h(px(40.))
            .child(Scrollbar::new("bar", source, Axis::Horizontal))
    }
}

fn opened(fits: bool, cx: &mut TestAppContext) -> (Rc<Owned>, &mut gpui::VisualTestContext) {
    cx.update(Theme::init);
    let owned = Rc::new(Owned {
        ratio: Cell::new(0.),
        fits,
    });
    let held = owned.clone();
    let (_, cx) = cx.add_window_view(move |_, _| Bar(held));
    cx.run_until_parked();
    (owned, cx)
}

#[gpui::test]
fn a_kept_scroll_sizes_its_thumb_and_takes_drags(cx: &mut TestAppContext) {
    let (owned, cx) = opened(false, cx);
    let thumb = cx.debug_bounds("scrollbar-thumb").expect("a thumb");
    assert_eq!(thumb.size.width, px(100.));
    let grip = thumb.center();
    cx.simulate_mouse_move(grip, None, Modifiers::none());
    cx.simulate_mouse_down(grip, MouseButton::Left, Modifiers::none());
    // The first move starts the drag, the next anchors it.
    for right in [10., 20., 170.] {
        cx.simulate_mouse_move(
            grip + point(px(right), px(0.)),
            MouseButton::Left,
            Modifiers::none(),
        );
    }
    cx.simulate_mouse_up(
        grip + point(px(170.), px(0.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.run_until_parked();
    let ratio = owned.ratio.get();
    assert!((ratio - 0.5).abs() < 0.01, "dragged to {ratio}");
}

#[gpui::test]
fn a_kept_scroll_that_fits_shows_no_thumb(cx: &mut TestAppContext) {
    let (_, cx) = opened(true, cx);
    assert!(cx.debug_bounds("scrollbar-thumb").is_none());
}
