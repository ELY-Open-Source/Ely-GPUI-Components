use ely_gpui_component::{
    chat::{
        CapabilityCards, FollowUpSuggestions, MessageAvatar, MessageBubble, PromptInput, Role,
        StreamingMarkdown, SuggestionChips, WelcomeScreen,
    },
    documents::{Template, TemplatePicker},
    feedback::{Toast, Toaster},
    forms::TextInput,
    primitives::{IconName, Severity},
    theme::{ActiveTheme, Radius},
};
use gpui::{App, Entity, IntoElement, ParentElement, Styled, Window, div, px};
use jiff::{Timestamp, tz::TimeZone};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

/// The welcome composer's text, which starters and saved prompts fill.
pub fn welcome_draft(window: &mut Window, cx: &mut App) -> Entity<TextInput> {
    window.use_keyed_state("chat-welcome-draft", cx, |window, cx| {
        TextInput::new(window, cx)
            .multi_line(1, 6)
            .placeholder("How can I help?")
    })
}

pub fn welcome_section(
    draft: &Entity<TextInput>,
    toaster: &Entity<Toaster>,
    _: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let zone = TimeZone::try_system().expect("the gallery reads the system time zone");
    let hour = Timestamp::now().to_zoned(zone).hour() as u8;
    let (filled, typed, sent, toaster) =
        (draft.clone(), draft.clone(), draft.clone(), toaster.clone());
    let starters = [
        ("Explain a lift", IconName::Sparkles),
        ("Review my palette", IconName::Palette),
        ("Draft release notes", IconName::FileText),
    ];
    let theme = cx.theme();
    section(
        "WelcomeScreen / Greeting / SuggestionChips / StarterPrompts / CapabilityCards",
        "The first thing a new conversation shows: a greeting for the time of day, the composer, prompts to start from and what the assistant can do; a press fills the composer.",
        cx,
    )
    .child(
        div()
            .w(px(760.))
            .h(px(520.))
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .child(
                WelcomeScreen::new("chat-welcome", hour)
                    .name("Ada")
                    .child(PromptInput::new("chat-welcome-compose", draft, move |_, cx| {
                        sent.update(cx, |input, cx| input.set_text("", cx));
                        toaster.update(cx, |toaster, cx| {
                            toaster.push(Toast::new("Message sent").severity(Severity::Success), cx);
                        })
                    }))
                    .child(SuggestionChips::new("chat-starters", starters, move |ix, _, cx| {
                        let text = starters[ix].0;
                        filled.update(cx, |input, cx| input.set_text(text, cx))
                    }))
                    .child(CapabilityCards::new(
                        "chat-capabilities",
                        [
                            (IconName::Globe, "Search the web", "Answers with sources you can open."),
                            (IconName::Code, "Write and run code", "Tries an idea before you use it."),
                            (IconName::FileText, "Read your files", "Pages and notes you attach."),
                        ],
                        move |ix, _, cx| {
                            let text = ["Search the web for ", "Write code that ", "Read my file and "][ix];
                            typed.update(cx, |input, cx| input.set_text(text, cx))
                        },
                    )),
            ),
    )
}

pub fn follow_ups_section(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let asked = keep("chat-follow-up", || None::<usize>, window, cx);
    let now_asked = *asked.read(cx);
    section(
        "FollowUpSuggestions",
        "Questions to ask next, under an answer; a press asks one.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                MessageBubble::new(Role::Assistant)
                    .avatar(MessageAvatar::new("chat-followup-mark", Role::Assistant, "Assistant"))
                    .child(StreamingMarkdown::new("chat-followup-text", "Dark themes lift their accents further, because saturated color on charcoal looks louder.", false)),
            )
            .child(probe(
                "chat-follow-ups",
                div().w(px(560.)).child(FollowUpSuggestions::new(
                    "chat-follow-ups",
                    QUESTIONS,
                    move |ix, _, cx| set(&asked, Some(ix), cx),
                )),
            ))
            .children(now_asked.map(|ix| MessageBubble::new(Role::User).child(QUESTIONS[ix]))),
    )
}

const QUESTIONS: [&str; 3] = [
    "How much further, in numbers?",
    "Does the same hold for text?",
    "Show both themes side by side",
];

fn prompts() -> Vec<Template> {
    [
        ("Review this code", "🔍", "Code", "A careful read for bugs and clarity.", "Read the code below. List bugs first, then anything unclear, each with the line and a fix."),
        ("Explain like a textbook", "📘", "Writing", "Plain words, one idea at a time.", "Explain the idea below to a newcomer: define terms before using them, one idea per paragraph."),
        ("Release notes", "📝", "Writing", "What changed, for people who use it.", "Turn these changes into notes a user reads: what they can do now, what moved, what broke."),
    ]
    .into_iter()
    .map(|(name, icon, category, description, markdown)| Template {
        name: name.into(),
        icon: icon.into(),
        category: category.into(),
        description: description.into(),
        markdown: markdown.into(),
    })
    .collect()
}

pub fn library_section(
    draft: &Entity<TextInput>,
    _: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let (all, used) = (prompts(), draft.clone());
    let texts: Vec<_> = all.iter().map(|prompt| prompt.markdown.clone()).collect();
    section(
        "PromptLibrary",
        "Saved prompts, found by name and kept by category, the one in view shown whole: documents::TemplatePicker over prompts.",
        cx,
    )
    .child(
        div().w(px(760.)).h(px(400.)).child(
            TemplatePicker::new("chat-prompts", all)
                .on_use(move |ix, _, cx| used.update(cx, |input, cx| input.set_text(texts[ix].to_string(), cx))),
        ),
    )
}
