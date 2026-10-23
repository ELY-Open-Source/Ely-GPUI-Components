use ely_gpui_component::{
    collab::{
        AnnotatedText, Annotation, Comment, CommentMarker, CommentSidebar, CommentThread, Decision,
        EditMode, Reaction, Suggestion, SuggestionMode, Thread, TrackChanges, accept, suggested,
        toggled,
    },
    forms::{Input, TextInput},
    theme::ActiveTheme,
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, ToSpan};

use super::people::{peers, you};
use crate::{
    probe::probe,
    ui::{keep, section, set},
};

fn threads(now: Timestamp) -> Vec<Thread> {
    let people = peers();
    let comment = |ix: usize, hours: i64, body: &str, reactions: Vec<Reaction>| Comment {
        author: people[ix].clone(),
        at: now - hours.hours(),
        body: body.to_string().into(),
        reactions,
    };
    let reaction = |emoji: &str, count: usize, mine: bool| Reaction {
        emoji: emoji.to_string().into(),
        count,
        mine,
    };
    vec![
        Thread {
            key: "gamma".into(),
            quote: Some("blends in gamma space".into()),
            comments: vec![
                comment(
                    0,
                    5,
                    "Should this say gamma? Most readers won't know the word.",
                    vec![reaction("👀", 2, false)],
                ),
                comment(
                    2,
                    3,
                    "Keep it, and link the glossary entry beside it.",
                    vec![reaction("👍", 1, true)],
                ),
            ],
            resolved: false,
        },
        Thread {
            key: "weight".into(),
            quote: Some("read with the same weight".into()),
            comments: vec![comment(
                1,
                26,
                "Could we show a light and a dark pair side by side here?",
                Vec::new(),
            )],
            resolved: false,
        },
        Thread {
            key: "title".into(),
            quote: Some("On Lift".into()),
            comments: vec![
                comment(3, 50, "The title reads well now.", Vec::new()),
                comment(0, 49, "Agreed, closing this.", Vec::new()),
            ],
            resolved: true,
        },
    ]
}

/// Changes the thread keyed `key`.
fn change(
    all: &Entity<Vec<Thread>>,
    key: &SharedString,
    cx: &mut App,
    edit: impl FnOnce(&mut Thread),
) {
    all.update(cx, |all, cx| {
        let thread = all
            .iter_mut()
            .find(|thread| &thread.key == key)
            .expect("the demo keeps its threads");
        edit(thread);
        cx.notify();
    });
}

/// Adds the field's words to the thread keyed `key` as yours, and clears it.
fn answer(all: &Entity<Vec<Thread>>, key: &SharedString, field: &Entity<TextInput>, cx: &mut App) {
    let body = field.read(cx).text().trim().to_string();
    field.update(cx, |field, cx| field.set_text("", cx));
    change(all, key, cx, |thread| {
        thread.comments.push(Comment {
            author: you(),
            at: Timestamp::now(),
            body: body.into(),
            reactions: Vec::new(),
        })
    });
}

fn reply_field(key: &'static str, window: &mut Window, cx: &mut App) -> Entity<TextInput> {
    window.use_keyed_state(key, cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Reply")
    })
}

