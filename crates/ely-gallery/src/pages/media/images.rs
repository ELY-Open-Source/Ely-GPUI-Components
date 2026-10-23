use ely_gpui_component::{
    media::{ImageThumbnail, ImageViewer},
    overlays::{Lightbox, Slide},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, picture, resource, row, section, set},
};

/// The gallery's pictures: a file, its width over height, a caption, and a corner label.
const PICTURES: [(&str, f32, &str, &str); 5] = [
    (asset!("atrium.jpg"), 1.5, "An atrium, noon", ""),
    (asset!("dunes.jpg"), 1.5, "Dunes at first light", ""),
    (
        asset!("atrium-olive.jpg"),
        1.5,
        "An olive tree by the stair",
        "HDR",
    ),
    (asset!("dunes-grass.jpg"), 1.5, "Grass on the ridge", ""),
    (asset!("dunes-square.jpg"), 1.0, "The long shadow", "+2"),
];

pub fn viewer(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "ImageViewer",
        "A picture to look at closely. It fits at first; the buttons, plus and minus, or Cmd-scroll zoom it, and 0 and 1 go back to fit and actual size, as a double press does. Drag it or use the arrows once it overhangs; R turns it.",
        cx,
    )
    .child(probe(
        "media-viewer",
        div().w(px(640.)).h(px(440.)).child(ImageViewer::new("media-viewer", resource(PICTURES[0].0))),
    ))
}

pub fn thumbnails(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let shown = keep("media-shown", || None::<usize>, window, cx);
    let opened = keep("media-last", || 0usize, window, cx);
    let last = *opened.read(cx);
    let thumbs = PICTURES
        .iter()
        .enumerate()
        .map(|(ix, (path, ratio, _, label))| {
            let (open, mark) = (shown.clone(), opened.clone());
            let thumb = ImageThumbnail::new(
                SharedString::from(format!("media-thumb-{ix}")),
                picture(*path),
                *ratio,
            )
            .selected(ix == last)
            .on_click(move |_, cx| {
                set(&mark, ix, cx);
                set(&open, Some(ix), cx);
            });
            let thumb = if label.is_empty() {
                thumb
            } else {
                thumb.label(*label)
            };
            div().w(px(96. * ratio + 6.)).child(thumb)
        });
    let overlay = (*shown.read(cx)).map(|at| {
        let (step, close) = (shown.clone(), shown.clone());
        Lightbox::new(
            "media-lightbox",
            PICTURES.map(|(path, _, caption, _)| Slide::new(picture(path), caption)),
            at,
            move |_, cx| set(&close, None, cx),
        )
        .on_step(move |to, _, cx| set(&step, Some(to), cx))
    });
    section(
        "ImageThumbnail / Lightbox → overlays::Lightbox",
        "Pictures small, each in its own shape, with a label when one helps. The one last opened keeps a ring; a press or Enter opens the lightbox, which steps through them all.",
        cx,
    )
    .child(probe("media-thumbs", row().items_end().children(thumbs)))
    .children(overlay)
}
