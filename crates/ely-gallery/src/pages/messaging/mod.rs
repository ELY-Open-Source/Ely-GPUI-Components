use ely_gpui_component::{
    data_display::Presence,
    messaging::{
        ChannelHeader, ChannelItem, ChannelList, DirectMessageItem, PinnedMessage, PinnedMessages,
    },
};
use gpui::{AnyElement, App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{SignedDuration, Timestamp};

mod calls;
mod messages;
mod people;
mod pickers;

use super::Page;
use crate::{
    probe::probe,
    step::Step,
    ui::{change, keep, section},
};

pub const PAGE: Page = Page {
    number: 28,
    slug: "messaging",
    title: "Messaging",
    summary: "Channels and people: the lists and heads of chats, the messages in them, the people behind them, and calls.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("messaging-channels", 120.0, 150.0),
    Step::UpAt("messaging-channels", 120.0, 150.0),
    Step::Key("down"),
    Step::Wait(200),
    Step::Shot("channel-opened"),
    Step::HoverAt("messaging-conversation", 200.0, 89.0),
    Step::Wait(200),
    Step::Shot("run-time"),
    Step::DownAt("messaging-members", 200.0, 91.0),
    Step::UpAt("messaging-members", 200.0, 91.0),
    Step::Wait(200),
    Step::Shot("member-chosen"),
    Step::DownAt("messaging-status", 35.0, 48.0),
    Step::UpAt("messaging-status", 35.0, 48.0),
    Step::Wait(200),
    Step::Key("tab"),
    Step::Type("palm tree"),
    Step::Wait(200),
    Step::Shot("status-emoji"),
    Step::Key("enter"),
    Step::Key("tab"),
    Step::Type("On leave till Monday"),
    Step::Wait(200),
    Step::Shot("status-draft"),
    Step::DownAt("messaging-call", 73.0, 112.0),
    Step::UpAt("messaging-call", 73.0, 112.0),
    Step::HoverAt("messaging-share", 10.0, 10.0),
    Step::Wait(200),
    Step::Shot("presenting"),
    Step::DownAt("messaging-incoming", 24.0, 14.0),
    Step::UpAt("messaging-incoming", 24.0, 14.0),
    Step::Wait(300),
    Step::Shot("incoming-call"),
    Step::Key("escape"),
    Step::Wait(200),
    Step::DownAt("messaging-stickers", 96.0, 105.0),
    Step::UpAt("messaging-stickers", 96.0, 105.0),
    Step::Wait(200),
    Step::Shot("sticker-sent"),
    Step::DownAt("messaging-gifs", 127.0, 48.0),
    Step::UpAt("messaging-gifs", 127.0, 48.0),
    Step::Wait(300),
    Step::DownAt("messaging-gifs", 78.0, 120.0),
    Step::UpAt("messaging-gifs", 78.0, 120.0),
    Step::Wait(300),
    Step::Shot("gif-sent"),
];

/// The demo's chats: which is open and what waits unread in each.
#[derive(Clone)]
struct Chats {
    open: SharedString,
    unread: Vec<(SharedString, u32)>,
}

impl Chats {
    fn unread(&self, key: &str) -> u32 {
        self.unread
            .iter()
            .find(|(each, _)| each.as_ref() == key)
            .map_or(0, |(_, count)| *count)
    }
}

fn channels(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chats = keep(
        "messaging-chats",
        || Chats {
            open: "design".into(),
            unread: vec![
                ("atrium".into(), 4),
                ("general".into(), 12),
                ("random".into(), 31),
                ("ben".into(), 2),
            ],
        },
        window,
        cx,
    );
    let now = chats.read(cx).clone();
    let channel = |key: &'static str| ChannelItem::new(key, key).unread(now.unread(key));
    let open = chats.clone();
    section(
        "ChannelList / ChatList",
        "Channels and people in sections, each a list: a press, an arrow or Enter opens a chat and clears what waited. Unread names stand out; a muted channel keeps quiet.",
        cx,
    )
    .child(probe(
        "messaging-channels",
        div().w(px(260.)).child(
            ChannelList::new("messaging-channels")
                .section("Starred", [("atrium", channel("atrium").pinned(true))])
                .section(
                    "Channels",
                    [
                        ("design", channel("design")),
                        ("general", channel("general")),
                        ("launch", channel("launch").private()),
                        ("random", channel("random").muted(true)),
                    ],
                )
                .section(
                    "Direct messages",
                    [
                        ("ana", DirectMessageItem::new("ana", "Ana Lima", Presence::Online)),
                        ("ben", DirectMessageItem::new("ben", "Ben Ito", Presence::Away).unread(now.unread("ben"))),
                        ("chloe", DirectMessageItem::new("chloe", "Chloé Martin", Presence::Busy)),
                        ("dev", DirectMessageItem::new("dev", "Dev Rao", Presence::Offline)),
                    ],
                )
                .open(now.open)
                .on_open(move |key, _, cx| {
                    change(&open, cx, |chats| {
                        chats.open = key.clone();
                        chats.unread.retain(|(each, _)| each != key);
                    })
                }),
        ),
    ))
}

fn items(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "ChannelItem / DirectMessageItem",
        "A channel by its mark, a hash or a lock, with its unread count, quiet when muted, pinned with a pin; a person with their presence on their initials.",
        cx,
    )
    .child(probe(
        "messaging-items",
        div()
            .w(px(520.))
            .flex()
            .gap_6()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(ChannelItem::new("item-unread", "design").unread(3))
                    .child(ChannelItem::new("item-private", "launch").private())
                    .child(ChannelItem::new("item-muted", "random").unread(31).muted(true))
                    .child(ChannelItem::new("item-pinned", "atrium").pinned(true)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(DirectMessageItem::new("dm-on", "Ana Lima", Presence::Online).unread(1))
                    .child(DirectMessageItem::new("dm-away", "Ben Ito", Presence::Away))
                    .child(DirectMessageItem::new("dm-busy", "Chloé Martin", Presence::Busy))
                    .child(DirectMessageItem::new("dm-off", "Dev Rao", Presence::Offline)),
            ),
    ))
}

