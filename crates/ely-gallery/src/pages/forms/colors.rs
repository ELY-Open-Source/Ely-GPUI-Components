use ely_gpui_component::{
    forms::{ColorPalette, ColorPicker, ColorSwatch, EyeDropper, GradientEditor, GradientStop},
    theme::{ActiveTheme, ControlSize},
};
use gpui::{App, Hsla, IntoElement, ParentElement, Styled, Window, div, px, rgb, rgba};

use crate::{
    probe::probe,
    ui::{code, keep, section, set, specimen, specimens},
};

fn hex(color: Hsla) -> String {
    let rgba = color.to_rgb();
    let byte = |value: f32| (value * 255.0).round() as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        byte(rgba.r),
        byte(rgba.g),
        byte(rgba.b)
    )
}

pub fn picker(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let color = keep("color-value", || Hsla::from(rgb(0x3772bb)), window, cx);
    let now = *color.read(cx);
    section(
        "ColorPicker",
        "Saturation and value on the plane, hue and alpha on the rails. Type it as hex, RGB or HSV; drag a letter to scrub.",
        cx,
    )
    .child(specimen(
        format!("{} at {:.0}%", hex(now), now.a * 100.0),
        probe(
            "color-picker",
            div().w(px(260.0)).child(
                ColorPicker::new("color-value", now)
                    .on_change(move |next, _, cx| set(&color, next, cx)),
            ),
        ),
        cx,
    ))
}

pub fn swatches(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chart = cx.theme().colors.chart;
    let names = [
        "Blue", "Teal", "Ochre", "Rose", "Violet", "Green", "Clay", "Cyan",
    ];
    let named: Vec<(&str, Hsla)> = names.into_iter().zip(chart).collect();
    let chosen = keep("palette-color", || chart[0], window, cx);
    let (now, picked) = (
        *chosen.read(cx),
        keep("dropper-color", || None::<Hsla>, window, cx),
    );
    let dropped = *picked.read(cx);
    section(
        "ColorSwatch / ColorPalette / EyeDropper",
        "Named patches; see-through colors sit on a checkerboard. The dropper reads the screen.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "palette",
                probe(
                    "palette",
                    ColorPalette::new("palette-color", named)
                        .selected(now)
                        .on_change(move |next, _, cx| set(&chosen, next, cx)),
                ),
                cx,
            ))
            .child(specimen(
                "swatches",
                div()
                    .flex()
                    .gap_2()
                    .child(ColorSwatch::new("swatch-solid", rgb(0x181613)).size(ControlSize::Md))
                    .child(ColorSwatch::new("swatch-glass", rgba(0x3772bb66)).size(ControlSize::Md))
                    .child(
                        ColorSwatch::new("swatch-chosen", rgb(0x267b4c))
                            .size(ControlSize::Md)
                            .selected(true),
                    ),
                cx,
            ))
            .child(specimen(
                dropped.map_or("eye dropper".to_string(), hex),
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(probe(
                        "dropper",
                        EyeDropper::new("eye-dropper")
                            .on_pick(move |next, _, cx| set(&picked, Some(next), cx)),
                    ))
                    .children(
                        dropped
                            .map(|color| ColorSwatch::new("dropped", color).size(ControlSize::Md)),
                    ),
                cx,
            )),
    )
}

pub fn gradient(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let stops = keep(
        "gradient-stops",
        || {
            vec![
                GradientStop {
                    at: 0.0,
                    color: rgb(0x181613).into(),
                },
                GradientStop {
                    at: 1.0,
                    color: rgb(0xe9e7e5).into(),
                },
            ]
        },
        window,
        cx,
    );
    let now = stops.read(cx).clone();
    section(
        "GradientEditor",
        "Click the bar to add a stop, drag stops to move them, Delete removes the chosen one.",
        cx,
    )
    .child(probe(
        "gradient",
        div().w(px(360.0)).child(
            GradientEditor::new("gradient-stops", now)
                .on_change(move |next, _, cx| set(&stops, next.to_vec(), cx)),
        ),
    ))
    .child(code("GradientStop { at: 0.5, color }", cx))
}
