use gpui::{
    Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    TestAppContext, VisualTestContext, Window,
};
use jiff::{Timestamp, tz::TimeZone};

use super::{Mailing, heard, mailing, note, press, settle, tab_to};
use crate::{
    forms,
    mail::{Contact, MailComposer, ScheduleSend, SignatureEditor},
};

fn contacts() -> [Contact; 2] {
    [
        Contact::new("Ana Lima", "ana@atrium.studio"),
        Contact::new("Ben Ito", "ben@atrium.studio"),
    ]
}

/// The owner's words; not a Tab stop.
fn words() -> gpui::Div {
    gpui::div()
        .debug_selector(|| "words".into())
        .child("They are dry.")
}

fn composer(owner: Entity<Mailing>) -> gpui::AnyElement {
    let (send, discard) = (owner.clone(), owner);
    MailComposer::new("composer", words())
        .contacts(contacts())
        .on_send(move |draft, _, cx| {
            note(
                &send,
                format!("send {} | {}", draft.to.join(","), draft.subject),
                cx,
            )
        })
        .on_discard(move |_, cx| note(&discard, "discard".into(), cx))
        .into_any_element()
}

fn writing(
    part: fn(Entity<Mailing>) -> gpui::AnyElement,
    cx: &mut TestAppContext,
) -> (Entity<Mailing>, &mut VisualTestContext) {
    let (host, cx) = mailing(part, cx);
    cx.update(|_, cx| forms::bind_keys(cx));
    (host, cx)
}

fn next(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.focus_next(cx));
    settle(cx);
}

#[gpui::test]
fn a_suggested_person_and_a_subject_go_out_as_one_draft(cx: &mut TestAppContext) {
    let (host, cx) = writing(composer, cx);
    assert!(cx.debug_bounds("words").is_some(), "the owner's words show");
    tab_to(1, cx);
    cx.simulate_input("be");
    settle(cx);
    press("enter", cx);
    next(cx);
    next(cx);
    cx.simulate_input("Samples");
    next(cx);
    press("space", cx);
    assert_eq!(heard(&host, cx), ["send ben@atrium.studio | Samples"]);
}

#[gpui::test]
fn cc_and_bcc_show_once_asked(cx: &mut TestAppContext) {
    let (_, cx) = writing(composer, cx);
    assert!(cx.debug_bounds("composer-cc").is_none());
    tab_to(2, cx);
    press("space", cx);
    assert!(cx.debug_bounds("composer-cc").is_some());
    assert!(cx.debug_bounds("composer-bcc").is_some());
}

#[gpui::test]
fn with_nobody_to_send_to_send_steps_aside_for_discard(cx: &mut TestAppContext) {
    let (host, cx) = writing(composer, cx);
    tab_to(4, cx);
    press("space", cx);
    assert_eq!(
        heard(&host, cx),
        ["discard"],
        "Send is off, so the stop after the subject is Discard"
    );
}

fn scheduling(owner: Entity<Mailing>) -> gpui::AnyElement {
    let (send, later) = (owner.clone(), owner);
    ScheduleSend::new("send")
        .zone(TimeZone::UTC)
        .on_send(move |_, cx| note(&send, "send".into(), cx))
        .on_schedule(move |at, _, cx| note(&later, format!("at {at}"), cx))
        .into_any_element()
}

#[gpui::test]
fn the_first_time_to_send_later_is_tomorrow_morning(cx: &mut TestAppContext) {
    let (host, cx) = writing(scheduling, cx);
    tab_to(2, cx);
    press("enter", cx);
    press("enter", cx);
    let morning = Timestamp::now()
        .to_zoned(TimeZone::UTC)
        .date()
        .tomorrow()
        .expect("a day follows")
        .at(8, 0, 0, 0)
        .to_zoned(TimeZone::UTC)
        .expect("a UTC time")
        .timestamp();
    assert_eq!(heard(&host, cx), [format!("at {morning}")]);
}

/// A signature the host keeps, and every change it heard.
struct Signing {
    words: SharedString,
    heard: Vec<String>,
}

impl Render for Signing {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        SignatureEditor::new("signature", self.words.clone()).on_change(move |words, _, cx| {
            owner.update(cx, |host, cx| {
                host.heard.push(words.to_string());
                host.words = words.clone();
                cx.notify();
            })
        })
    }
}

