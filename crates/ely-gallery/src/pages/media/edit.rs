use std::path::PathBuf;

use ely_gpui_component::{
    buttons::SegmentedControl,
    media::{Crop, ImageAnnotator, ImageCropper, ImageUpload, Mark},
    theme::ActiveTheme,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, picture, section, set},
};

const ATRIUM: &str = asset!("atrium.jpg");
const STAIR: &str = asset!("atrium-stair.jpg");
const DUNES: &str = asset!("dunes.jpg");

/// Crop shapes to pick from, width over height; none is free.
const ASPECTS: [(&str, Option<f32>); 4] = [
    ("Free", None),
    ("1:1", Some(1.0)),
    ("4:3", Some(4.0 / 3.0)),
    ("16:9", Some(16.0 / 9.0)),
];

pub fn cropper(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let kept = keep(
        "media-crop",
        || {
            (
                Crop {
                    x: 0.12,
                    y: 0.1,
                    w: 0.7,
                    h: 0.75,
                },
                "Free",
            )
        },
        window,
        cx,
    );
    let (crop, shape) = *kept.read(cx);
    let aspect = ASPECTS
        .iter()
        .find(|(name, _)| *name == shape)
        .expect("a listed shape")
        .1;
    let (reshaped, moved) = (kept.clone(), kept.clone());
    let shapes = ASPECTS.iter().fold(
        SegmentedControl::new("media-crop-shape", shape),
        |control, (name, _)| control.segment(*name, *name, None),
    );
    let readout = format!(
        "{:.0}% across, {:.0}% down · {:.0}% by {:.0}%",
        crop.x * 100.0,
        crop.y * 100.0,
        crop.w * 100.0,
        crop.h * 100.0
    );
    section(
        "ImageCropper",
        "A box to crop a picture by. Drag it to move it and its handles to size it; with a shape, only its corners size it. The arrows move it, Shift further.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_3()
            .child(shapes.on_change(move |name, _, cx| {
                let (crop, _) = *reshaped.read(cx);
                let name: &'static str = ASPECTS.iter().find(|(each, _)| *each == name.as_ref()).expect("a listed shape").0;
                let aspect = ASPECTS.iter().find(|(each, _)| *each == name).expect("a listed shape").1;
                let crop = aspect.map_or(crop, |aspect| crop.fitted(aspect, 1.5));
                set(&reshaped, (crop, name), cx);
            }))
            .child(probe(
                "media-cropper",
                div().w(px(560.)).child(
                    ImageCropper::new("media-cropper", picture(ATRIUM), 1.5, crop)
                        .aspect(aspect)
                        .on_change(move |crop, _, cx| {
                            let shape = moved.read(cx).1;
                            set(&moved, (crop, shape), cx)
                        }),
                ),
            ))
            .child(div().text_size(cx.theme().text_size(ely_gpui_component::theme::TextSize::Sm)).text_color(cx.theme().colors.fg_muted).child(readout)),
    )
}

pub fn upload(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let kept = keep(
        "media-upload",
        || (Some(PathBuf::from(DUNES)), None::<Crop>),
        window,
        cx,
    );
    let (picked, crop) = kept.read(cx).clone();
    let (pick, cropped, removed) = (kept.clone(), kept.clone(), kept.clone());
    section(
        "ImageUpload",
        "A picture to send: drop one or browse for it, then crop it. Replace picks another; Remove empties it.",
        cx,
    )
    .child(probe(
        "media-upload",
        div().w(px(420.)).child(
            ImageUpload::new("media-upload", picked, crop)
                .aspect(Some(1.0))
                .on_pick(move |path, _, cx| set(&pick, (Some(path), None), cx))
                .on_crop(move |crop, _, cx| {
                    let picked = cropped.read(cx).0.clone();
                    set(&cropped, (picked, Some(crop)), cx)
                })
                .on_remove(move |_, cx| set(&removed, (None, None), cx)),
        ),
    ))
}

pub fn annotator(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let kept = keep(
        "media-marks",
        || {
            vec![
                Mark::Box {
                    from: (0.52, 0.18),
                    to: (0.9, 0.86),
                    color: 0,
                },
                Mark::Arrow {
                    from: (0.2, 0.3),
                    to: (0.36, 0.52),
                    color: 2,
                },
                Mark::Pin {
                    at: (0.3, 0.72),
                    color: 1,
                },
            ]
        },
        window,
        cx,
    );
    let marks = kept.read(cx).clone();
    let count = SharedString::from(format!("{} marks", marks.len()));
    let changed = kept.clone();
    section(
        "ImageAnnotator",
        "Marks on a picture: boxes, arrows, lines drawn freehand, numbered pins. Pick a tool and a color above; with Select, a press chooses a mark and Delete removes it. Cmd-Z undoes.",
        cx,
    )
    .child(probe(
        "media-annotator",
        div().w(px(560.)).child(
            ImageAnnotator::new("media-annotator", picture(STAIR), 1.5, marks)
                .on_change(move |marks, _, cx| set(&changed, marks.to_vec(), cx)),
        ),
    ))
    .child(div().text_size(cx.theme().text_size(ely_gpui_component::theme::TextSize::Sm)).text_color(cx.theme().colors.fg_muted).child(count))
}
