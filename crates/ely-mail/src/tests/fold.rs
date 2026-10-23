//! Folding sections, and a right press on a box.

use gpui::{Entity, IntoElement, Modifiers, MouseButton, TestAppContext};

use super::{Mailing, heard, mailing, note, press, tab_to};
use crate::{
    primitives::IconName,
    {Mailbox, MailboxList},
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

#[gpui::test]
fn the_menu_key_or_shift_f10_opens_the_current_boxs_menu(cx: &mut TestAppContext) {
    let (host, cx) = mailing(|owner| list(owner, &[]), cx);
    tab_to(4, cx);
    press("menu", cx);
    press("shift-f10", cx);
    press("f10", cx);
    assert_eq!(heard(&host, cx), ["menu clients", "menu clients"]);
}

#[gpui::test]
fn the_menu_key_opens_at_the_row(cx: &mut TestAppContext) {
    let (host, cx) = mailing(
        |owner| {
            MailboxList::new("boxes")
                .section(
                    "Mailboxes",
                    [Mailbox::new("inbox", "Inbox", IconName::Inbox)],
                )
                .open("inbox")
                .on_menu(move |key, at, _, cx| note(&owner, format!("{key} {at:?}"), cx))
                .into_any_element()
        },
        cx,
    );
    let row = cx
        .debug_bounds("item-selected-Inbox")
        .expect("the open box");
    tab_to(1, cx);
    press("menu", cx);
    assert_eq!(heard(&host, cx), [format!("inbox {:?}", row.bottom_left())]);
}

#[gpui::test]
fn a_fixed_section_shows_no_fold(cx: &mut TestAppContext) {
    let (_, cx) = mailing(
        |owner| {
            MailboxList::new("boxes")
                .fixed_section("Unified", [Mailbox::new("all", "All", IconName::Inbox)])
                .section(
                    "ann@example.com",
                    [Mailbox::new("inbox", "Inbox", IconName::Inbox)],
                )
                .on_fold(move |title, _, _, cx| note(&owner, format!("fold {title}"), cx))
                .into_any_element()
        },
        cx,
    );
    assert!(cx.debug_bounds("fold Unified").is_none());
    assert!(cx.debug_bounds("fold ann@example.com").is_some());
}
