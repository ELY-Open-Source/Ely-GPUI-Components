use ely_gpui_component::{
    canvas::{
        Artboard, Corner, Frame, Guide, InfiniteCanvas, MiniMap, Shape, ShapeKind, Viewport,
        ZoomControls,
    },
    theme::{ActiveTheme, Radius},
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

/// The demo's view, in pixels.
const VIEW: (f32, f32) = (820.0, 460.0);

fn boards() -> Vec<Artboard> {
    vec![
        Artboard::new("home", "Home · 390", Frame::new(0.0, 0.0, 390.0, 640.0)),
        Artboard::new(
            "detail",
            "Detail · 390",
            Frame::new(470.0, 0.0, 390.0, 640.0),
        ),
    ]
}

fn shapes() -> Vec<Shape> {
    vec![
        Shape::new(
            "hero",
            "Hero",
            ShapeKind::Rect,
            Frame::new(24.0, 32.0, 342.0, 180.0),
        )
        .hue(0),
        Shape::new(
            "title",
            "Title",
            ShapeKind::Text("Studio Atrium".into()),
            Frame::new(24.0, 232.0, 342.0, 32.0),
        ),
        Shape::new(
            "dot",
            "Status",
            ShapeKind::Ellipse,
            Frame::new(24.0, 290.0, 56.0, 56.0),
        )
        .hue(5),
        Shape::new(
            "badge",
            "Badge",
            ShapeKind::Polygon(6),
            Frame::new(100.0, 290.0, 56.0, 56.0),
        )
        .hue(2),
        Shape::new(
            "rule",
            "Rule",
            ShapeKind::Line(Corner::BottomRight),
            Frame::new(24.0, 380.0, 342.0, 0.0),
        )
        .hue(4),
        Shape::new(
            "flow",
            "Flow",
            ShapeKind::Arrow(Corner::BottomRight),
            Frame::new(390.0, 120.0, 80.0, 0.0),
        )
        .hue(3),
        Shape::new(
            "sketch",
            "Sketch",
            ShapeKind::Path {
                points: vec![
                    (0.0, 40.0),
                    (40.0, 0.0),
                    (80.0, 30.0),
                    (120.0, 10.0),
                    (160.0, 50.0),
                ],
                width: 2.0,
            },
            Frame::new(494.0, 60.0, 160.0, 50.0),
        )
        .hue(1),
        Shape::new(
            "note",
            "Note",
            ShapeKind::Note("Ask Chloé about the stair".into()),
            Frame::new(494.0, 150.0, 180.0, 120.0),
        )
        .hue(2),
    ]
}

pub fn plane(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let view = keep(
        "canvas-view",
        || Viewport::new(-60.0, -60.0, 0.7),
        window,
        cx,
    );
    let now = *view.read(cx);
    let frames: Vec<Frame> = boards().iter().map(|board| board.frame).collect();
    let whole = frames
        .iter()
        .skip(1)
        .fold(frames[0], |whole, frame| whole.union(frame));
    let (panned, zoomed, fitted, steered) = (view.clone(), view.clone(), view.clone(), view);
    let theme = cx.theme();
    let canvas = InfiniteCanvas::new("canvas-plane", now)
        .artboards(boards())
        .shapes(shapes())
        .guides([Guide::Vertical(24.0), Guide::Horizontal(212.0)])
        .rulers()
        .on_viewport(move |next, _, cx| set(&panned, next, cx))
        .layer(
            div()
                .absolute()
                .bottom_3()
                .right_3()
                .p_1()
                .rounded(theme.radius(Radius::Md))
                .bg(theme.colors.surface)
                .border_1()
                .border_color(theme.colors.border)
                .child(
                    ZoomControls::new("canvas-zoom", now.zoom)
                        .on_zoom(move |zoom, _, cx| {
                            let now = *zoomed.read(cx);
                            let middle = (VIEW.0 / 2.0, VIEW.1 / 2.0);
                            set(&zoomed, now.zoomed(zoom / now.zoom, middle), cx)
                        })
                        .on_fit(move |_, cx| {
                            set(&fitted, Viewport::fitting(whole, VIEW, 48.0), cx)
                        }),
                ),
        )
        .layer(
            div().absolute().bottom_3().left_8().child(
                MiniMap::new("canvas-map", frames, now, VIEW)
                    .on_viewport(move |next, _, cx| set(&steered, next, cx)),
            ),
        );
    section(
        "InfiniteCanvas · CanvasGrid / Guides · Ruler · ArtboardFrame · ZoomControls / ZoomIndicator · MiniMap",
        "An endless plane: a dot grid that thins as it zooms out, rulers in canvas units, guides, and artboards with their names. Drag to pan, scroll to move, and hold Command to zoom about the pointer. The zoom steps in fixed stops, and the minimap moves the view to a press.",
        cx,
    )
    .child(probe(
        "canvas-plane",
        div()
            .w(px(VIEW.0))
            .h(px(VIEW.1))
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .overflow_hidden()
            .child(canvas),
    ))
}
