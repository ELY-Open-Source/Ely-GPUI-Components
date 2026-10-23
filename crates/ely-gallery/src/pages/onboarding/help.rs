use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    data_display::{Change, Release},
    feedback::EmptyState,
    forms::{Choice, FormField, Input, TextInput},
    onboarding::{
        ContactSupport, FeedbackWidget, HelpArticle, HelpPanel, InlineHelp,
        KeyboardShortcutCheatsheet, WhatsNewDialog,
    },
    primitives::{HelpTooltip, IconName},
    settings::Shortcut,
    theme::{ActiveTheme, ControlSize, TextSize},
};
use gpui::{
    App, Entity, FontWeight, IntoElement, ParentElement, SharedString, Styled, Window, div,
    prelude::*, px,
};

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

/// The help demos: the article open, whether the panel shows, and which dialog is up.
#[derive(Default)]
struct Help {
    open: Option<SharedString>,
    hidden: bool,
    dialog: Option<Dialog>,
    sent: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Dialog {
    Shortcuts,
    WhatsNew,
}

fn field(
    key: &'static str,
    hint: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TextInput> {
    window.use_keyed_state(key, cx, move |window, cx| {
        TextInput::new(window, cx).placeholder(hint)
    })
}

fn article(key: &str, title: &str, summary: &str, body: &str) -> HelpArticle {
    HelpArticle {
        key: key.to_string().into(),
        title: title.to_string().into(),
        summary: summary.to_string().into(),
        body: body.to_string().into(),
    }
}

fn articles() -> [HelpArticle; 4] {
    [
        article(
            "start",
            "Getting started",
            "Make a project and invite your team.",
            "Press **New project** in the sidebar, give it a name, and pick who can see it.\n\nThen [invite your team](help:team).",
        ),
        article(
            "team",
            "Invite your team",
            "Bring people in by email.",
            "Open **Settings › Members** and press **Invite**. Each person gets a link that works for seven days.",
        ),
        article(
            "billing",
            "Change billing",
            "Who can, and where.",
            "Only admins can change billing. Open **Settings › Billing** to change the plan or the card.",
        ),
        article(
            "keys",
            "Work from the keyboard",
            "The shortcuts worth learning first.",
            "Press `cmd-k` to open anything, and `?` for every shortcut.",
        ),
    ]
}

fn shortcut(group: &str, name: &str, keys: &str) -> Shortcut {
    Shortcut {
        group: group.to_string().into(),
        name: name.to_string().into(),
        keys: keys.to_string().into(),
    }
}

pub fn tips(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("onboarding-help", Help::default, window, cx);
    let billing = field(
        "onboarding-billing-email",
        "billing@example.com",
        window,
        cx,
    );
    let theme = cx.theme();
    let more = state.clone();
    section(
        "HelpTooltip · InlineHelp · EmptyStateWithAction",
        "A help mark beside a name shows its tip on hover; FormLabel's help is one. A line of help sits where it is needed, and Learn more opens the article in the panel below. An empty state with an action is feedback::EmptyState.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div()
                    .w(px(320.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("Retention"))
                            .child(HelpTooltip::new("onboarding-retention-help", "Deleted items stay 30 days before they are gone.")),
                    )
                    .child(
                        FormField::new("onboarding-billing-field", "Billing email")
                            .help("Invoices go here as well as to the owner.")
                            .child(Input::new(&billing)),
                    )
                    .child(
                        InlineHelp::new("onboarding-billing-hint", "Only admins can change billing.").on_more(move |_, cx| {
                            change(&more, cx, |help| {
                                help.open = Some("billing".into());
                                help.hidden = false;
                            })
                        }),
                    ),
            )
            .child(
                div().w(px(320.)).text_size(theme.text_size(TextSize::Sm)).child(
                    EmptyState::new("onboarding-empty", IconName::FolderOpen, "No projects yet")
                        .body("Projects hold your work and who can see it.")
                        .action(
                            Button::new("onboarding-empty-new", "New project")
                                .variant(ButtonVariant::Primary)
                                .size(ControlSize::Sm)
                                .on_click(|_, _, _| log::info!("gallery: new project")),
                        ),
                ),
            ),
    )
}

pub fn panel(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("onboarding-help", Help::default, window, cx);
    let search = field("onboarding-help-search", "Search help", window, cx);
    let (open, hidden) = (state.read(cx).open.clone(), state.read(cx).hidden);
    let theme = cx.theme();
    let [opener, keys, close, show] = [(); 4].map(|_| state.clone());
    let body = match hidden {
        true => Button::new("onboarding-help-show", "Show help")
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Sm)
            .on_click(move |_, _, cx| change(&show, cx, |help| help.hidden = false))
            .into_any_element(),
        false => div()
            .h(px(460.))
            .rounded_lg()
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .child(
                HelpPanel::new(
                    "onboarding-help-panel",
                    articles(),
                    &search,
                    open,
                    move |key, _, cx| {
                        let key = key.cloned();
                        change(&opener, cx, |help| help.open = key)
                    },
                )
                .on_contact(|_, _| log::info!("gallery: contact support"))
                .on_shortcuts(move |_, cx| {
                    change(&keys, cx, |help| help.dialog = Some(Dialog::Shortcuts))
                })
                .on_close(move |_, cx| change(&close, cx, |help| help.hidden = true)),
            )
            .into_any_element(),
    };
    section(
        "HelpPanel / DocsSidebar",
        "Help beside the work: a search over the articles, one article with the way back, and links at its foot. A link to help:<key> opens another article.",
        cx,
    )
    .child(probe("onboarding-help-panel", div().w(px(320.)).child(body)))
}

