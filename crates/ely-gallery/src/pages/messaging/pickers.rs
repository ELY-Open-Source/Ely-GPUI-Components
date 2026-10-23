use ely_gpui_component::{
    messaging::{ChatMessage, Gif, GifPicker, Gifs, Sticker, StickerPack, StickerPicker},
    primitives::Image,
    theme::{ActiveTheme, Radius, TextSize},
};
use gpui::{
    App, ElementId, IntoElement, ObjectFit, ParentElement, SharedString, Styled, Window, div, px,
};
use jiff::{civil::date, tz::TimeZone};

use crate::{
    probe::probe,
    ui::{keep, picture, section, set, specimen, specimens},
};

const ATRIUM: [(&str, &str, &str); 4] = [
    ("arch", "Arch", asset!("stickers/arch.png")),
    ("stair", "Stair", asset!("stickers/stair.png")),
    ("column", "Column", asset!("stickers/column.png")),
    ("window", "Window", asset!("stickers/window.png")),
];

const GARDEN: [(&str, &str, &str); 4] = [
    ("olive", "Olive", asset!("stickers/olive.png")),
    ("sun", "Sun", asset!("stickers/sun.png")),
    ("stone", "Stone", asset!("stickers/stone.png")),
    ("sprout", "Sprout", asset!("stickers/sprout.png")),
];

/// The demo's GIFs: key, title, file, and width over height.
const GIFS: [(&str, &str, &str, f32); 4] = [
    (
        "stair-light",
        "Light on the stair",
        asset!("gifs/stair-light.gif"),
        1.5,
    ),
    (
        "olive-breath",
        "The olive tree",
        asset!("gifs/olive-breath.gif"),
        1.5,
    ),
    (
        "dunes-drift",
        "Dunes at dusk",
        asset!("gifs/dunes-drift.gif"),
        240.0 / 129.0,
    ),
    (
        "atrium-fade",
        "The atrium, before and after",
        asset!("gifs/atrium-fade.gif"),
        200.0 / 133.0,
    ),
];

fn pack(name: &str, stickers: [(&'static str, &'static str, &'static str); 4]) -> StickerPack {
    StickerPack::new(
        name.to_string(),
        stickers.map(|(key, title, file)| Sticker::new(key, title, picture(file))),
    )
}

/// What you sent, as a message at a story time.
fn sent(id: &'static str, body: impl IntoElement) -> ChatMessage {
    let at = date(2026, 9, 26)
        .at(10, 20, 0, 0)
        .to_zoned(TimeZone::UTC)
        .expect("a UTC time")
        .timestamp();
    ChatMessage::new(id, "You", at)
        .zone(TimeZone::UTC)
        .child(body)
}

fn hint(text: &'static str, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .text_size(theme.text_size(TextSize::Sm))
        .text_color(theme.colors.fg_muted)
        .child(text)
}

pub fn stickers(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chosen = keep("messaging-sticker", || None::<&'static str>, window, cx);
    let shown = *chosen.read(cx);
    let message = match shown.and_then(|key| {
        ATRIUM
            .iter()
            .chain(GARDEN.iter())
            .find(|(each, _, _)| *each == key)
    }) {
        Some((key, _, file)) => sent(
            "messaging-sticker-sent",
            div().size(px(96.)).child(
                Image::new(
                    (
                        ElementId::from("messaging-sticker-picture"),
                        key.to_string(),
                    ),
                    picture(*file),
                )
                .fit(ObjectFit::Contain)
                .size_full(),
            ),
        )
        .into_any_element(),
        None => hint("Pick a sticker to send it.", cx).into_any_element(),
    };
    section(
        "StickerPicker",
        "Stickers a pack at a time, chosen above; a search spans every pack and keeps the stickers whose names hold it. Arrows move, and Enter or a press sends one.",
        cx,
    )
    .child(probe(
        "messaging-stickers",
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                StickerPicker::new(
                    "messaging-stickers",
                    [pack("Atrium", ATRIUM), pack("Garden", GARDEN)],
                )
                .on_pick(move |key, _, cx| {
                    let key = ATRIUM
                        .iter()
                        .chain(GARDEN.iter())
                        .map(|(each, _, _)| *each)
                        .find(|each| *each == key.as_ref())
                        .expect("a picked sticker is listed");
                    set(&chosen, Some(key), cx)
                }),
            )
            .child(div().w(px(260.)).child(message)),
    ))
}

/// The GIFs whose titles hold `words`, as the host would answer.
fn found(words: &str) -> Vec<Gif> {
    let words = words.to_lowercase();
    GIFS.iter()
        .filter(|(_, title, _, _)| title.to_lowercase().contains(&words))
        .map(|(key, title, file, ratio)| Gif::new(*key, *title, picture(*file), *ratio))
        .collect()
}

pub fn gifs(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let words = keep("messaging-gif-words", SharedString::default, window, cx);
    let chosen = keep("messaging-gif", || None::<&'static str>, window, cx);
    let results = found(&words.read(cx).clone());
    let shown = *chosen.read(cx);
    let theme = cx.theme();
    let round = theme.radius(Radius::Md);
    let message = match shown.and_then(|key| GIFS.iter().find(|(each, _, _, _)| *each == key)) {
        Some((key, _, file, ratio)) => sent(
            "messaging-gif-sent",
            div().w(px(200.)).h(px(200. / ratio)).child(
                Image::new(
                    (ElementId::from("messaging-gif-picture"), key.to_string()),
                    picture(*file),
                )
                .size_full()
                .rounded(round),
            ),
        )
        .into_any_element(),
        None => hint("Pick a GIF to send it.", cx).into_any_element(),
    };
    section(
        "GifPicker",
        "GIFs from the host in two columns packed tight, each moving. The host answers the search as it changes; while it is empty, categories fill it. Each GIF is a Tab stop that a press, Enter or Space sends.",
        cx,
    )
    .child(probe(
        "messaging-gifs",
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div().w(px(320.)).child(
                    GifPicker::new("messaging-gifs", Gifs::Found(results))
                        .categories(["Stair", "Olive", "Dunes", "Atrium"])
                        .on_query(move |query, _, cx| set(&words, query.clone(), cx))
                        .on_pick(move |key, _, cx| {
                            let key = GIFS
                                .iter()
                                .map(|(each, _, _, _)| *each)
                                .find(|each| *each == key.as_ref())
                                .expect("a picked GIF is listed");
                            set(&chosen, Some(key), cx)
                        }),
                ),
            )
            .child(div().w(px(260.)).child(message)),
    ))
    .child(specimens().child(specimen(
        "searching",
        div()
            .w(px(240.))
            .child(GifPicker::new("messaging-gifs-waiting", Gifs::Loading)),
        cx,
    )))
}
