use ely_gpui_component::{
    forms::{
        CodeInput, Combobox, EmojiPicker, FontPicker, IconPicker, JsonInput, NumberInput,
        SignaturePad, Stroke, code_highlights, json_highlights,
    },
    primitives::IconName,
    theme::ActiveTheme,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::text::field;
use crate::{
    probe::probe,
    ui::{keep, section, set, specimen, specimens},
};

pub fn signature(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let strokes = keep("signature-strokes", Vec::<Stroke>::new, window, cx);
    let now = strokes.read(cx).clone();
    section(
        "SignaturePad",
        "Sign with the pointer. A stroke lands when the pen lifts; Clear starts over.",
        cx,
    )
    .child(specimen(
        match now.len() {
            1 => "1 stroke".to_string(),
            count => format!("{count} strokes"),
        },
        probe(
            "signature",
            div().w(px(420.0)).child(
                SignaturePad::new("signature", now)
                    .on_change(move |next, _, cx| set(&strokes, next.to_vec(), cx)),
            ),
        ),
        cx,
    ))
}

pub fn code(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let line = field("input-code", window, cx, |input| {
        input
            .placeholder("cargo run -p ely-gallery")
            .highlighter(code_highlights)
    });
    let json = field("input-json", window, cx, |input| {
        input
            .multi_line(4, 8)
            .placeholder("{ \"name\": \"Ely\" }")
            .highlighter(json_highlights)
    });
    section(
        "CodeInput / JsonInput",
        "One line of code, colored the way most languages read. JSON is checked as you type.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "code",
                probe("code", div().w(px(420.0)).child(CodeInput::new(&line))),
                cx,
            ))
            .child(specimen(
                "json",
                probe("json", div().w(px(420.0)).child(JsonInput::new(&json))),
                cx,
            )),
    )
}

pub fn font(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let search = field("input-font", window, cx, |input| {
        input.placeholder("Search fonts")
    });
    let ui = cx.theme().font_family.clone();
    let family = keep("font-family", move || ui, window, cx);
    let now = family.read(cx).clone();
    section(
        "FontPicker",
        "Every family the system knows, and a line set in the chosen one.",
        cx,
    )
    .child(probe(
        "font",
        div().w(px(420.0)).child(
            FontPicker::new("font-picker", &search)
                .selected(now)
                .on_change(move |next, _, cx| set(&family, next.clone(), cx)),
        ),
    ))
}

pub fn glyphs(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let icon = keep("icon-picked", || IconName::Sparkles, window, cx);
    let emoji = keep("emoji-picked", || SharedString::from("✨"), window, cx);
    let (icon_now, emoji_now) = (*icon.read(cx), emoji.read(cx).clone());
    section(
        "IconPicker / EmojiPicker",
        "Type to narrow. Down moves into the grid, arrows walk it, Enter picks.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                icon_now.name(),
                probe(
                    "icons",
                    IconPicker::new("icon-picker")
                        .selected(icon_now)
                        .on_change(move |next, _, cx| set(&icon, next, cx)),
                ),
                cx,
            ))
            .child(specimen(
                emoji_now.clone(),
                probe(
                    "emoji",
                    EmojiPicker::new("emoji-picker")
                        .selected(emoji_now)
                        .on_change(move |next, _, cx| {
                            set(&emoji, SharedString::from(next.to_string()), cx)
                        }),
                ),
                cx,
            )),
    )
}

pub fn regional(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let language = field("input-language", window, cx, |input| {
        input.placeholder("Language")
    });
    let country = field("input-country", window, cx, |input| {
        input.placeholder("Country")
    });
    let currency = field("input-currency", window, cx, |input| {
        input.placeholder("Currency")
    });
    let spoken = keep("regional-language", || SharedString::from("zh"), window, cx);
    let land = keep("regional-country", || SharedString::from("CN"), window, cx);
    let money = keep(
        "regional-currency",
        || SharedString::from("CNY"),
        window,
        cx,
    );
    let amount = keep("regional-amount", || 1280.0, window, cx);
    let (spoken_now, land_now, money_now, amount_now) = (
        spoken.read(cx).clone(),
        land.read(cx).clone(),
        money.read(cx).clone(),
        *amount.read(cx),
    );
    section(
        "LanguageSelect / CountrySelect / CurrencySelect",
        "Languages under their own names, countries with flags, currencies by ISO 4217. Type a name or a code.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                spoken_now.clone(),
                div().w(px(260.0)).child(
                    Combobox::languages("language-select", &language)
                        .selected(spoken_now)
                        .on_change(move |next, _, cx| set(&spoken, next.clone(), cx)),
                ),
                cx,
            ))
            .child(specimen(
                land_now.clone(),
                probe(
                    "country",
                    div().w(px(260.0)).child(
                        Combobox::countries("country-select", &country)
                            .selected(land_now)
                            .on_change(move |next, _, cx| set(&land, next.clone(), cx)),
                    ),
                ),
                cx,
            ))
            .child(specimen(
                money_now.clone(),
                div().w(px(260.0)).child(
                    Combobox::currencies("currency-select", &currency)
                        .selected(money_now.clone())
                        .on_change(move |next, _, cx| set(&money, next.clone(), cx)),
                ),
                cx,
            )),
    )
    .child(specimen(
        format!("an amount in {money_now}"),
        div().w(px(260.0)).child(
            NumberInput::new("regional-amount", amount_now)
                .label("Amount")
                .currency(&money_now)
                .on_change(move |next, _, cx| set(&amount, next, cx)),
        ),
        cx,
    ))
}
