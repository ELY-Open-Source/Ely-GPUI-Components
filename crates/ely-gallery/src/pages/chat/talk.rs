use std::time::Duration;

use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    chat::{
        ChatContainer, DateSeparator, MessageActions, MessageAvatar, MessageBubble, MessageFooter,
        MessageHeader, MessageList, Role, StreamingCursor, StreamingMarkdown, stop_button,
    },
    primitives::IconName,
    theme::{ActiveTheme, Radius},
};
use gpui::{App, ElementId, IntoElement, ParentElement, Styled, Task, Window, div, px};
use jiff::{Timestamp, ToSpan, civil::Date, tz::TimeZone};

use crate::{
    probe::probe,
    ui::{keep, section},
};

/// One row of the demo conversation.
#[derive(Clone)]
enum Row {
    Day(Date),
    Note(&'static str),
    Said {
        role: Role,
        name: &'static str,
        minutes_ago: i64,
        text: &'static str,
        live: bool,
    },
}

const ANSWER: &str = "A **lift** blends a color toward white. Half a lift sits midway between the color and white; a full lift is white itself.\n\n- In light, accents take half a lift.\n- In dark, they take more, so they read with the same weight.";

const LIVE: &str = "The lift $t$ mixes a channel toward one:\n\n$$c' = c + t\\,(1 - c)$$\n\nIn code, for each channel:\n\n```rust\nfn lift(c: f32, t: f32) -> f32 {\n    c + t * (1.0 - c)\n}\n```\n\nApplied to every";

fn rows(today: Date) -> Vec<Row> {
    let yesterday = today.yesterday().expect("the day before exists");
    vec![
        Row::Day(yesterday),
        Row::Said {
            role: Role::User,
            name: "Ada Park",
            minutes_ago: 1500,
            text: "What does a lift do to a color?",
            live: false,
        },
        Row::Said {
            role: Role::Assistant,
            name: "Assistant",
            minutes_ago: 1499,
            text: ANSWER,
            live: false,
        },
        Row::Day(today),
        Row::Note("Grace Lin joined the conversation"),
        Row::Said {
            role: Role::User,
            name: "Ada Park",
            minutes_ago: 2,
            text: "Show me the math, and the code.",
            live: false,
        },
        Row::Said {
            role: Role::Assistant,
            name: "Assistant",
            minutes_ago: 1,
            text: LIVE,
            live: true,
        },
    ]
}

fn row(ix: usize, row: &Row, today: Date, now: Timestamp) -> gpui::AnyElement {
    let key = |what: &str| (ElementId::from("chat-row"), format!("{what}-{ix}"));
    let body = match row {
        Row::Day(day) => return DateSeparator::new(*day, today).into_any_element(),
        Row::Note(text) => MessageBubble::new(Role::System).child(*text),
        Row::Said {
            role,
            name,
            minutes_ago,
            text,
            live,
        } => {
            let header = MessageHeader::new(key("header"), *name).at(now - minutes_ago.minutes());
            let header = if *role == Role::Assistant {
                header.model("Ely Large")
            } else {
                header
            };
            let bubble = MessageBubble::new(*role)
                .avatar(MessageAvatar::new(key("avatar"), *role, *name))
                .header(header)
                .child(StreamingMarkdown::new(key("text"), *text, *live));
            match (role, live) {
                (Role::Assistant, false) => bubble.footer(
                    MessageFooter::new().fact("412 tokens").fact("2.1 s").child(
                        MessageActions::new(key("actions"), *text)
                            .on_regenerate(|_, _| log::info!("gallery: regenerate"))
                            .rating(Some(true), |next, _, _| {
                                log::info!("gallery: rated {next:?}")
                            }),
                    ),
                ),
                (Role::Assistant, true) => bubble.footer(MessageFooter::new().child(
                    stop_button(key("stop")).on_click(|_, _, _| log::info!("gallery: stop")),
                )),
                _ => bubble,
            }
        }
    };
    div().px_6().py_3().child(body).into_any_element()
}

pub fn conversation(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let today = Timestamp::now()
        .to_zoned(TimeZone::try_system().expect("the gallery reads the system time zone"))
        .date();
    let now = Timestamp::now();
    let all = rows(today);
    let count = all.len();
    let theme = cx.theme();
    section(
        "ChatContainer / MessageList / MessageBubble / MessageAvatar / MessageHeader / MessageFooter / DateSeparator / ScrollToBottomButton",
        "A conversation held at its newest message and drawn only near the view. Yours sit on the right in a quiet bubble; the assistant's read as prose beside its mark. Scroll up and it stays put, with a way back down that counts what arrived.",
        cx,
    )
    .child(probe(
        "chat-conversation",
        div()
            .w(px(760.))
            .h(px(600.))
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .overflow_hidden()
            .child(ChatContainer::new(MessageList::new("chat-messages", count, move |ix, _, _| {
                row(ix, &all[ix], today, now)
            }))),
    ))
}

/// The demo stream: how much of it has arrived, whether more is coming, and the task that feeds it.
struct Stream {
    arrived: usize,
    live: bool,
    _feed: Option<Task<()>>,
}

pub fn streaming(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let stream = keep(
        "chat-stream",
        || Stream {
            arrived: ANSWER.len(),
            live: false,
            _feed: None,
        },
        window,
        cx,
    );
    let (arrived, live) = (stream.read(cx).arrived, stream.read(cx).live);
    let text = &ANSWER[..arrived];
    let theme = cx.theme();
    let replay = stream.clone();
    section(
        "StreamingText / StreamingMarkdown / StreamingCursor",
        "Text as it arrives: what came in shows at a steady pace and catches up when far behind, a caret where more will come; markdown takes shape as its lines land. The cursor alone waits for the first words.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .p_4()
                    .rounded(theme.radius(Radius::Lg))
                    .border_1()
                    .border_color(theme.colors.border)
                    .min_h(px(140.))
                    .child(if arrived == 0 {
                        StreamingCursor::new("chat-cursor").into_any_element()
                    } else {
                        StreamingMarkdown::new("chat-streamed", text.to_string(), live).into_any_element()
                    }),
            )
            .child(
                div().flex().child(
                    Button::new("chat-replay", "Replay the stream")
                        .variant(ButtonVariant::Secondary)
                        .icon(IconName::RefreshCw)
                        .on_click(move |_, window, cx| {
                            let feeder = replay.downgrade();
                            let feed = window.spawn(cx, async move |cx| {
                                cx.background_executor().timer(Duration::from_millis(600)).await;
                                loop {
                                    cx.background_executor().timer(Duration::from_millis(45)).await;
                                    let more = cx.update(|_, cx| feeder.update(cx, |stream, cx| {
                                        let next = (stream.arrived + 7).min(ANSWER.len());
                                        stream.arrived = (next..=ANSWER.len())
                                            .find(|at| ANSWER.is_char_boundary(*at))
                                            .expect("the text ends on a boundary");
                                        stream.live = stream.arrived < ANSWER.len();
                                        cx.notify();
                                        stream.live
                                    }));
                                    if !matches!(more.and_then(|inner| inner), Ok(true)) {
                                        break;
                                    }
                                }
                            });
                            replay.update(cx, |stream, cx| {
                                stream.arrived = 0;
                                stream.live = true;
                                stream._feed = Some(feed);
                                cx.notify();
                            });
                        }),
                ),
            ),
    )
}