fn header(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "ChannelHeader",
        "A channel's name and topic, and beside them its members, its pins and a call; the actions drop below on a narrow window.",
        cx,
    )
    .child(probe(
        "messaging-header",
        div().w(px(640.)).child(
            ChannelHeader::new("messaging-header", "design")
                .topic("Where the atrium takes its shape: drawings, samples and the week's reviews")
                .members(12, |_, _| log::info!("gallery: members"))
                .pins(3, |_, _| log::info!("gallery: pins"))
                .on_call(|_, _| log::info!("gallery: call")),
        ),
    ))
}

fn pins(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let now = Timestamp::now();
    let pin = |key: &str, author: &str, hours: i64, text: &str| PinnedMessage {
        key: SharedString::from(key.to_string()),
        author: SharedString::from(author.to_string()),
        at: now - SignedDuration::from_hours(hours),
        text: SharedString::from(text.to_string()),
    };
    let all = vec![
        pin(
            "review",
            "Ana Lima",
            3,
            "Friday's review moves to 10:00, in the model room.",
        ),
        pin(
            "samples",
            "Ben Ito",
            30,
            "Plaster samples are in the stair hall. Please leave notes on the backs.",
        ),
        pin(
            "brief",
            "Chloé Martin",
            200,
            "The brief, final: light from above, one stair, olive trees by the windows.",
        ),
    ];
    let kept = keep("messaging-pins", move || all, window, cx);
    let shown = kept.read(cx).clone();
    section(
        "PinnedMessages",
        "Messages pinned in a chat, newest first: a press, an arrow or Enter opens one where it stands; Unpin lets one go.",
        cx,
    )
    .child(probe(
        "messaging-pins",
        div().w(px(420.)).child(
            PinnedMessages::new("messaging-pins", shown)
                .on_open(|key, _, _| log::info!("gallery: open {key}"))
                .on_unpin(move |key, _, cx| change(&kept, cx, |all| all.retain(|pin| pin.key != *key))),
        ),
    ))
}

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(channels(window, cx))
        .child(items(cx))
        .child(header(cx))
        .child(pins(window, cx))
        .child(messages::conversation(window, cx))
        .child(messages::thread(window, cx))
        .child(messages::receipts(cx))
        .child(people::members(window, cx))
        .child(people::statuses(cx))
        .child(people::setter(window, cx))
        .child(calls::controls(window, cx))
        .child(calls::grid(window, cx))
        .child(calls::share(window, cx))
        .child(calls::incoming(window, cx))
        .child(pickers::stickers(window, cx))
        .child(pickers::gifs(window, cx))
        .into_any_element()
}
