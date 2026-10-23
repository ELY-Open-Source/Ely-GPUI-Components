use std::time::Duration;

use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    chat::{
        BranchNavigator, ErrorMessage, FeedbackForm, MessageActions, MessageAvatar, MessageBubble,
        MessageEditor, QuoteReply, RateLimitNotice, Role, ThinkingBlock, ThinkingDuration,
        ThinkingIndicator, continue_button, regenerate_button, stop_button,
    },
    forms::TextInput,
    primitives::IconName,
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use web_time::Instant;

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const SENT: &str = "What does a lift do to a color?";

fn field(
    key: &'static str,
    text: &str,
    hint: &str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TextInput> {
    let (text, hint) = (text.to_string(), hint.to_string());
    window.use_keyed_state(key, cx, move |window, cx| {
        let mut input = TextInput::new(window, cx)
            .multi_line(2, 6)
            .placeholder(hint);
        input.set_text(text, cx);
        input
    })
}

pub fn actions(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let rating = keep("chat-rating", || None::<bool>, window, cx);
    let now_rating = *rating.read(cx);
    let branch = keep("chat-branch", || 1_usize, window, cx);
    let now_branch = *branch.read(cx);
    let editing = keep("chat-editing", || false, window, cx);
    let sent = keep("chat-sent", || SharedString::from(SENT), window, cx);
    let now_sent = sent.read(cx).clone();
    let draft = field("chat-draft", SENT, "Your message", window, cx);
    let open = editing.clone();
    let body = if *editing.read(cx) {
        let (save, cancel, text) = (editing.clone(), editing.clone(), draft.clone());
        MessageEditor::new(
            "chat-editor",
            &draft,
            now_sent,
            move |_, cx| {
                let next = SharedString::from(text.read(cx).text().trim().to_string());
                set(&sent, next, cx);
                set(&save, false, cx)
            },
            move |_, cx| set(&cancel, false, cx),
        )
        .into_any_element()
    } else {
        div()
            .flex()
            .items_center()
            .gap_3()
            .child(div().child(now_sent))
            .child(probe(
                "chat-edit-open",
                Button::new("chat-edit", "Edit")
                    .variant(ButtonVariant::Ghost)
                    .icon(IconName::Pencil)
                    .on_click(move |_, _, cx| set(&open, true, cx)),
            ))
            .into_any_element()
    };
    section(
        "QuoteReply / MessageActions / MessageEditor / BranchNavigator / RegenerateButton / StopGeneratingButton / ContinueButton",
        "A quote to answer, the actions on a message, a sent message opened to change (the submit key saves, Escape cancels), the versions an answer has, and the buttons that regenerate, stop and continue.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_4()
            .child(QuoteReply::new("chat-quote", "Assistant", "A lift blends a color toward white. Half a lift sits midway between the color and white.").on_remove(|_, _| log::info!("gallery: quote removed")))
            .child(body)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        MessageActions::new("chat-actions", SENT)
                            .on_edit(|_, _| log::info!("gallery: edit"))
                            .on_regenerate(|_, _| log::info!("gallery: regenerate"))
                            .rating(now_rating, move |next, _, cx| set(&rating, next, cx)),
                    )
                    .child(BranchNavigator::new("chat-branch", now_branch, 3, move |next, _, cx| set(&branch, next, cx))),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(regenerate_button("chat-regenerate").on_click(|_, _, _| log::info!("gallery: regenerate")))
                    .child(stop_button("chat-stop").on_click(|_, _, _| log::info!("gallery: stop")))
                    .child(continue_button("chat-continue").on_click(|_, _, _| log::info!("gallery: continue"))),
            ),
    )
}

pub fn status(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let until = *keep(
        "chat-limit",
        || Instant::now() + Duration::from_secs(222),
        window,
        cx,
    )
    .read(cx);
    let picked = keep(
        "chat-reasons",
        || vec![SharedString::from("Too long")],
        window,
        cx,
    );
    let now_picked = picked.read(cx).clone();
    let note = field("chat-note", "", "Anything else? (optional)", window, cx);
    section(
        "ErrorMessage / RetryMessage / RateLimitNotice / FeedbackForm",
        "An answer that failed and a way to try again; a limit reached, counting down to when it lifts; and after a bad mark, what went wrong.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_4()
            .child(
                MessageBubble::new(Role::Error)
                    .avatar(MessageAvatar::new("chat-error-mark", Role::Error, "Assistant"))
                    .child(ErrorMessage::new("chat-error", "The answer stopped halfway").detail("The connection closed before the end. Nothing was lost.").on_retry(|_, _| log::info!("gallery: retry"))),
            )
            .child(RateLimitNotice::new("chat-limit", until).on_upgrade(|_, _| log::info!("gallery: raise limit")))
            .child(
                FeedbackForm::new(
                    "chat-feedback",
                    ["Wrong", "Unclear", "Too long", "Missed the point"],
                    now_picked,
                    &note,
                    move |next, _, cx| set(&picked, next.to_vec(), cx),
                    |_, _| log::info!("gallery: feedback sent"),
                )
                .on_cancel(|_, _| log::info!("gallery: feedback cancelled")),
            ),
    )
}

const REASONING: &str = "The reader asks what a lift does. Start from the definition: a blend toward white. Then give the two themes, since they lift by different amounts, and say why: a dark ground makes saturated color look louder, so it needs less of it.";

pub fn thinking(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    section(
        "ThinkingBlock / ReasoningPanel / ThinkingIndicator / ThinkingDuration",
        "The reasoning behind an answer, folded under how long it took; open, its steps read in a quiet column. While the assistant works, a word says what it is doing.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_4()
            .child(probe(
                "chat-thinking",
                div().w(px(560.)).child(ThinkingBlock::new("chat-thought", REASONING, false, Duration::from_secs(12))),
            ))
            .child(ThinkingBlock::new("chat-thinking-now", "Weighing how dark themes change", true, Duration::from_secs(3)))
            .child(ThinkingIndicator::new("chat-searching").label("Searching the web"))
            .child(ThinkingDuration::new(Duration::from_secs(75))),
    )
}