pub fn comments(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let all = keep("collab-threads", || threads(Timestamp::now()), window, cx);
    let now = all.read(cx).clone();
    let (one, marked, side) = (
        reply_field("collab-reply-one", window, cx),
        reply_field("collab-reply-marker", window, cx),
        reply_field("collab-reply-side", window, cx),
    );
    let active = keep(
        "collab-active",
        || Some(SharedString::from("gamma")),
        window,
        cx,
    );
    let picked = active.read(cx).clone();
    let thread = |key: &str| {
        now.iter()
            .find(|thread| thread.key.as_ref() == key)
            .expect("the demo keeps its threads")
            .clone()
    };
    let key = |key: &'static str| SharedString::from(key);
    let (sent, resolved, reacted) = (all.clone(), all.clone(), all.clone());
    let first = CommentThread::new("collab-thread", thread("gamma"))
        .reply(&one, {
            let one = one.clone();
            move |_, cx| answer(&sent, &key("gamma"), &one, cx)
        })
        .on_resolve(move |on, _, cx| {
            change(&resolved, &key("gamma"), cx, |thread| thread.resolved = on)
        })
        .on_react(move |ix, emoji, _, cx| {
            change(&reacted, &key("gamma"), cx, |thread| {
                thread.comments[ix].reactions = toggled(&thread.comments[ix].reactions, emoji)
            })
        });
    let (sent, resolved) = (all.clone(), all.clone());
    let marker = CommentMarker::new("collab-marker", thread("weight"))
        .reply(&marked, {
            let marked = marked.clone();
            move |_, cx| answer(&sent, &key("weight"), &marked, cx)
        })
        .on_resolve(move |on, _, cx| {
            change(&resolved, &key("weight"), cx, |thread| thread.resolved = on)
        });
    let (sent, resolved, reacted, pick) = (all.clone(), all.clone(), all, active.clone());
    let sidebar = CommentSidebar::new("collab-sidebar", now.clone())
        .active(picked.clone(), move |key, _, cx| {
            set(&pick, Some(key.clone()), cx)
        })
        .reply(&side, {
            let side = side.clone();
            move |_, cx| {
                if let Some(key) = active.read(cx).clone() {
                    answer(&sent, &key, &side, cx)
                }
            }
        })
        .on_resolve(move |key, on, _, cx| change(&resolved, key, cx, |thread| thread.resolved = on))
        .on_react(move |key, ix, emoji, _, cx| {
            change(&reacted, key, cx, |thread| {
                thread.comments[ix].reactions = toggled(&thread.comments[ix].reactions, emoji)
            })
        });
    let theme = cx.theme();
    div()
        .child(
            section(
                "CommentThread / Reply / Reactions / ReactionPicker",
                "A conversation on the words it quotes: each comment with its author, time and reactions; a reply below, Enter to send; resolve above. Press a reaction to add yours or take it back, or pick another.",
                cx,
            )
            .child(probe("collab-thread", div().w(px(440.)).child(first))),
        )
        .child(
            section(
                "CommentBubble / CommentMarker",
                "A mark beside the words a thread is about, with its count; a press opens the thread in a bubble over the page.",
                cx,
            )
            .child(
                div()
                    .w(px(560.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_color(theme.colors.fg).child("In dark, accents lift further so they read with the same weight."))
                    .child(probe("collab-marker", marker)),
            ),
        )
        .child(
            section(
                "CommentSidebar",
                "Every conversation down a column, open or resolved; a press picks one, and the one picked takes replies.",
                cx,
            )
            .child(probe(
                "collab-sidebar",
                div()
                    .w(px(400.))
                    .h(px(560.))
                    .rounded(theme.radius(ely_gpui_component::theme::Radius::Lg))
                    .border_1()
                    .border_color(theme.colors.border)
                    .bg(theme.colors.sunken)
                    .child(sidebar),
            )),
        )
}

const PASSAGE: &str = "Every accent in the library is a base color and a lift. The base carries the hue; the lift carries how loud the color is. Hover, press and focus are lifts of the accent.";

pub fn annotations(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let picked = keep("collab-picked", || None::<usize>, window, cx);
    let now = *picked.read(cx);
    let people = peers();
    let mark = |words: &str, author: usize, note: Option<&str>| {
        let start = PASSAGE.find(words).expect("the passage holds its marks");
        Annotation {
            range: start..start + words.len(),
            author: people[author].clone(),
            note: note.map(|note| note.to_string().into()),
        }
    };
    section(
        "Annotation / Highlight",
        "Stretches people marked, washed in their colors; the ones with a note are underlined, and resting on one shows it. A press picks one. A plain mark with no author is typography::Highlight.",
        cx,
    )
    .child(probe(
        "collab-annotations",
        div().w(px(560.)).child(
            AnnotatedText::new(
                "collab-annotated",
                PASSAGE,
                [
                    mark("a base color and a lift", 0, Some("The one idea to remember.")),
                    mark("how loud the color is", 1, None),
                    mark("lifts of the accent", 2, Some("Link to the motion page.")),
                ],
            )
            .picked(now, move |ix, _, cx| set(&picked, Some(ix), cx)),
        ),
    ))
}

const BASE: &str = "A lift blends color toward white. Half a lift sits midway between.";

/// `base` with every one of `suggestions` taken.
fn applied(base: &str, suggestions: Vec<Suggestion>) -> String {
    let (mut text, mut left) = (base.to_string(), suggestions);
    while !left.is_empty() {
        let Some(taken) = accept(&text, &left, 0) else {
            break;
        };
        (text, left) = taken;
    }
    text
}

pub fn changes(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let mode = keep("collab-mode", || EditMode::Suggesting, window, cx);
    let now_mode = *mode.read(cx);
    let base = keep("collab-base", || BASE.to_string(), window, cx);
    let now_base = base.read(cx).clone();
    let draft = window.use_keyed_state("collab-suggesting", cx, |window, cx| {
        let mut input = TextInput::new(window, cx);
        input.set_text(
            "A tint blends every color toward white. Half a lift sits midway.",
            cx,
        );
        input
    });
    let edited = draft.read(cx).text().to_string();
    let (switch_base, switch_draft) = (base.clone(), draft.clone());
    let switch = SuggestionMode::new("collab-mode", now_mode, move |next, _, cx| {
        if *mode.read(cx) == EditMode::Editing {
            let text = switch_draft.read(cx).text().to_string();
            set(&switch_base, text, cx);
        }
        switch_draft.update(cx, |input, cx| {
            input.set_disabled(next == EditMode::Viewing, cx)
        });
        set(&mode, next, cx)
    });
    let track = match now_mode {
        EditMode::Editing => TrackChanges::new("collab-track", edited, Vec::new()),
        EditMode::Viewing => {
            let pending = suggested(&now_base, &edited, &you());
            TrackChanges::new("collab-track", now_base, pending)
        }
        EditMode::Suggesting => {
            let pending = suggested(&now_base, &edited, &you());
            let (decided, field) = (base.clone(), draft.clone());
            TrackChanges::new("collab-track", now_base, pending).on_decide(
                move |decision, _, cx| {
                    let base_text = decided.read(cx).clone();
                    let pending = suggested(&base_text, field.read(cx).text(), &you());
                    match decision {
                        Decision::Accept(ix) => {
                            if let Some((text, _)) = accept(&base_text, &pending, ix) {
                                set(&decided, text, cx)
                            }
                        }
                        Decision::AcceptAll => set(&decided, applied(&base_text, pending), cx),
                        Decision::Reject(ix) => {
                            let mut rest = pending;
                            if ix >= rest.len() {
                                return;
                            }
                            rest.remove(ix);
                            let text = applied(&base_text, rest);
                            field.update(cx, |input, cx| input.set_text(text, cx));
                        }
                        Decision::RejectAll => {
                            field.update(cx, |input, cx| input.set_text(base_text, cx))
                        }
                    }
                },
            )
        }
    };
    section(
        "SuggestionMode / TrackChanges",
        "Editing changes the words; suggesting keeps them and proposes the change, removed words struck and added ones underlined in the author's color, each to accept or reject; viewing only reads. Type in the field while suggesting.",
        cx,
    )
    .child(probe(
        "collab-changes",
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_4()
            .child(switch)
            .child(Input::new(&draft))
            .child(track),
    ))
}
