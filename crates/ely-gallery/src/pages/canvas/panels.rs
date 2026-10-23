use ely_gpui_component::canvas::{
    AlignmentToolbar, AssetPanel, Corner, Frame, InspectorPanel, LayerPanel, Shape, ShapeKind,
    aligned,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::tools::studio;
use crate::ui::{change, section};

fn asset(
    key: &'static str,
    name: &'static str,
    kind: ShapeKind,
    w: f32,
    h: f32,
    hue: usize,
) -> Shape {
    Shape::new(key, name, kind, Frame::new(0.0, 0.0, w, h)).hue(hue)
}

fn shapes() -> Vec<Shape> {
    vec![
        asset("button", "Button", ShapeKind::Rect, 120.0, 40.0, 0),
        asset("dot", "Dot", ShapeKind::Ellipse, 40.0, 40.0, 1),
        asset("hexagon", "Hexagon", ShapeKind::Polygon(6), 60.0, 60.0, 2),
        asset(
            "pointer",
            "Pointer",
            ShapeKind::Arrow(Corner::BottomRight),
            80.0,
            40.0,
            3,
        ),
        asset(
            "divider",
            "Divider",
            ShapeKind::Line(Corner::TopRight),
            120.0,
            0.0,
            4,
        ),
        asset(
            "heading",
            "Heading",
            ShapeKind::Text("Heading".into()),
            160.0,
            32.0,
            5,
        ),
    ]
}

fn notes() -> Vec<Shape> {
    vec![
        asset(
            "idea",
            "Idea",
            ShapeKind::Note("An idea".into()),
            120.0,
            120.0,
            6,
        ),
        asset(
            "todo",
            "To do",
            ShapeKind::Note("To do".into()),
            120.0,
            120.0,
            7,
        ),
    ]
}

pub fn arrange(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let studio = studio(window, cx);
    let now = studio.read(cx);
    let (shapes_now, selected) = (now.shapes.clone(), now.selected.clone());
    let chosen: Vec<Shape> = shapes_now
        .iter()
        .filter(|shape| selected.contains(&shape.key))
        .cloned()
        .collect();
    let [picked, shown, named, stacked, lined, edited, placed] = [(); 7].map(|_| studio.clone());
    let layers = LayerPanel::new("canvas-layers", shapes_now)
        .selected(selected.clone())
        .on_select(move |keys, _, cx| change(&picked, cx, |studio| studio.selected = keys.to_vec()))
        .on_show(move |keys, _, cx| {
            change(&shown, cx, |studio| {
                for shape in &mut studio.shapes {
                    shape.hidden = !keys.contains(&shape.key);
                }
                studio.record("Show or hide".into());
            })
        })
        .on_rename(move |key, name, _, cx| {
            change(&named, cx, |studio| {
                let shape = studio
                    .shapes
                    .iter_mut()
                    .find(|shape| shape.key == *key)
                    .expect("a listed shape");
                shape.name = name.clone();
                studio.record(format!("Rename {name}"));
            })
        })
        .on_restack(move |order, _, cx| {
            change(&stacked, cx, |studio| {
                studio.shapes.sort_by_key(|shape| {
                    order
                        .iter()
                        .position(|key| *key == shape.key)
                        .expect("every layer restacked")
                });
                studio.record("Restack".into());
            })
        });
    let toolbar =
        AlignmentToolbar::new("canvas-align", selected.len()).on_align(move |align, _, cx| {
            change(&lined, cx, |studio| {
                let keys: Vec<SharedString> = studio.selected.clone();
                let frames: Vec<Frame> = studio
                    .shapes
                    .iter()
                    .filter(|shape| keys.contains(&shape.key))
                    .map(|shape| shape.frame)
                    .collect();
                let moved = aligned(&frames, align);
                studio
                    .shapes
                    .iter_mut()
                    .filter(|shape| keys.contains(&shape.key))
                    .zip(moved)
                    .for_each(|(shape, frame)| shape.frame = frame);
                studio.record(align.words().into());
            })
        });
    let inspector = InspectorPanel::new("canvas-inspector", chosen, move |shape, _, cx| {
        change(&edited, cx, |studio| {
            let at = studio
                .shapes
                .iter()
                .position(|each| each.key == shape.key)
                .expect("a listed shape");
            let words = format!("Edit {}", shape.name);
            studio.shapes[at] = shape;
            studio.record(words);
        })
    });
    let assets = AssetPanel::new("canvas-assets")
        .group("Shapes", shapes())
        .group("Notes", notes())
        .on_pick(move |shape, _, cx| {
            change(&placed, cx, |studio| {
                studio.made += 1;
                let key = SharedString::from(format!("made-{}", studio.made));
                let mut copy = shape.clone();
                copy.key = key.clone();
                copy.frame.x = 280.0 + 12.0 * studio.made as f32;
                copy.frame.y = 180.0 + 12.0 * studio.made as f32;
                studio.shapes.push(copy);
                studio.selected = vec![key];
                studio.record(format!("Place {}", shape.name));
            })
        });
    section(
        "LayerPanel · InspectorPanel · AlignmentToolbar · AssetPanel",
        "Panels on the drawing above. The layers list it topmost first: a box shows or hides a layer, F2 renames it, a drag restacks it. The inspector edits the one selected; the toolbar lines up or spreads several; an asset pressed lands on the canvas. Each change is a step in the history.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(div().w(px(240.)).h(px(280.)).child(layers))
            .child(
                div()
                    .w(px(300.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(toolbar)
                    .child(inspector),
            )
            .child(div().w(px(256.)).child(assets)),
    )
}
