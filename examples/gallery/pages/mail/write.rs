use ely_gpui_component::{
    chat::Attachment,
    forms::{Input, TextInput},
    mail::{Contact, MailComposer, ScheduleSend, SignatureEditor},
    theme::{ActiveTheme, Radius},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::{day_and_hour, quiet};
use crate::{
    probe::probe,
    ui::{change, keep, section, set},
};

const SIGNATURE: &str = "Chloé Martin\nLead designer · Atrium Studio";

fn contacts() -> [Contact; 4] {
    [
        Contact::new("Ana Lima", "ana@atrium.studio"),
        Contact::new("Ben Ito", "ben@atrium.studio"),
        Contact::new("Chloé Martin", "chloe@atrium.studio"),
        Contact::new("Dev Rao", "dev@atrium.studio"),
    ]
}

fn file(key: &str, name: &str, bytes: u64) -> Attachment {
    Attachment {
        key: SharedString::from(key.to_string()),
        name: SharedString::from(name.to_string()),
        bytes,
        preview: None,
        progress: None,
    }
}

/// The demo composer: which one it is, its files, and what became of the last draft.
#[derive(Clone)]
struct Writing {
    round: usize,
    files: Vec<Attachment>,
    said: Option<SharedString>,
}

pub fn composer(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let writing = keep(
        "mail-writing",
        || Writing {
            round: 0,
            files: vec![file("plan", "atrium-plan.pdf", 2_400_000)],
            said: None,
        },
        window,
        cx,
    );
    let now = writing.read(cx).clone();
    let (sent, later, gone, attach, remove) = (
        writing.clone(),
        writing.clone(),
        writing.clone(),
        writing.clone(),
        writing,
    );
    let words = window.use_keyed_state(("mail-words", now.round), cx, |window, cx| {
        TextInput::new(window, cx)
            .multi_line(6, 16)
            .placeholder("Write your message")
    });
    let theme = cx.theme();
    let frame = div()
        .w(px(560.))
        .p_4()
        .border_1()
        .border_color(theme.colors.border)
        .rounded(theme.radius(Radius::Lg))
        .child(
            MailComposer::new(("mail-composer", now.round), Input::new(&words))
                .contacts(contacts())
                .signature(SIGNATURE)
                .files(now.files.clone())
                .on_send(move |draft, _, cx| {
                    let said = format!("Sent to {}.", draft.to.join(", "));
                    change(&sent, cx, |writing| writing.said = Some(said.into()))
                })
                .on_schedule(move |draft, at, _, cx| {
                    let said = format!("{} goes {}.", draft.subject, day_and_hour(at));
                    change(&later, cx, |writing| writing.said = Some(said.into()))
                })
                .on_discard(move |_, cx| {
                    change(&gone, cx, |writing| {
                        writing.round += 1;
                        writing.said = Some("Discarded.".into());
                    })
                })
                .on_attach(move |_, cx| {
                    change(&attach, cx, |writing| {
                        let ix = writing.files.len();
                        writing.files.push(file(
                            &format!("photo-{ix}"),
                            &format!("stair-hall-{ix}.jpg"),
                            3_100_000,
                        ));
                    })
                })
                .on_remove(move |key, _, cx| {
                    change(&remove, cx, |writing| {
                        writing.files.retain(|each| each.key != *key)
                    })
                }),
        );
    section(
        "MailComposer / RecipientInput → TagInput",
        "To, and Cc and Bcc once asked for, suggest people as you type and mark what is not an address. Then the subject, the words, the files and the signature. Send wakes once there is someone to send to, now or at a time; Discard lets the draft go.",
        cx,
    )
    .child(probe(
        "mail-composer",
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(frame)
            .children(now.said.map(|said| quiet(said, cx))),
    ))
}

pub fn signature(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let words = keep(
        "mail-signature",
        || SharedString::from(SIGNATURE),
        window,
        cx,
    );
    let shown = words.read(cx).clone();
    section(
        "SignatureEditor",
        "A signature's lines in a field, and under it how they will sit below a message.",
        cx,
    )
    .child(probe(
        "mail-signature",
        div().w(px(420.)).child(
            SignatureEditor::new("mail-signature", shown)
                .on_change(move |next, _, cx| set(&words, next.clone(), cx)),
        ),
    ))
}

pub fn schedule(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let said = keep("mail-schedule-said", || None::<SharedString>, window, cx);
    let shown = said.read(cx).clone();
    let (sent, later) = (said.clone(), said);
    section(
        "ScheduleSend",
        "Send now, or open the arrow for tomorrow morning, tomorrow afternoon, Monday morning, or a day and hour of your own.",
        cx,
    )
    .child(probe(
        "mail-schedule",
        div()
            .flex()
            .items_center()
            .gap_4()
            .child(
                ScheduleSend::new("mail-schedule")
                    .on_send(move |_, cx| set(&sent, Some("Sent now.".into()), cx))
                    .on_schedule(move |at, _, cx| {
                        set(&later, Some(format!("Goes {}.", day_and_hour(at)).into()), cx)
                    }),
            )
            .children(shown.map(|shown| quiet(shown, cx))),
    ))
}
