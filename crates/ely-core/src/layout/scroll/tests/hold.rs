use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    App, Axis, Context, Entity, IntoElement, ParentElement, Pixels, Point, Render, Styled,
    TestAppContext, VisualTestContext, Window, div, point, px,
};

use super::super::{ScrollModel, ScrollPlace, ScrollSource, Scrollbar};
use crate::theme::Theme;

struct Position(Cell<f32>);

impl ScrollModel for Position {
    fn place(&self, _: Axis, _: &App) -> Option<ScrollPlace> {
        Some(ScrollPlace {
            span: px(400.),
            shown: 0.25,
            passed: self.0.get(),
        })
    }

    fn set(&self, _: Axis, ratio: f32, _: &mut App) {
        self.0.set(ratio);
    }

    fn offset(&self, _: &App) -> Point<Pixels> {
        point(px(self.0.get()), px(0.))
    }
}

/// Counts its renders under an auto bar.
struct Host {
    position: Rc<Position>,
    renders: usize,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders += 1;
        div().relative().w(px(400.)).h(px(40.)).child(
            Scrollbar::new(
                "bar",
                ScrollSource::Model(self.position.clone()),
                Axis::Horizontal,
            )
            .auto(),
        )
    }
}

fn hosted(cx: &mut TestAppContext) -> (Rc<Position>, Entity<Host>, &mut VisualTestContext) {
    cx.update(Theme::init);
    cx.update(|cx| Theme::update(cx, |theme| theme.reduced_motion = false));
    let position = Rc::new(Position(Cell::new(0.)));
    let held = position.clone();
    let (host, cx) = cx.add_window_view(move |_, _| Host {
        position: held,
        renders: 0,
    });
    cx.run_until_parked();
    (position, host, cx)
}

/// Scrolls to `to`, then draws the frames the view asks.
fn scroll(to: f32, position: &Position, host: &Entity<Host>, cx: &mut VisualTestContext) {
    position.0.set(to);
    host.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    for _ in 0..8 {
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
    }
}

/// Waits `millis` of real and test time; renders since `from`.
fn after(millis: u64, from: usize, host: &Entity<Host>, cx: &mut VisualTestContext) -> usize {
    std::thread::sleep(Duration::from_millis(millis));
    cx.executor().advance_clock(Duration::from_millis(millis));
    cx.run_until_parked();
    host.read_with(cx, |host, _| host.renders) - from
}

#[gpui::test]
fn one_scroll_wakes_once_when_its_hold_ends(cx: &mut TestAppContext) {
    let (position, host, cx) = hosted(cx);
    scroll(0.4, &position, &host, cx);
    let from = host.read_with(cx, |host, _| host.renders);
    assert_eq!(after(810, from, &host, cx), 0, "the hold draws nothing");
    assert_eq!(after(100, from, &host, cx), 1, "its end wakes once");
}

#[gpui::test]
fn earlier_scrolls_never_wake_the_latest_hold(cx: &mut TestAppContext) {
    let (position, host, cx) = hosted(cx);
    for step in 1..=4 {
        scroll(step as f32 * 0.1, &position, &host, cx);
        if step < 4 {
            after(100, 0, &host, cx);
        }
    }
    let from = host.read_with(cx, |host, _| host.renders);
    let held: Vec<usize> = [610, 100, 100]
        .into_iter()
        .map(|millis| after(millis, from, &host, cx))
        .collect();
    assert_eq!(held, [0, 0, 0], "old wakes stay asleep");
}
