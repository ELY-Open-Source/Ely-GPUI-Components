//! Folding sections, and a right press on a box.

use gpui::{Entity, IntoElement, Modifiers, MouseButton, TestAppContext};

use super::{Mailing, heard, mailing, note, press, tab_to};
use crate::{
    mail::{Mailbox, MailboxList},
    primitives::IconName,
};

fn list(owner: Entity<Mailing>, folded: &[&str]) -> gpui::AnyElement {
    let (fold, menu) = (owner.clone(), owner);
    MailboxList::new("boxes")
        .section(
            "Mailboxes",
            [Mailbox::new("inbox", "Inbox", IconName::Inbox)],
        )
        .section(
            "Labels",
            [
                Mailbox::label("atrium", "Atrium", 0).unread(2),
                Mailbox::label("clients", "Clients", 3).unread(3),
            ],
        )
        .open("clients")
        .folded(folded.iter().map(|title| title.to_string()))
        .on_fold(move |title, folded, _, cx| note(&fold, format!("fold {title} {folded}"), cx))
        .on_menu(move |key, _, _, cx| note(&menu, format!("menu {key}"), cx))
        .into_any_element()
}

#[gpui::test]
fn a_folded_section_hides_its_boxes_and_sums_their_mail(cx: &mut TestAppContext) {
    let (_, cx) = mailing(|owner| list(owner, &["Labels"]), cx);
    assert!(cx.debug_bounds("item-selected-Clients").is_none());
    assert!(cx.debug_bounds("folded-unread Labels").is_some());
}

#[gpui::test]
fn an_open_section_shows_its_boxes(cx: &mut TestAppContext) {
    let (_, cx) = mailing(|owner| list(owner, &[]), cx);
    assert!(cx.debug_bounds("item-selected-Clients").is_some());
    assert!(cx.debug_bounds("folded-unread Labels").is_none());
}

#[gpui::test]
fn a_title_press_or_enter_folds_it(cx: &mut TestAppContext) {
    let (host, cx) = mailing(|owner| list(owner, &["Labels"]), cx);
    let title = cx.debug_bounds("fold Mailboxes").expect("a title");
    cx.simulate_click(title.center(), Modifiers::none());
    tab_to(3, cx);
    press("enter", cx);
    assert_eq!(
        heard(&host, cx),
        ["fold Mailboxes true", "fold Labels false"]
    );
}

#[gpui::test]
fn a_right_press_names_the_box(cx: &mut TestAppContext) {
    let (host, cx) = mailing(|owner| list(owner, &[]), cx);
    let row = cx
        .debug_bounds("item-selected-Clients")
        .expect("the open box");
    cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::none());
    cx.simulate_mouse_up(row.center(), MouseButton::Right, Modifiers::none());
    assert_eq!(heard(&host, cx), ["menu clients"]);
}