fn signing<'a>(
    words: &str,
    cx: &'a mut TestAppContext,
) -> (Entity<Signing>, &'a mut VisualTestContext) {
    super::setup(cx);
    cx.update(forms::bind_keys);
    let (host, cx) = cx.add_window_view(|_, _| Signing {
        words: SharedString::from(words.to_string()),
        heard: Vec::new(),
    });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    (host, cx)
}

#[gpui::test]
fn an_echo_keeps_the_caret_where_it_is(cx: &mut TestAppContext) {
    let (host, cx) = signing("", cx);
    tab_to(1, cx);
    cx.simulate_input("ab");
    settle(cx);
    press("left", cx);
    cx.simulate_input("X");
    settle(cx);
    cx.simulate_input("Y");
    settle(cx);
    let heard = host.read_with(cx, |host, _| host.heard.clone());
    assert_eq!(
        heard.last().map(String::as_str),
        Some("aXYb"),
        "the caret stayed after X"
    );
}

#[gpui::test]
fn a_new_signature_from_the_owner_starts_the_field_over(cx: &mut TestAppContext) {
    let (host, cx) = signing("Ana", cx);
    host.update(cx, |host, cx| {
        host.words = "Ben Ito".into();
        cx.notify();
    });
    settle(cx);
    tab_to(1, cx);
    cx.simulate_input("!");
    settle(cx);
    let heard = host.read_with(cx, |host, _| host.heard.clone());
    assert_eq!(heard, ["Ben Ito!"]);
}

/// A composer the host starts from addresses, and what it sent.
struct Replying {
    to: SharedString,
    cc: Vec<SharedString>,
    bcc: Vec<SharedString>,
    sending: bool,
    check: fn(&str) -> bool,
    sent: Vec<String>,
    blind: Vec<String>,
    changed: Vec<String>,
}

fn line(draft: &crate::mail::Draft) -> String {
    format!(
        "{} | {} | {}",
        draft.to.join(","),
        draft.cc.join(","),
        draft.subject
    )
}

impl Render for Replying {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (owner, watcher) = (cx.entity(), cx.entity());
        MailComposer::new("composer", words())
            .to([self.to.clone()])
            .cc(self.cc.clone())
            .bcc(self.bcc.clone())
            .subject("Re: Samples")
            .busy(self.sending)
            .check(self.check)
            .on_change(move |draft, _, cx| {
                watcher.update(cx, |host, _| host.changed.push(line(draft)))
            })
            .on_send(move |draft, _, cx| {
                owner.update(cx, |host, _| {
                    host.sent.push(line(draft));
                    host.blind.push(draft.bcc.join(","));
                })
            })
            .on_discard(|_, _| {})
    }
}

fn replying<'a>(
    cc: &[&str],
    sending: bool,
    cx: &'a mut TestAppContext,
) -> (Entity<Replying>, &'a mut VisualTestContext) {
    super::setup(cx);
    cx.update(forms::bind_keys);
    let cc = cc
        .iter()
        .map(|address| SharedString::from(address.to_string()));
    let (host, cx) = cx.add_window_view(|_, _| Replying {
        to: "ana@atrium.studio".into(),
        cc: cc.collect(),
        bcc: Vec::new(),
        sending,
        check: forms::is_email,
        sent: Vec::new(),
        blind: Vec::new(),
        changed: Vec::new(),
    });
    cx.update(|window, _| window.activate_window());
    settle(cx);
    (host, cx)
}

#[gpui::test]
fn a_new_start_from_the_owner_begins_the_draft_over(cx: &mut TestAppContext) {
    let (host, cx) = replying(&[], false, cx);
    host.update(cx, |host, cx| {
        host.to = "ben@atrium.studio".into();
        cx.notify();
    });
    settle(cx);
    tab_to(4, cx);
    press("space", cx);
    assert_eq!(
        host.read_with(cx, |host, _| host.sent.clone()),
        ["ben@atrium.studio |  | Re: Samples"]
    );
}

