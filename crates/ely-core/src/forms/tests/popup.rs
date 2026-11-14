use gpui::{
    Context, IntoElement, KeyUpEvent, Keystroke, Modifiers, ParentElement, Render, ScrollDelta,
    ScrollEvent, ScrollHandle, Styled, TestAppContext, TouchPhase, Window, div, point, prelude::*,
    px,
};
use jiff::civil::{Date, date};

use super::setup;
use crate::forms::{Choice, DatePicker, Select};

/// A long select at the top of a page that scrolls.
struct Page(ScrollHandle);

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let choices = (0..40).map(|ix| Choice::new(format!("{ix}"), format!("Choice {ix}")));
        div()
            .id("page")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.0)
            .child(div().w(px(200.0)).child(Select::new("long", choices)))
            .child(div().h(px(2000.0)))
    }
}

#[gpui::test]
fn a_wheel_over_an_open_list_leaves_the_page(cx: &mut TestAppContext) {
    setup(cx);
    let page = ScrollHandle::new();
    let (_, cx) = cx.add_window_view({
        let page = page.clone();
        |_, _| Page(page)
    });
    cx.run_until_parked();
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("enter");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
    });
    cx.run_until_parked();
    let list = cx.debug_bounds("option-list").expect("the list opens");
    cx.simulate_event(ScrollEvent {
        position: list.center(),
        delta: ScrollDelta::Pixels(point(px(0.0), px(-120.0))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
        source: gpui::ScrollSource::Pointer,
    });
    cx.run_until_parked();
    assert_eq!(page.offset().y, px(0.0), "the page stayed");
}

/// A date picker above a wide area that counts its clicks.
struct PickerOverArea {
    picked: Option<Date>,
    below: usize,
}

impl Render for PickerOverArea {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (picker, area) = (cx.entity(), cx.entity());
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                DatePicker::new("date", self.picked)
                    .today(date(2026, 11, 14))
                    .on_change(move |day, _, cx| {
                        picker.update(cx, |view, _| view.picked = Some(day))
                    }),
            )
            .child(
                div()
                    .id("below")
                    .debug_selector(|| "date-below".into())
                    .w(px(600.0))
                    .h(px(600.0))
                    .on_click(move |_, _, cx| area.update(cx, |view, _| view.below += 1)),
            )
    }
}

#[gpui::test]
fn a_click_in_a_date_popup_leaves_what_lies_below(cx: &mut TestAppContext) {
    setup(cx);
    let (view, cx) = cx.add_window_view(|_, _| PickerOverArea {
        picked: None,
        below: 0,
    });
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("enter");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
    });
    cx.run_until_parked();
    let popup = cx.debug_bounds("date-popup").expect("the calendar opens");
    let below = cx.debug_bounds("date-below").expect("the area draws");
    assert!(
        below.contains(&popup.center()),
        "the calendar covers the area"
    );
    cx.simulate_click(popup.center(), Modifiers::none());
    cx.run_until_parked();
    let (picked, below) = view.read_with(cx, |view, _| (view.picked, view.below));
    assert!(picked.is_some(), "the click picked a day");
    assert_eq!(below, 0, "the area below took no click");
}
