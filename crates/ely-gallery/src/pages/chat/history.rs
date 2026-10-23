use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    chat::{
        Conversation, ConversationExport, ConversationList, MessageAvatar, MessageBubble,
        MessageList, Project, ProjectKnowledgePanel, ProjectList, Role, SharedConversationView,
        StreamingMarkdown, new_chat_button,
    },
    feedback::{Toast, Toaster},
    forms::{SearchInput, TextInput},
    overlays::PromptDialog,
    primitives::{IconName, Severity},
    theme::{ActiveTheme, Radius},
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, ToSpan, tz::TimeZone};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

/// The conversations and the one open, shared by the list and the shared view.
pub struct History {
    pub all: Entity<Vec<Conversation>>,
    pub active: Entity<Option<SharedString>>,
}

pub fn history(window: &mut Window, cx: &mut App) -> History {
    History {
        all: keep(
            "chat-conversations",
            || conversations(Timestamp::now()),
            window,
            cx,
        ),
        active: keep(
            "chat-conversation",
            || Some(SharedString::from("lift")),
            window,
            cx,
        ),
    }
}

fn conversations(now: Timestamp) -> Vec<Conversation> {
    [
        ("lift", "What a lift does to a color", 0, false),
        ("gamma", "Gamma and the eye", 3, true),
        ("dark", "Designing the dark theme", 26, false),
        ("type", "A type scale for the library", 30, false),
        ("motion", "Durations that settle", 120, false),
        ("icons", "Choosing an icon set", 200, true),
        ("grid", "Spacing on a rem scale", 400, false),
        ("release", "Release notes for 1.0", 2000, false),
    ]
    .into_iter()
    .map(|(key, title, hours, pinned)| Conversation {
        key: key.into(),
        title: title.into(),
        at: now - (hours as i64).hours(),
        pinned,
    })
    .collect()
}

/// Adds `conversation` at the top unless its key is there, and opens it.
fn open_new(history: &History, conversation: Conversation, cx: &mut App) {
    let key = conversation.key.clone();
    let mut next = history.all.read(cx).clone();
    if !next.iter().any(|item| item.key == key) {
        next.insert(0, conversation);
        set(&history.all, next, cx);
    }
    set(&history.active, Some(key), cx)
}

pub fn conversations_section(
    history: &History,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let today = Timestamp::now()
        .to_zoned(TimeZone::try_system().expect("the gallery reads the system time zone"))
        .date();
    let query = window.use_keyed_state("chat-find-conversation", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Search conversations")
    });
    let naming = window.use_keyed_state("chat-rename-field", cx, |window, cx| {
        TextInput::new(window, cx)
    });
    let renaming = keep("chat-renaming", || None::<SharedString>, window, cx);
    let (now_all, now_active, now_query, now_renaming) = (
        history.all.read(cx).clone(),
        history.active.read(cx).clone(),
        query.read(cx).text().to_string(),
        renaming.read(cx).clone(),
    );
    let (all, active) = (history.all.clone(), history.active.clone());
    let (fresh, picked, pinned, deleted, renamed) = (
        History {
            all: all.clone(),
            active: active.clone(),
        },
        active.clone(),
        all.clone(),
        all.clone(),
        all.clone(),
    );
    let (asked, field, shut, done) = (renaming.clone(), naming.clone(), renaming.clone(), renaming);
    let theme = cx.theme();
    section(
        "ConversationList / ConversationItem / ConversationSearch / NewChatButton",
        "Conversations down a sidebar, pinned first, then by day; the one open marked with its menu, the others show theirs on hover or focus. The search is a forms::SearchInput; the list keeps titles that hold every word.",
        cx,
    )
    .child(probe(
        "chat-history",
        div()
            .w(px(300.))
            .h(px(520.))
            .flex()
            .flex_col()
            .gap_2()
            .p_2()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.sunken)
            .child(new_chat_button("chat-new").on_click(move |_, _, cx| {
                let at = Timestamp::now();
                let key = SharedString::from(format!("new-{}", at.as_millisecond()));
                open_new(&fresh, Conversation { key, title: "New chat".into(), at, pinned: false }, cx)
            }))
            .child(SearchInput::new("chat-find", &query))
            .child(
                div().flex_1().min_h_0().child(
                    ConversationList::new("chat-list", now_all, today)
                        .query(now_query)
                        .active(now_active, move |key, _, cx| set(&picked, Some(key.clone()), cx))
                        .actions(
                            move |key, _, cx| {
                                let title = all.read(cx).iter().find(|item| &item.key == key).map(|item| item.title.clone());
                                let title = title.expect("a listed conversation has a title");
                                field.update(cx, |input, cx| input.set_text(title, cx));
                                set(&asked, Some(key.clone()), cx)
                            },
                            move |key, _, cx| {
                                let mut next = pinned.read(cx).clone();
                                let item = next.iter_mut().find(|item| &item.key == key).expect("a listed conversation");
                                item.pinned = !item.pinned;
                                set(&pinned, next, cx)
                            },
                            move |key, _, cx| {
                                let mut next = deleted.read(cx).clone();
                                next.retain(|item| &item.key != key);
                                set(&deleted, next, cx)
                            },
                        ),
                ),
            ),
    ))
    .children(now_renaming.map(|key| {
        PromptDialog::new("chat-rename", "Rename conversation", &naming, move |_, cx| set(&shut, None, cx))
            .label("Title")
            .submit("Rename")
            .check(|text| match text.trim() {
                "" => Err("A title cannot be empty.".into()),
                _ => Ok(()),
            })
            .on_submit(move |text, _, cx| {
                let mut next = renamed.read(cx).clone();
                let item = next.iter_mut().find(|item| item.key == key).expect("the conversation being renamed");
                item.title = text.trim().to_string().into();
                set(&renamed, next, cx);
                set(&done, None, cx)
            })
    }))
}

