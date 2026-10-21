//! Rows fill the list, so their ends line up.

use std::rc::Rc;

use gpui::{AnyElement, Entity, IntoElement, ParentElement, Styled, TestAppContext, div, px};
use jiff::Timestamp;

use super::{Mailing, mailing};
use crate::mail::{Mail, MailList};

fn two(_: Entity<Mailing>) -> AnyElement {
    let long = "A subject far longer than any row can hold, ".repeat(4);
    let mails = vec![
        Mail::new("short", "Ana", "Hi", Timestamp::now()),
        Mail::new("long", "Bartholomew", long, Timestamp::UNIX_EPOCH),
    ];
    let list = MailList::new("mails", Rc::new(mails)).on_star(|_, _, _, _| {});
    div()
        .w(px(600.0))
        .h(px(480.0))
        .child(list)
        .into_any_element()
}

#[gpui::test]
fn every_row_ends_at_the_lists_edge(cx: &mut TestAppContext) {
    let (_, cx) = mailing(two, cx);
    let short = cx.debug_bounds("star short").expect("the short row's star");
    let long = cx.debug_bounds("star long").expect("the long row's star");
    assert_eq!(short.right(), long.right(), "{short:?} and {long:?}");
    assert!(short.right() > px(560.0), "{short:?} in 600");
    assert!(long.right() <= px(600.0), "{long:?} past the list");
}
