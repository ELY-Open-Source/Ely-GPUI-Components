use ely_gpui_component::mail::{Contact, MailReader, MailThreadView, Message, QuotedText};
use gpui::{App, IntoElement, ParentElement, Styled, div, px};
use jiff::{SignedDuration, Timestamp};

use crate::{probe::probe, ui::section};

fn ana() -> Contact {
    Contact::new("Ana Lima", "ana@atrium.studio")
}

fn ben() -> Contact {
    Contact::new("Ben Ito", "ben@atrium.studio")
}

fn you() -> Contact {
    Contact::new("You", "you@atrium.studio")
}

fn ago(minutes: i64) -> Timestamp {
    ely_gpui_component::typography::format::now() - SignedDuration::from_mins(minutes)
}

fn paragraph(text: &'static str) -> impl IntoElement {
    div().child(text)
}

pub fn reader(cx: &mut App) -> impl IntoElement + use<> {
    let message = Message::new("review", ana(), ago(12))
        .to([you(), ben()])
        .cc([Contact::new("Dev Rao", "dev@atrium.studio")])
        .file("atrium-plan.pdf", 2_400_000)
        .file("stair-hall.jpg", 3_100_000);
    section(
        "MailReader / QuotedText",
        "A message open for reading: the subject, who wrote to whom and when, replies, the words and the files. Words quoted from before fold behind a small mark until pressed.",
        cx,
    )
    .child(probe(
        "mail-reader",
        div().w(px(640.)).child(
            MailReader::new("mail-reader", "Friday's review moves to 10:00", message)
                .on_reply(|key, _, _| log::info!("gallery: reply {key}"))
                .on_reply_all(|key, _, _| log::info!("gallery: reply all {key}"))
                .on_forward(|key, _, _| log::info!("gallery: forward {key}"))
                .on_download(|key, file, _, _| log::info!("gallery: download {key} {file}"))
                .on_open_file(|key, file, _, _| log::info!("gallery: open {key} {file}"))
                .child(paragraph("Morning both,"))
                .child(paragraph(
                    "Friday's review moves to 10:00, in the model room. The client wants to see the stair at the scale we'll build it, so please bring the plaster samples if they've dried.",
                ))
                .child(paragraph("The plan and a photo of the stair hall are attached. — Ana"))
                .child(
                    QuotedText::new("mail-quoted")
                        .child(paragraph("On Thursday, Ben Ito wrote:"))
                        .child(paragraph(
                            "The samples are in the stair hall: lime, tadelakt and a sanded gypsum. The tadelakt needs another day.",
                        )),
                ),
        ),
    ))
}

pub fn thread(cx: &mut App) -> impl IntoElement + use<> {
    let message = |key: &'static str, from: Contact, minutes: i64, snippet: &'static str| {
        Message::new(key, from, ago(minutes))
            .to([you()])
            .snippet(snippet)
    };
    section(
        "MailThreadView",
        "A conversation, oldest first: the newest stands open, and the others fold to a line of who, their first words and when, which a press, Enter or Space opens.",
        cx,
    )
    .child(probe(
        "mail-thread",
        div().w(px(640.)).child(
            MailThreadView::new("mail-thread", "Plaster samples")
                .on_reply(|key, _, _| log::info!("gallery: reply {key}"))
                .on_forward(|key, _, _| log::info!("gallery: forward {key}"))
                .message(
                    message("s1", ben(), 60 * 30, "Three finishes are in the stair hall: lime, tadelakt and a sanded gypsum."),
                    paragraph("Three finishes are in the stair hall: lime, tadelakt and a sanded gypsum. Leave notes on the backs."),
                )
                .message(
                    message("s2", ana(), 60 * 26, "The lime reads warmest by the windows. Can we see it at dusk?"),
                    paragraph("The lime reads warmest by the windows. Can we see it at dusk?"),
                )
                .message(
                    message("s3", Contact::new("Chloé Martin", "chloe@atrium.studio"), 60 * 5, "Dusk works. I'll bring the lamp we use for the model."),
                    paragraph("Dusk works. I'll bring the lamp we use for the model."),
                )
                .message(
                    message("s4", ben(), 40, "The tadelakt is sealed. All three are ready for Friday."),
                    paragraph("The tadelakt is sealed. All three are ready for Friday."),
                ),
        ),
    ))
}
