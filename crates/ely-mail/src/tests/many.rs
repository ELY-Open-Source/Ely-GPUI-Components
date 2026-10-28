use std::rc::Rc;

use gpui::{
    AnyElement, Entity, IntoElement, Modifiers, MouseButton, ParentElement, Styled, TestAppContext,
    div, point, px,
};
use jiff::Timestamp;

use super::{Mailing, heard, mailing, note, press, tab_to};
use crate::{
    i18n::I18n,
    {
        Mail, MailList,
        item::{When, say},
    },
};

const COUNT: usize = 100_000;

thread_local! {
    static MANY: Rc<Vec<Mail>> = Rc::new(
        (0..COUNT)
            .map(|n| Mail::new(format!("m{n}"), "Ana", format!("Note {n}"), Timestamp::UNIX_EPOCH))
            .collect(),
    );
}

/// A hundred thousand messages; it hears count and first key.
fn many(owner: Entity<Mailing>) -> AnyElement {
    let list = MailList::new("mails", MANY.with(Rc::clone))
        .on_select(move |keys, _, cx| {
            let first = keys.first().map_or("none", |key| key.as_ref());
            note(&owner, format!("{} from {first}", keys.len()), cx)
        })
        .on_star(|_, _, _, _| {});
    div().h(px(480.0)).child(list).into_any_element()
}

#[gpui::test]
fn only_the_rows_in_view_draw(cx: &mut TestAppContext) {
    let (host, cx) = mailing(many, cx);
    assert!(cx.debug_bounds("star m0").is_some(), "the first row draws");
    assert!(
        cx.debug_bounds("star m60").is_none(),
        "a row out of view does not"
    );
    tab_to(1, cx);
    press("end", cx);
    assert!(
        cx.debug_bounds("star m99999").is_some(),
        "End scrolls to the last"
    );
    assert!(
        cx.debug_bounds("star m0").is_none(),
        "the first left the view"
    );
    assert_eq!(heard(&host, cx), ["1 from m99999"]);
}

#[gpui::test]
fn its_scrollbar_follows_and_drags_the_list(cx: &mut TestAppContext) {
    let (_, cx) = mailing(many, cx);
    // Normal motion: the bar holds, then fades.
    cx.update(|_, cx| crate::theme::Theme::update(cx, |theme| theme.reduced_motion = false));
    tab_to(1, cx);
    // Plain input; only frames the view asked for are drawn.
    cx.simulate_keystrokes("end");
    cx.run_until_parked();
    let asked: Vec<usize> = (0..8)
        .map(|_| {
            let asked = cx.update(|window, cx| window.simulate_next_frame(cx));
            cx.run_until_parked();
            asked
        })
        .collect();
    assert!(asked[0] > 0, "the moved list asks a frame: {asked:?}");
    assert!(
        asked[4..].iter().all(|&n| n == 0),
        "then it rests: {asked:?}"
    );
    // The hold ends by one wake, then the bar fades.
    std::thread::sleep(std::time::Duration::from_millis(950));
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(950));
    cx.run_until_parked();
    let fading: Vec<usize> = (0..8)
        .map(|_| {
            let asked = cx.update(|window, cx| window.simulate_next_frame(cx));
            cx.run_until_parked();
            asked
        })
        .collect();
    assert!(fading[0] > 0, "the wake starts the fade: {fading:?}");
    std::thread::sleep(std::time::Duration::from_millis(400));
    let fading: Vec<usize> = (0..8)
        .map(|_| {
            let asked = cx.update(|window, cx| window.simulate_next_frame(cx));
            cx.run_until_parked();
            asked
        })
        .collect();
    assert!(
        fading[4..].iter().all(|&n| n == 0),
        "the faded bar rests: {fading:?}"
    );
    let thumb = cx
        .debug_bounds("scrollbar-thumb")
        .expect("a scroll shows the thumb");
    assert!(
        thumb.top() > px(400.),
        "the thumb sits at the end: {thumb:?}"
    );
    assert!(cx.debug_bounds("star m99999").is_some());
    let grip = thumb.center();
    cx.simulate_mouse_move(grip, None, Modifiers::none());
    cx.simulate_mouse_down(grip, MouseButton::Left, Modifiers::none());
    for up in [10., 200., 400.] {
        cx.simulate_mouse_move(
            grip - point(px(0.), px(up)),
            MouseButton::Left,
            Modifiers::none(),
        );
    }
    cx.simulate_mouse_up(
        grip - point(px(0.), px(400.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    let moved = cx.debug_bounds("scrollbar-thumb").expect("the thumb stays");
    assert!(
        moved.top() < thumb.top() - px(150.),
        "the thumb follows the pointer: {moved:?}"
    );
    assert!(
        cx.debug_bounds("star m99999").is_none(),
        "the drag scrolls the list"
    );
}

#[gpui::test]
fn shift_extends_and_command_a_takes_all(cx: &mut TestAppContext) {
    let (host, cx) = mailing(many, cx);
    tab_to(1, cx);
    press("down", cx);
    press("shift-down", cx);
    press("shift-down", cx);
    press("cmd-a", cx);
    press("space", cx);
    assert_eq!(
        heard(&host, cx),
        [
            "1 from m1",
            "2 from m1",
            "3 from m1",
            "100000 from m0",
            "1 from m3"
        ]
    );
}

#[gpui::test]
fn dates_read_in_the_readers_language(cx: &mut TestAppContext) {
    cx.update(|cx| {
        assert_eq!(say(When::Weekday(1), cx), "Mon");
        assert_eq!(say(When::Day(10, 19), cx), "Oct 19");
        assert_eq!(say(When::Date(2025, 12, 31), cx), "Dec 31, 2025");
        cx.set_global(I18n::new("zh-CN"));
        assert_eq!(say(When::Weekday(7), cx), "周日");
        assert_eq!(say(When::Day(10, 19), cx), "10月19日");
        assert_eq!(say(When::Date(2025, 12, 31), cx), "2025年12月31日");
        assert_eq!(say(When::Time("09:05".into()), cx), "09:05");
    });
}
