use ely_gpui_component::{
    forms::{EmailInput, InlineEdit, MaskedInput, PhoneInput, PinInput, UrlInput},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::text::field;
use crate::{
    probe::probe,
    ui::{section, specimen, specimens},
};

pub fn email_url(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let email = field("input-email", window, cx, |input| {
        input.placeholder("name@example.com")
    });
    let url = field("input-url", window, cx, |input| {
        input.placeholder("https://")
    });
    section(
        "EmailInput / UrlInput",
        "Checked once you leave the field; a quiet note says what fits.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "email",
                probe("email", div().w(px(280.0)).child(EmailInput::new(&email))),
                cx,
            ))
            .child(specimen(
                "url",
                div().w(px(280.0)).child(UrlInput::new(&url)),
                cx,
            )),
    )
}

pub fn phone_masked(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let phone = field("input-phone", window, cx, |input| input);
    let card = field("input-card", window, cx, |input| input);
    let country = window.use_keyed_state("phone-country", cx, |_, _| "US");
    let current = *country.read(cx);
    section(
        "PhoneInput / MaskedInput",
        "Typing fits the pattern; literals appear on their own.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "phone",
                probe(
                    "phone",
                    div()
                        .w(px(300.0))
                        .child(PhoneInput::new("phone", &phone, current).on_country(
                            move |code, _, cx| {
                                country.update(cx, |country, cx| {
                                    *country = code;
                                    cx.notify();
                                })
                            },
                        )),
                ),
                cx,
            ))
            .child(specimen(
                "masked: aa-9999",
                probe(
                    "masked",
                    div()
                        .w(px(200.0))
                        .child(MaskedInput::new("card", &card, "aa-9999")),
                ),
                cx,
            )),
    )
}

pub fn pin(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let entered = window.use_keyed_state("pin-entered", cx, |_, _| SharedString::from("waiting"));
    let note = entered.read(cx).clone();
    section(
        "PinInput / OTPInput",
        "Six boxes, one field underneath: type, paste, or backspace through them.",
        cx,
    )
    .child(probe(
        "pin",
        PinInput::new("otp", 6).on_complete(move |code, _, cx| {
            let code = SharedString::from(format!("entered {code}"));
            entered.update(cx, |entered, cx| {
                *entered = code;
                cx.notify();
            })
        }),
    ))
    .child(Caption::new(format!("Code: {note}")))
}

pub fn inline_edit(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let title = window.use_keyed_state("inline-title", cx, |_, _| {
        SharedString::from("Quarterly planning")
    });
    let now = title.read(cx).clone();
    section(
        "InlineEdit / EditableText",
        "Click the text to edit. Enter or leaving keeps it; Escape puts it back.",
        cx,
    )
    .child(probe(
        "inline",
        div()
            .w(px(280.0))
            .child(InlineEdit::new("title", now).on_commit(move |text, _, cx| {
                let text = text.clone();
                title.update(cx, |title, cx| {
                    *title = text;
                    cx.notify();
                })
            })),
    ))
}
