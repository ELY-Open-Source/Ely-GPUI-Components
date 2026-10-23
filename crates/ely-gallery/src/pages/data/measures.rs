use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    data_display::{Barcode, Comparison, Gauge, Meter, QrCode, Stars, UsageBar},
    theme::{ActiveTheme, IconSize, TextSize},
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, row, section, set, specimen, specimens},
};

pub fn codes(cx: &mut App) -> impl IntoElement + use<> {
    let qr = QrCode::new("https://github.com/ELY-Open-Source/Ely-GPUI-Components")
        .expect("a short link fits a QR code");
    let label = Barcode::code128("ELY-2419").expect("printable ASCII");
    let retail = Barcode::ean13("400638133393").expect("twelve digits");
    section(
        "QRCode / Barcode",
        "Codes for cameras and scanners: dark on a light tile in both themes, so they read anywhere. Code 128 takes printable ASCII; EAN-13 adds its own check digit.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("QR code", qr, cx))
            .child(specimen("Code 128", label, cx))
            .child(specimen("EAN-13", retail, cx)),
    )
}

/// Readings the demo steps through: storage, quota, memory, CPU, temperature.
const READINGS: [(f32, f32, f32, f64, f64); 3] = [
    (0.15, 0.62, 0.71, 38.0, 54.0),
    (0.41, 0.86, 0.9, 64.0, 71.5),
    (0.58, 0.97, 0.99, 91.0, 88.0),
];

pub fn measures(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let turn = keep("measures-turn", || 0usize, window, cx);
    let (now, next) = (*turn.read(cx), turn.clone());
    let (disk, quota, memory, cpu, heat) = READINGS[now % READINGS.len()];
    section(
        "Meter / Gauge / UsageBar",
        "How full things are. A meter turns amber past 80% and red past 95%; a gauge rolls its number and sweeps its arc; a usage bar splits one whole into parts.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(
                div()
                    .w(px(320.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(Meter::new("meter-disk", "Storage", disk).detail(format!("{:.0} GB of 256 GB", disk * 256.0)))
                    .child(Meter::new("meter-quota", "Upload quota", quota))
                    .child(Meter::new("meter-memory", "Memory", memory)),
            )
            .child(Gauge::new("gauge-cpu", "CPU", cpu, 0.0, 100.0).suffix("%"))
            .child(
                Gauge::new("gauge-heat", "Temperature", heat, 20.0, 100.0)
                    .decimals(1)
                    .suffix("°C")
                    .thresholds(0.7, 0.85),
            ),
    )
    .child(
        div().w(px(560.)).child(
            UsageBar::new(256.0)
                .part("Apps", 42.0)
                .part("Photos", 81.5)
                .part("Documents", 23.0)
                .part("System", 18.0),
        ),
    )
    .child(row().child(probe(
        "measures-next",
        Button::new("measures-next", "Next reading")
            .variant(ButtonVariant::Ghost)
            .on_click(move |_, _, cx| set(&next, now + 1, cx)),
    )))
}

pub fn stars(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let score = |value: f32, note: &'static str| {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(Stars::new(value))
            .child(
                div()
                    .text_size(theme.text_size(TextSize::Sm))
                    .text_color(theme.colors.fg_muted)
                    .child(note),
            )
    };
    section(
        "Stars / Rating display",
        "A score in stars, read only. Parts of a star fill too; to set a score, use Rating in Forms.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(score(4.6, "4.6 · 1,284 reviews"))
            .child(score(3.0, "3.0 · 18 reviews"))
            .child(div().flex().items_center().gap_2().child(Stars::new(8.5).max(10).size(IconSize::Sm)).child(
                div()
                    .text_size(theme.text_size(TextSize::Sm))
                    .text_color(theme.colors.fg_muted)
                    .child("8.5 of 10, small"),
            )),
    )
}

pub fn comparison(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "Comparison",
        "Options side by side, a row per feature. One column can stand out.",
        cx,
    )
    .child(
        div().w(px(640.)).child(
            Comparison::new(["Free", "Pro", "Team"])
                .featured(1)
                .row("Storage", ["5 GB", "1 TB", "Unlimited"])
                .marks("Version history", [false, true, true])
                .marks("Shared spaces", [false, false, true])
                .row("Support", ["Community", "Email", "Priority"]),
        ),
    )
}
