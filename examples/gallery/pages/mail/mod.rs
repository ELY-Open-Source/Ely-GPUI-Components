use ely_gpui_component::{
    buttons::Button,
    feedback::EmptyState,
    mail::{Mail, MailList, Mailbox, MailboxList},
    primitives::IconName,
    theme::{ActiveTheme, Radius, TextSize},
    typography::format,
};
use gpui::{AnyElement, App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{SignedDuration, Timestamp, tz::TimeZone};

mod read;
mod sort;
mod write;

use super::Page;
use crate::{
    probe::probe,
    step::Step,
    ui::{change, keep, section},
};

pub const PAGE: Page = Page {
    number: 29,
    slug: "mail",
    title: "Mail",
    summary: "Mailboxes and the messages in them: reading, writing, sorting and putting off.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("mail-boxes", 100.0, 139.0),
    Step::UpAt("mail-boxes", 100.0, 139.0),
    Step::Wait(200),
    Step::Shot("box-opened"),
    Step::DownAt("mail-list", 417.0, 112.0),
    Step::UpAt("mail-list", 417.0, 112.0),
    Step::Wait(200),
    Step::Shot("starred"),
    Step::DownAt("mail-reader", 12.0, 227.0),
    Step::UpAt("mail-reader", 12.0, 227.0),
    Step::Wait(300),
    Step::Shot("quoted-open"),
    Step::DownAt("mail-thread", 200.0, 120.0),
    Step::UpAt("mail-thread", 200.0, 120.0),
    Step::Wait(300),
    Step::Shot("thread-opened"),
    Step::DownAt("mail-composer", 270.0, 31.0),
    Step::UpAt("mail-composer", 270.0, 31.0),
    Step::Type("an"),
    Step::Wait(200),
    Step::Shot("recipient-suggested"),
    Step::Key("enter"),
    Step::Wait(200),
    Step::Shot("recipient-added"),
    Step::DownAt("mail-labels", 150.0, 25.0),
    Step::UpAt("mail-labels", 150.0, 25.0),
    Step::Type("Suppliers"),
    Step::Wait(200),
    Step::Shot("label-new"),
    Step::Key("enter"),
    Step::Wait(200),
    Step::Shot("label-made"),
    Step::DownAt("mail-snooze", 50.0, 16.0),
    Step::UpAt("mail-snooze", 50.0, 16.0),
    Step::Wait(300),
    Step::Shot("snooze-open"),
    Step::Key("up"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("snooze-dialog"),
    Step::Key("tab"),
    Step::Key("enter"),
    Step::Wait(300),
    Step::Shot("snooze-when"),
    Step::Key("escape"),
    Step::Key("escape"),
];

/// What a demo says it did, under it.
fn quiet(text: SharedString, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .text_size(theme.text_size(TextSize::Sm))
        .text_color(theme.colors.fg_muted)
        .child(text)
}

/// A moment as the demos say it: its weekday and hour here.
fn day_and_hour(at: Timestamp) -> String {
    let zone = TimeZone::try_system().expect("the gallery reads the system time zone");
    format::datetime(at, &zone, "%a %H:%M").expect("a fixed pattern formats")
}

fn boxes(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = keep("mail-open-box", || SharedString::from("inbox"), window, cx);
    let shown = open.read(cx).clone();
    section(
        "MailboxList / FolderList",
        "Mailboxes, folders and labels in sections: a press, an arrow or Enter opens one, and the one open is lit. A box with mail unread stands out with its count; a label wears its hue.",
        cx,
    )
    .child(probe(
        "mail-boxes",
        div().w(px(260.)).child(
            MailboxList::new("mail-boxes")
                .section(
                    "Mailboxes",
                    [
                        Mailbox::new("inbox", "Inbox", IconName::Inbox).unread(12),
                        Mailbox::new("starred", "Starred", IconName::Star),
                        Mailbox::new("snoozed", "Snoozed", IconName::Clock),
                        Mailbox::new("sent", "Sent", IconName::Send),
                        Mailbox::new("drafts", "Drafts", IconName::FileText).unread(2),
                        Mailbox::new("archive", "Archive", IconName::Archive),
                        Mailbox::new("spam", "Spam", IconName::OctagonAlert).unread(4),
                        Mailbox::new("trash", "Trash", IconName::Trash2),
                    ],
                )
                .section(
                    "Folders",
                    [
                        Mailbox::new("atrium", "Atrium", IconName::Folder).unread(3),
                        Mailbox::new("invoices", "Invoices", IconName::Folder),
                    ],
                )
                .section(
                    "Labels",
                    [
                        Mailbox::label("clients", "Clients", 0),
                        Mailbox::label("studio", "Studio", 5),
                        Mailbox::label("press", "Press", 3),
                    ],
                )
                .open(shown)
                .on_open(move |key, _, cx| change(&open, cx, |open| *open = key.clone())),
        ),
    ))
}

/// The demo's inbox: dated back from now, so the list reads the time, the weekday, the date and the year.
fn inbox() -> Vec<Mail> {
    let now = Timestamp::now();
    let ago = |minutes: i64| now - SignedDuration::from_mins(minutes);
    vec![
        Mail::new(
            "review",
            "Ana Lima",
            "Friday's review moves to 10:00",
            ago(12),
        )
        .snippet("We'll meet in the model room. Bring the plaster samples if they've dried.")
        .unread(true),
        Mail::new("samples", "Ben Ito", "Plaster samples", ago(95))
            .snippet("Three finishes are in the stair hall: lime, tadelakt and a sanded gypsum.")
            .attachments(3)
            .unread(true),
        Mail::new("brief", "Chloé Martin", "The brief, final", ago(60 * 26))
            .snippet("Light from above, one stair, olive trees by the windows.")
            .starred(true),
        Mail::new(
            "invoice",
            "Studio Office",
            "Invoice 2026-114",
            ago(60 * 24 * 3),
        )
        .snippet("Please find the invoice for the September model work attached.")
        .attachments(1),
        Mail::new(
            "press",
            "Dev Rao",
            "A visit from the press",
            ago(60 * 24 * 9),
        )
        .snippet("They'd like to see the atrium before the opening. Would the 14th work?"),
        Mail::new(
            "welcome",
            "Eli Park",
            "Welcome to the studio",
            ago(60 * 24 * 400),
        )
        .snippet("A few notes to start: where the models live, and who to ask.")
        .starred(true),
    ]
}

fn list(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let selected = keep(
        "mail-selected",
        || vec![SharedString::from("review")],
        window,
        cx,
    );
    let stars = keep(
        "mail-stars",
        || vec![SharedString::from("brief"), SharedString::from("welcome")],
        window,
        cx,
    );
    let starred = stars.read(cx).clone();
    let mails = inbox()
        .into_iter()
        .map(|mail| {
            let on = starred.contains(&mail.key);
            mail.starred(on)
        })
        .collect::<Vec<_>>();
    let chosen = selected.read(cx).clone();
    section(
        "MailList / MailItem",
        "Messages with the sender strong while unread, the subject and first words under it, when each came, a paperclip for files, and a star to press. A press or an arrow selects; Command or Shift takes several.",
        cx,
    )
    .child(probe(
        "mail-list",
        div().w(px(440.)).h(px(320.)).child(
            MailList::new("mail-list", std::rc::Rc::new(mails))
                .selected(chosen)
                .on_select(move |keys, _, cx| change(&selected, cx, |selected| *selected = keys.to_vec()))
                .on_star(move |key, on, _, cx| {
                    change(&stars, cx, |stars| {
                        stars.retain(|each| each != key);
                        if on {
                            stars.push(key.clone());
                        }
                    })
                }),
        ),
    ))
}

fn zero(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    section(
        "InboxZeroState → EmptyState",
        "An inbox with nothing left in it says so, calm and brief.",
        cx,
    )
    .child(probe(
        "mail-zero",
        div()
            .w(px(440.))
            .py_6()
            .border_1()
            .border_color(theme.colors.border)
            .rounded(theme.radius(Radius::Lg))
            .child(
                EmptyState::new("mail-zero", IconName::Inbox, "All caught up")
                    .body("Nothing waits in your inbox. New mail lands here, newest first.")
                    .action(Button::new("mail-zero-archive", "Open the archive")),
            ),
    ))
}

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(boxes(window, cx))
        .child(list(window, cx))
        .child(zero(cx))
        .child(read::reader(cx))
        .child(read::thread(cx))
        .child(write::composer(window, cx))
        .child(write::signature(window, cx))
        .child(sort::labels(window, cx))
        .child(sort::snooze(window, cx))
        .child(write::schedule(window, cx))
        .into_any_element()
}
