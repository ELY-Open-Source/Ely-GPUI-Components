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

/// A tall area; its renders, bounded so a loop fails.
struct Tall(usize, gpui::ScrollHandle, f32);

impl Render for Tall {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.0 += 1;
        assert!(self.0 < 100, "the area redraws without end");
        super::ScrollArea::new("tall")
            .track_scroll(&self.1)
            .vertical()
            .w(px(300.))
            .h(px(200.))
            .child(div().h(px(self.2)))
    }
}

/// A tall area, its thumb shown by hover.
fn hovered(
    cx: &mut TestAppContext,
) -> (
    gpui::Entity<Tall>,
    &mut gpui::VisualTestContext,
    gpui::Bounds<Pixels>,
) {
    cx.update(Theme::init);
    let (view, cx) = cx.add_window_view(|_, _| Tall(0, gpui::ScrollHandle::new(), 2000.));
    cx.simulate_mouse_move(point(px(150.), px(100.)), None, Modifiers::none());
    let thumb = cx.debug_bounds("scrollbar-thumb").expect("shown on hover");
    (view, cx, thumb)
}

#[gpui::test]
fn a_key_with_the_pointer_on_the_bar_keeps_it(cx: &mut TestAppContext) {
    let (_, cx, thumb) = hovered(cx);
    cx.simulate_mouse_move(thumb.center(), None, Modifiers::none());
    cx.simulate_keystrokes("ctrl-s");
    assert!(
        cx.debug_bounds("scrollbar-thumb").is_some(),
        "gone after a key"
    );
}

#[gpui::test]
fn a_thumb_drags_without_a_scroll_first(cx: &mut TestAppContext) {
    let (view, cx, thumb) = hovered(cx);
    let grip = thumb.center();
    cx.simulate_mouse_move(grip, None, Modifiers::none());
    cx.simulate_mouse_down(grip, MouseButton::Left, Modifiers::none());
    for down in [10., 20., 80.] {
        let at = grip + point(px(0.), px(down));
        cx.simulate_mouse_move(at, MouseButton::Left, Modifiers::none());
        assert!(
            cx.debug_bounds("scrollbar-thumb").is_some(),
            "gone at {down}"
        );
    }
    let offset = cx.update(|_, app| view.read(app).1.offset().y);
    assert!(offset < px(0.), "the drag scrolled to {offset:?}");
    cx.simulate_mouse_up(
        grip + point(px(0.), px(80.)),
        MouseButton::Left,
        Modifiers::none(),
    );
}

#[gpui::test]
fn a_bar_gone_under_the_pointer_lets_go(cx: &mut TestAppContext) {
    let (view, cx, thumb) = hovered(cx);
    cx.simulate_mouse_move(thumb.center(), None, Modifiers::none());
    let tall = |cx: &mut gpui::VisualTestContext, height| {
        cx.update(|_, app| {
            view.update(app, |view, cx| {
                view.2 = height;
                cx.notify();
            })
        });
        cx.run_until_parked();
        // The handle measures at paint, a frame late.
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    };
    tall(cx, 100.);
    assert!(
        cx.debug_bounds("scrollbar-thumb").is_none(),
        "nothing to scroll"
    );
    cx.simulate_mouse_move(point(px(600.), px(400.)), None, Modifiers::none());
    tall(cx, 2000.);
    assert!(
        cx.debug_bounds("scrollbar-thumb").is_none(),
        "held after it went"
    );
}

/// An area scrolling both ways; renders bounded.
struct Wide(usize, f32);

impl Render for Wide {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.0 += 1;
        assert!(self.0 < 100, "the area redraws without end");
        super::ScrollArea::new("wide")
            .w(px(300.))
            .h(px(200.))
            .child(div().w(px(self.1)).h(px(2000.)))
    }
}

#[gpui::test]
fn one_bar_going_keeps_the_other_held(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (view, cx) = cx.add_window_view(|_, _| Wide(0, 2000.));
    cx.simulate_mouse_move(point(px(150.), px(100.)), None, Modifiers::none());
    cx.simulate_mouse_move(point(px(291.), px(10.)), None, Modifiers::none());
    cx.update(|_, app| {
        view.update(app, |view, cx| {
            view.1 = 100.;
            cx.notify();
        })
    });
    // Step by step: a frame without it is a flicker.
    for frame in 0..40 {
        cx.update(|window, _| window.refresh());
        cx.executor().tick();
        let shown = cx.debug_bounds("scrollbar-thumb").is_some();
        assert!(shown, "the down bar went at frame {frame}");
    }
}

#[gpui::test]
fn a_bar_stays_while_the_pointer_rests_on_it(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (_, cx) = cx.add_window_view(|_, _| Tall(0, gpui::ScrollHandle::new(), 2000.));
    cx.simulate_mouse_move(point(px(600.), px(400.)), None, Modifiers::none());
    assert!(
        cx.debug_bounds("scrollbar-thumb").is_none(),
        "hidden outside"
    );
    cx.simulate_mouse_move(point(px(150.), px(100.)), None, Modifiers::none());
    let thumb = cx.debug_bounds("scrollbar-thumb").expect("shown on hover");
    for step in 0..4 {
        let at = thumb.center() + point(px(0.), px(step as f32));
        cx.simulate_mouse_move(at, None, Modifiers::none());
        assert!(
            cx.debug_bounds("scrollbar-thumb").is_some(),
            "gone at move {step}"
        );
    }
    cx.simulate_mouse_move(point(px(600.), px(100.)), None, Modifiers::none());
    assert!(
        cx.debug_bounds("scrollbar-thumb").is_none(),
        "hidden once left"
    );
}

mod hold;