#[gpui::test]
fn copies_given_show_and_go_out(cx: &mut TestAppContext) {
    let (host, cx) = replying(&["cy@atrium.studio"], false, cx);
    assert!(cx.debug_bounds("composer-cc").is_some());
    assert!(cx.debug_bounds("composer-bcc").is_some());
    // To, Cc, Bcc, the subject, then Send.
    tab_to(5, cx);
    press("space", cx);
    assert_eq!(
        host.read_with(cx, |host, _| host.sent.clone()),
        ["ana@atrium.studio | cy@atrium.studio | Re: Samples"]
    );
}

#[gpui::test]
fn blind_copies_given_show_and_go_out(cx: &mut TestAppContext) {
    let (host, cx) = replying(&[], false, cx);
    host.update(cx, |host, cx| {
        host.bcc = vec!["dee@atrium.studio".into()];
        cx.notify();
    });
    settle(cx);
    assert!(cx.debug_bounds("composer-bcc").is_some());
    tab_to(5, cx);
    press("space", cx);
    assert_eq!(
        host.read_with(cx, |host, _| host.blind.clone()),
        ["dee@atrium.studio"]
    );
}

#[gpui::test]
fn while_sending_send_waits(cx: &mut TestAppContext) {
    let (host, cx) = replying(&[], true, cx);
    tab_to(4, cx);
    press("space", cx);
    assert!(host.read_with(cx, |host, _| host.sent.is_empty()));
}

fn sent(host: &Entity<Replying>, cx: &mut VisualTestContext) -> Vec<String> {
    host.read_with(cx, |host, _| host.sent.clone())
}

#[gpui::test]
fn an_address_typed_but_not_added_still_goes(cx: &mut TestAppContext) {
    let (host, cx) = replying(&["cy@atrium.studio"], false, cx);
    tab_to(2, cx);
    cx.simulate_input("bob@atrium.studio");
    settle(cx);
    tab_to(5, cx);
    press("space", cx);
    assert_eq!(
        sent(&host, cx),
        ["ana@atrium.studio | cy@atrium.studio,bob@atrium.studio | Re: Samples"]
    );
}

#[gpui::test]
fn typed_words_that_are_no_address_hold_send(cx: &mut TestAppContext) {
    let (host, cx) = replying(&[], false, cx);
    tab_to(1, cx);
    cx.simulate_input("bob@");
    settle(cx);
    tab_to(4, cx);
    assert!(
        cx.debug_bounds("tag-input-unaccepted").is_some(),
        "To says why"
    );
    press("space", cx);
    assert!(sent(&host, cx).is_empty(), "Send stays off");
}

#[gpui::test]
fn the_owner_says_what_an_address_is(cx: &mut TestAppContext) {
    let (host, cx) = replying(&[], false, cx);
    host.update(cx, |host, cx| {
        host.to = "\"first last\"@atrium.studio".into();
        host.check = |text| text.ends_with("@atrium.studio");
        cx.notify();
    });
    settle(cx);
    tab_to(4, cx);
    press("space", cx);
    assert_eq!(
        sent(&host, cx),
        ["\"first last\"@atrium.studio |  | Re: Samples"]
    );
}

#[gpui::test]
fn edits_reach_the_owner_as_drafts(cx: &mut TestAppContext) {
    let (host, cx) = replying(&[], false, cx);
    tab_to(1, cx);
    cx.simulate_input("bo");
    settle(cx);
    tab_to(3, cx);
    cx.simulate_input("!");
    settle(cx);
    let changed = host.read_with(cx, |host, _| host.changed.clone());
    assert!(changed.contains(&"ana@atrium.studio,bo |  | Re: Samples".to_owned()));
    assert_eq!(
        changed.last().map(String::as_str),
        Some("ana@atrium.studio,bo |  | Re: Samples!")
    );
}

#[gpui::test]
fn a_quoted_comma_typed_key_by_key_still_sends(cx: &mut TestAppContext) {
    let (host, cx) = replying(&[], false, cx);
    host.update(cx, |host, cx| {
        host.check = |text| text.ends_with("@atrium.studio");
        cx.notify();
    });
    settle(cx);
    tab_to(1, cx);
    for ch in "\"last, first\"@atrium.studio".chars() {
        cx.simulate_input(&ch.to_string());
        settle(cx);
    }
    tab_to(4, cx);
    press("space", cx);
    assert_eq!(
        sent(&host, cx),
        ["ana@atrium.studio,\"last, first\"@atrium.studio |  | Re: Samples"]
    );
}