pub fn projects_section(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let active = keep(
        "chat-project",
        || Some(SharedString::from("library")),
        window,
        cx,
    );
    let files = keep(
        "chat-knowledge",
        || {
            vec![
                (SharedString::from("lift-notes.pdf"), 482_000_u64),
                (SharedString::from("palette.json"), 12_400),
                (SharedString::from("type-scale.md"), 6_800),
            ]
        },
        window,
        cx,
    );
    let (now_active, now_files) = (active.read(cx).clone(), files.read(cx).clone());
    let (added, removed) = (files.clone(), files);
    section(
        "FolderList / ProjectList / ProjectKnowledgePanel",
        "Projects that hold conversations, each with how many; and what a project knows beyond the conversation, how full that knowledge is, with a way to add more.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_start()
            .gap_6()
            .child(
                div().w(px(240.)).child(
                    ProjectList::new(
                        "chat-projects",
                        [
                            Project { key: "library".into(), name: "Component library".into(), count: 24 },
                            Project { key: "site".into(), name: "The website".into(), count: 9 },
                            Project { key: "notes".into(), name: "Color notes".into(), count: 3 },
                        ],
                    )
                    .active(now_active, move |key, _, cx| set(&active, Some(key.clone()), cx)),
                ),
            )
            .child(
                div().w(px(360.)).child(
                    ProjectKnowledgePanel::new("chat-knowledge", now_files, 2_000_000)
                        .on_add(move |_, cx| {
                            let mut next = added.read(cx).clone();
                            next.push((format!("reading-{}.md", next.len() + 1).into(), 3_200));
                            set(&added, next, cx)
                        })
                        .on_remove(move |ix, _, cx| {
                            let mut next = removed.read(cx).clone();
                            next.remove(ix);
                            set(&removed, next, cx)
                        }),
                ),
            ),
    )
}

const SHARED: [(&str, &str); 4] = [
    ("What does a lift do to a color?", ""),
    (
        "",
        "A **lift** blends a color toward white. Half a lift sits midway between the color and white.",
    ),
    ("And a shade?", ""),
    (
        "",
        "A **shade** blends toward black the same way. Dark themes build their surfaces from shades.",
    ),
];

const FORMATS: [(&str, &str); 3] = [("markdown", "Markdown"), ("pdf", "PDF"), ("json", "JSON")];

pub fn shared_section(
    history: &History,
    toaster: &Entity<Toaster>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let exporting = keep("chat-exporting", || false, window, cx);
    let format = keep(
        "chat-export-format",
        || SharedString::from("markdown"),
        window,
        cx,
    );
    let thinking = keep("chat-export-thinking", || false, window, cx);
    let (open, now_format, now_thinking) = (
        *exporting.read(cx),
        format.read(cx).clone(),
        *thinking.read(cx),
    );
    let (opener, closer) = (exporting.clone(), exporting);
    let (carried, toaster) = (
        History {
            all: history.all.clone(),
            active: history.active.clone(),
        },
        toaster.clone(),
    );
    let messages = MessageList::new("chat-shared-messages", SHARED.len(), |ix, _, _| {
        let (asked, answered) = SHARED[ix];
        let bubble = if asked.is_empty() {
            MessageBubble::new(Role::Assistant)
                .avatar(MessageAvatar::new(
                    ("chat-shared-mark", ix),
                    Role::Assistant,
                    "Assistant",
                ))
                .child(StreamingMarkdown::new(
                    ("chat-shared-text", ix),
                    answered,
                    false,
                ))
        } else {
            MessageBubble::new(Role::User).child(asked)
        };
        div().px_6().py_3().child(bubble).into_any_element()
    });
    let theme = cx.theme();
    section(
        "SharedConversationView / ConversationExport",
        "A conversation someone shared, to read and carry on; and saving one to a file, in a format, with or without its thinking.",
        cx,
    )
    .child(
        div()
            .w(px(640.))
            .h(px(300.))
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .overflow_hidden()
            .child(
                SharedConversationView::new("chat-shared", "Grace Lin", Timestamp::now() - 3.hours(), messages).on_continue(move |_, cx| {
                    let conversation = Conversation { key: "shared-lift".into(), title: SHARED[0].0.into(), at: Timestamp::now(), pinned: false };
                    open_new(&carried, conversation, cx)
                }),
            ),
    )
    .child(probe(
        "chat-export-open",
        Button::new("chat-export", "Export…")
            .variant(ButtonVariant::Secondary)
            .icon(IconName::Download)
            .on_click(move |_, _, cx| set(&opener, true, cx)),
    ))
    .children(open.then(|| {
        let label = FORMATS.iter().find(|(value, _)| *value == now_format.as_ref()).expect("a known format").1;
        ConversationExport::new(
            "chat-export-dialog",
            now_format,
            move |next, _, cx| set(&format, next.clone(), cx),
            move |_, cx| {
                toaster.update(cx, |toaster, cx| {
                    toaster.push(Toast::new(format!("Exported as {label}")).severity(Severity::Success), cx);
                })
            },
            move |_, cx| set(&closer, false, cx),
        )
        .thinking(now_thinking, move |on, _, cx| set(&thinking, on, cx))
    }))
}
