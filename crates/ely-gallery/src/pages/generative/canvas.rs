use ely_gpui_component::generative::{InpaintCanvas, MaskBrush, MaskStroke};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

/// A mask over the olive tree, painted before the demo opens.
fn tree() -> Vec<MaskStroke> {
    vec![MaskStroke {
        points: vec![
            (0.07, 0.18),
            (0.12, 0.34),
            (0.09, 0.52),
            (0.13, 0.7),
            (0.1, 0.86),
        ],
        radius: 0.07,
        erase: false,
    }]
}

pub fn canvas(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let strokes = keep("gen-mask", tree, window, cx);
    let brush = keep("gen-brush", || (0.05_f32, false), window, cx);
    let (now, (radius, erase)) = (strokes.read(cx).clone(), *brush.read(cx));
    let (painted, undone, cleared) = (strokes.clone(), strokes.clone(), strokes);
    let (keyed, set_brush) = (brush.clone(), brush);
    let count = now.len();
    section(
        "InpaintCanvas / MaskBrush",
        "A picture to repaint in part: paint a mask where it should change, erase where it went too far; the mask washes over the picture. On the canvas, [ and ] size the brush and X swaps painting for erasing. Undo takes the last stroke, Clear takes them all.",
        cx,
    )
    .child(
        div()
            .w(px(460.))
            .flex()
            .flex_col()
            .gap_3()
            .child(probe(
                "gen-canvas",
                div().w(px(460.)).child(
                    InpaintCanvas::new("gen-canvas", asset!("atrium-olive.jpg"), 1.5, now)
                        .brush(radius, erase)
                        .on_stroke(move |stroke, _, cx| {
                            let mut next = painted.read(cx).clone();
                            next.push(stroke);
                            set(&painted, next, cx)
                        })
                        .on_brush(move |radius, erase, _, cx| set(&keyed, (radius, erase), cx)),
                ),
            ))
            .child(
                MaskBrush::new("gen-mask-brush", radius, erase, count)
                    .on_brush(move |radius, erase, _, cx| set(&set_brush, (radius, erase), cx))
                    .on_undo(move |_, cx| {
                        let mut next = undone.read(cx).clone();
                        next.pop();
                        set(&undone, next, cx)
                    })
                    .on_clear(move |_, cx| set(&cleared, Vec::new(), cx)),
            ),
    )
}
