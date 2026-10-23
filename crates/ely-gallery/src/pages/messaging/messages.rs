use ely_gpui_component::{
    buttons::Button,
    chat::DateSeparator,
    collab::{Reaction, Reactions, toggled},
    data_display::Avatar,
    forms::Input,
    messaging::{ChatMessage, Delivery, MessageThread, ReadReceipt, ThreadPanel, UnreadDivider},
    motion::TypingIndicator,
    theme::{ActiveTheme, Radius},
};
use gpui::{App, Entity, IntoElement, ParentElement, Styled, Window, div, px};
use jiff::{SignedDuration, Timestamp, civil::date, tz::TimeZone};

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

/// A story morning, so captures read the same times.
fn at(hour: i8, minute: i8) -> Timestamp {
    date(2026, 9, 26)
        .at(hour, minute, 0, 0)
        .to_zoned(TimeZone::UTC)
        .expect("a UTC time")
        .timestamp()
}

fn message(id: &'static str, author: &'static str, hour: i8, minute: i8) -> ChatMessage {
    ChatMessage::new(id, author, at(hour, minute)).zone(TimeZone::UTC)
}

/// Whether the thread panel shows.
fn thread_open(window: &mut Window, cx: &mut App) -> Entity<bool> {
    keep("messaging-thread-open", || true, window, cx)
}

fn thread_mark(open: Entity<bool>) -> MessageThread {
    MessageThread::new(
        "messaging-thread-mark",
        3,
        Timestamp::now() - SignedDuration::from_hours(2),
    )
    .repliers([
        Avatar::new("messaging-replier-ana", "Ana Lima"),
        Avatar::new("messaging-replier-ben", "Ben Ito"),
    ])
    .on_open(move |_, cx| change(&open, cx, |open| *open = true))
}

pub fn conversation(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let reactions = keep(
        "messaging-reactions",
        || {
            vec![
                Reaction {
                    emoji: "👍".into(),
                    count: 3,
                    mine: false,
                },
                Reaction {
                    emoji: "🌿".into(),
                    count: 1,
                    mine: true,
                },
            ]
        },
        window,
        cx,
    );
    let open = thread_open(window, cx);
    let shown = reactions.read(cx).clone();
    let footer = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            Reactions::new("messaging-reactions", shown).on_toggle(move |emoji, _, cx| {
                change(&reactions, cx, |all| *all = toggled(all, emoji))
            }),
        )
        .child(thread_mark(open));
    section(
        "ChatMessage / UnreadDivider / MessageReactions / TypingIndicator",
        "Messages under their day: one author's messages within five minutes form a run, the head only on the first and the time in the gutter on hover. A rule marks where unread ones start. Reactions and a thread sit under a message, and a line says who is typing.",
        cx,
    )
    .child(probe(
        "messaging-conversation",
        div()
            .w(px(640.))
            .flex()
            .flex_col()
            .child(DateSeparator::new(date(2026, 9, 26), date(2026, 9, 26)))
            .child(
                message("messaging-m1", "Ana Lima", 9, 2)
                    .child("Morning. The plaster samples came in overnight."),
            )
            .child(
                message("messaging-m2", "Ana Lima", 9, 3)
                    .after("Ana Lima", at(9, 2))
                    .child("Three finishes: lime, tadelakt and a sanded gypsum."),
            )
            .child(
                message("messaging-m3", "Ana Lima", 9, 5)
                    .after("Ana Lima", at(9, 3))
                    .child("They're in the stair hall, along the left wall."),
            )
            .child(
                message("messaging-m4", "Ben Ito", 9, 7)
                    .after("Ana Lima", at(9, 5))
                    .child("I'll look before the review. Is the tadelakt sealed yet?"),
            )
            .child(
                message("messaging-m5", "Ana Lima", 9, 21)
                    .after("Ben Ito", at(9, 7))
                    .child("Not yet. It needs another day."),
            )
            .child(UnreadDivider::new())
            .child(
                message("messaging-m6", "Chloé Martin", 9, 34)
                    .after("Ana Lima", at(9, 21))
                    .child("Pinned the brief: light from above, one stair, olive trees by the windows. Thoughts on where the trees go?")
                    .footer(footer),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .child(TypingIndicator::new("messaging-typing").who("Dev Rao")),
            ),
    ))
}

pub fn thread(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = thread_open(window, cx);
    let reply = window.use_keyed_state("messaging-thread-reply", cx, |window, cx| {
        ely_gpui_component::forms::TextInput::new(window, cx).placeholder("Reply…")
    });
    let shown = *open.read(cx);
    let theme = cx.theme();
    let frame = div()
        .w(px(360.))
        .h(px(480.))
        .border_1()
        .border_color(theme.colors.border)
        .rounded(theme.radius(Radius::Lg));
    let body = if shown {
        let closing = open.clone();
        frame.child(
            ThreadPanel::new(
                "messaging-thread",
                message("messaging-t0", "Chloé Martin", 9, 34)
                    .child("Pinned the brief: light from above, one stair, olive trees by the windows. Thoughts on where the trees go?"),
            )
            .chat("# design")
            .child(
                message("messaging-t1", "Ana Lima", 9, 40)
                    .child("By the south windows, where the stair takes its light."),
            )
            .child(
                message("messaging-t2", "Ana Lima", 9, 41)
                    .after("Ana Lima", at(9, 40))
                    .child("Two there, and one on the landing."),
            )
            .child(
                message("messaging-t3", "Ben Ito", 10, 12)
                    .after("Ana Lima", at(9, 41))
                    .child("Agreed. I'll move them on the plan."),
            )
            .composer(Input::new(&reply))
            .on_close(move |_, cx| change(&closing, cx, |open| *open = false)),
        )
    } else {
        let opening = open.clone();
        frame.flex().items_center().justify_center().child(
            Button::new("messaging-thread-reopen", "Open the thread")
                .on_click(move |_, _, cx| change(&opening, cx, |open| *open = true)),
        )
    };
    section(
        "MessageThread / ThreadPanel",
        "A thread's mark under its message: who replied, how many, and when the last came; a press opens the thread. The panel holds the first message, a rule counting the replies, the replies in their runs, and a composer at the bottom.",
        cx,
    )
    .child(probe("messaging-thread", body))
}

pub fn receipts(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "ReadReceipt",
        "Where a sent message stands: a clock while it goes, one tick once sent, two once delivered, two in the info tone once read. Read in a group, it shows who saw it.",
        cx,
    )
    .child(probe(
        "messaging-receipts",
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(ReadReceipt::new(Delivery::Sending))
            .child(ReadReceipt::new(Delivery::Sent))
            .child(ReadReceipt::new(Delivery::Delivered))
            .child(ReadReceipt::new(Delivery::Read))
            .child(ReadReceipt::new(Delivery::Read).readers([
                Avatar::new("messaging-seen-ana", "Ana Lima"),
                Avatar::new("messaging-seen-ben", "Ben Ito"),
                Avatar::new("messaging-seen-chloe", "Chloé Martin"),
                Avatar::new("messaging-seen-dev", "Dev Rao"),
            ])),
    ))
}