pub fn dialogs(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("onboarding-help", Help::default, window, cx);
    let search = field("onboarding-keys-search", "Search shortcuts", window, cx);
    let dialog = state.read(cx).dialog;
    let [keys, news, shut, shut_news] = [(); 4].map(|_| state.clone());
    let open = match dialog {
        Some(Dialog::Shortcuts) => Some(
            KeyboardShortcutCheatsheet::new(
                "onboarding-cheatsheet",
                [
                    shortcut("General", "Open anything", "cmd-k"),
                    shortcut("General", "Every shortcut", "?"),
                    shortcut("General", "Settings", "cmd-,"),
                    shortcut("Editing", "Undo", "cmd-z"),
                    shortcut("Editing", "Redo", "cmd-shift-z"),
                    shortcut("Editing", "Find", "cmd-f"),
                    shortcut("View", "Split view", "cmd-\\"),
                    shortcut("View", "Zoom in", "cmd-="),
                ],
                &search,
                move |_, cx| change(&shut, cx, |help| help.dialog = None),
            )
            .into_any_element(),
        ),
        Some(Dialog::WhatsNew) => Some(
            WhatsNewDialog::new("onboarding-whats-new", move |_, cx| {
                change(&shut_news, cx, |help| help.dialog = None)
            })
            .release(
                Release::new("2.4", "Sep 25, 2026")
                    .summary("Split view, and a calmer sidebar.")
                    .change(Change::Added, "Split view: two files side by side.")
                    .change(Change::Added, "Pin a view to the toolbar.")
                    .change(Change::Fixed, "Search keeps its place after an edit."),
            )
            .release(
                Release::new("2.3", "Sep 9, 2026")
                    .change(Change::Changed, "The sidebar folds with a narrow window.")
                    .change(Change::Fixed, "Sync resumes after sleep."),
            )
            .into_any_element(),
        ),
        None => None,
    };
    section(
        "KeyboardShortcutCheatsheet · WhatsNewDialog",
        "Every shortcut in one dialog, its search taking focus as it opens; and what changed, newest first.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_3()
            .child(probe(
                "onboarding-shortcuts",
                Button::new("onboarding-shortcuts", "Keyboard shortcuts")
                    .icon(IconName::Keyboard)
                    .on_click(move |_, _, cx| change(&keys, cx, |help| help.dialog = Some(Dialog::Shortcuts))),
            ))
            .child(probe(
                "onboarding-whats-new",
                Button::new("onboarding-whats-new", "What's new")
                    .icon(IconName::Sparkles)
                    .on_click(move |_, _, cx| change(&news, cx, |help| help.dialog = Some(Dialog::WhatsNew))),
            )),
    )
    .children(open)
}

pub fn feedback(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("onboarding-help", Help::default, window, cx);
    let sent = state.read(cx).sent;
    let done = state.clone();
    let topics = [
        Choice::new("billing", "Billing"),
        Choice::new("bug", "Something broke"),
        Choice::new("account", "My account"),
        Choice::new("other", "Something else"),
    ];
    section(
        "FeedbackWidget · ContactSupport",
        "Four faces and a note, sent with one press; and a message to people, sent once it has a topic and words.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(probe(
                "onboarding-feedback",
                FeedbackWidget::new("onboarding-feedback", |sentiment, note, _, _| log::info!("gallery: feedback {} {note}", sentiment.key())),
            ))
            .child(
                div()
                    .w(px(360.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        ContactSupport::new("onboarding-support", topics, move |topic, _, _, cx| {
                                log::info!("gallery: support about {topic}");
                                change(&done, cx, |help| help.sent = true)
                            })
                            .address("help@example.com"),
                    )
                    .when(sent, |column| column.child(div().text_sm().child("Sent. We answer within a day."))),
            ),
    )
}
