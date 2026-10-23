use ely_gpui_component::{
    theme::TextSize,
    typography::{
        CopyableText, CurrencyText, DateTimeText, DurationStyle, DurationText, Emoji, FileSizeText,
        Latex, NumberText, PercentText, PluralText, RelativeTime, SelectableText,
    },
};
use gpui::{App, IntoElement, ParentElement, Styled, div, px};
use jiff::{SignedDuration, Timestamp};

use crate::{
    probe::probe,
    ui::{code, section, specimen, specimens},
};

/// Fixed moment, so shots repeat: 24 Sep 2026, 09:41 UTC.
fn moment() -> Timestamp {
    Timestamp::from_second(1_790_242_860).expect("valid timestamp")
}

pub fn relative_time(cx: &App) -> impl IntoElement + use<> {
    let ago = |seconds: i64| {
        Timestamp::now()
            .checked_sub(SignedDuration::from_secs(seconds))
            .expect("in range")
    };
    section(
        "RelativeTime",
        "Refreshes every thirty seconds while on screen.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("10 s", RelativeTime::new("rel-a", ago(10)), cx))
            .child(specimen("3 min", RelativeTime::new("rel-b", ago(180)), cx))
            .child(specimen("2 h", RelativeTime::new("rel-c", ago(7_200)), cx))
            .child(specimen(
                "3 d",
                RelativeTime::new("rel-d", ago(259_200)),
                cx,
            )),
    )
}

pub fn date_time(cx: &App) -> impl IntoElement + use<> {
    section("DateTimeText", "strftime patterns in the system zone.", cx).child(
        specimens()
            .child(specimen("default", DateTimeText::new(moment()), cx))
            .child(specimen(
                "%Y-%m-%d",
                DateTimeText::new(moment()).pattern("%Y-%m-%d"),
                cx,
            ))
            .child(specimen(
                "%A %H:%M",
                DateTimeText::new(moment()).pattern("%A %H:%M"),
                cx,
            )),
    )
}

pub fn numbers(cx: &App) -> impl IntoElement + use<> {
    section(
        "NumberText",
        "Grouped, fixed precision, tabular figures.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "2 decimals",
                NumberText::new(1_234_567.891).decimals(2),
                cx,
            ))
            .child(specimen("integer", NumberText::new(42_000.0), cx))
            .child(specimen(
                "negative",
                NumberText::new(-1_234.5).decimals(1),
                cx,
            )),
    )
}

pub fn currency(cx: &App) -> impl IntoElement + use<> {
    section(
        "CurrencyText",
        "ISO 4217 codes. Yen and won have no minor units.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("USD", CurrencyText::new(1_234.5, "USD"), cx))
            .child(specimen("EUR", CurrencyText::new(-42.0, "EUR"), cx))
            .child(specimen("JPY", CurrencyText::new(1_500.4, "JPY"), cx))
            .child(specimen("CHF", CurrencyText::new(9.99, "CHF"), cx)),
    )
}

pub fn percent(cx: &App) -> impl IntoElement + use<> {
    section(
        "PercentText",
        "Signed values carry direction in word and color.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("plain", PercentText::new(0.1234).decimals(2), cx))
            .child(specimen("gain", PercentText::new(0.052).signed(), cx))
            .child(specimen("loss", PercentText::new(-0.031).signed(), cx)),
    )
}

pub fn file_size(cx: &App) -> impl IntoElement + use<> {
    section(
        "FileSizeText",
        "Decimal units by default, binary on request.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("1536 B", FileSizeText::new(1_536), cx))
            .child(specimen("3.2 GB", FileSizeText::new(3_200_000_000), cx))
            .child(specimen("binary", FileSizeText::new(3 << 30).binary(), cx)),
    )
}

pub fn duration(cx: &App) -> impl IntoElement + use<> {
    section(
        "DurationText",
        "A clock for media, a compact span for logs.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "clock",
                DurationText::new(5_025, DurationStyle::Clock),
                cx,
            ))
            .child(specimen(
                "clock",
                DurationText::new(1_425, DurationStyle::Clock),
                cx,
            ))
            .child(specimen(
                "compact",
                DurationText::new(245, DurationStyle::Compact),
                cx,
            )),
    )
}

pub fn plural(cx: &App) -> impl IntoElement + use<> {
    section("PluralText", "The noun follows the count.", cx).child(
        specimens()
            .child(specimen("1", PluralText::new(1, "file", "files"), cx))
            .child(specimen(
                "12345",
                PluralText::new(12_345, "file", "files"),
                cx,
            )),
    )
}

pub fn copyable(cx: &App) -> impl IntoElement + use<> {
    section(
        "CopyableText",
        "One click copies. A check confirms, then leaves.",
        cx,
    )
    .child(probe(
        "copy-token",
        CopyableText::new("token", "ely_live_4f9a2c81d7").mono(),
    ))
}

pub fn selectable(cx: &App) -> impl IntoElement + use<> {
    section(
        "SelectableText",
        "Drag to select. ⌘C copies, ⌘A takes it all.",
        cx,
    )
    .child(probe(
        "select-text",
        div().max_w(px(520.0)).child(SelectableText::new(
            "selectable",
            "Selection works on read-only text too: logs, messages, receipts.",
        )),
    ))
}

pub fn emoji(cx: &App) -> impl IntoElement + use<> {
    let glyph = |code: &str| Emoji::shortcode(code).expect("known shortcode");
    section("Emoji", "Shortcodes resolve to the system color font.", cx).child(
        specimens()
            .child(specimen("sparkles", glyph("sparkles"), cx))
            .child(specimen("herb", glyph("herb"), cx))
            .child(specimen("coffee", glyph("coffee"), cx))
            .child(specimen("xl", glyph("sparkles").size(TextSize::Xxl), cx)),
    )
}

pub fn latex(cx: &App) -> impl IntoElement + use<> {
    let math =
        |id: &'static str, source: &str| Latex::parse(id, source).expect("gallery formula parses");
    section(
        "Latex / MathInline",
        "TeX typeset by RaTeX, in the text's color.",
        cx,
    )
    .child(
        specimens()
            .gap_10()
            .child(specimen("euler", math("euler", r"e^{i\pi} + 1 = 0"), cx))
            .child(specimen(
                "sum",
                math("sum", r"\sum_{k=1}^{n} k = \frac{n(n+1)}{2}"),
                cx,
            ))
            .child(specimen(
                "root",
                math("root", r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}"),
                cx,
            )),
    )
    .child(code("Latex::parse(\"euler\", r\"e^{i\\pi} + 1 = 0\")?", cx))
}
