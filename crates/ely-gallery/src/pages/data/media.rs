use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    data_display::{BeforeAfter, Carousel, Gallery, Watermark},
    overlays::Slide,
    primitives::Image,
    theme::{ActiveTheme, Radius, TextSize},
    typography::{Caption, Paragraph, Title},
};
use gpui::{App, IntoElement, ObjectFit, ParentElement, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, picture, section, set},
};

const PICTURES: [(&str, &str); 6] = [
    (asset!("dunes-sun.jpg"), "Sunset over the dunes"),
    (asset!("atrium-stair.jpg"), "A stair that turns"),
    (asset!("dunes-grass.jpg"), "Marram grass"),
    (asset!("atrium-olive.jpg"), "An olive by the glass"),
    (asset!("dunes.jpg"), "Dunes at first light"),
    (asset!("atrium.jpg"), "An atrium, noon"),
];

pub fn carousel(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let colors = &theme.colors;
    let slides = PICTURES[..4]
        .iter()
        .enumerate()
        .map(|(ix, (path, caption))| {
            div()
                .relative()
                .size_full()
                .child(
                    Image::new(("carousel-picture", ix), picture(*path))
                        .fit(ObjectFit::Cover)
                        .size_full(),
                )
                .child(
                    div()
                        .absolute()
                        .left_4()
                        .bottom_4()
                        .px_2()
                        .py_1()
                        .rounded(theme.radius(Radius::Sm))
                        .bg(colors.media_backdrop.opacity(0.5))
                        .text_color(colors.on_media)
                        .text_size(theme.text_size(TextSize::Sm))
                        .child(*caption),
                )
        });
    section(
        "Carousel",
        "Slides one at a time, gliding between them; past the last comes the first. Arrows show on hover, the dots stretch toward where it goes, and Left and Right step it. With autoplay it turns on its own and holds while pointed at.",
        cx,
    )
    .child(div().flex().child(probe(
        "carousel",
        Carousel::new("carousel").w(px(560.)).h_72().children(slides),
    )))
}

pub fn gallery(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "Gallery",
        "Pictures in square tiles. Press one, or Enter on a focused tile, to open the lightbox there.",
        cx,
    )
    .child(div().flex().child(probe(
        "gallery",
        div().w(px(420.)).child(Gallery::new(
            "gallery",
            PICTURES.map(|(path, caption)| Slide::new(picture(path), caption)),
        )),
    )))
}

pub fn before_after(cx: &mut App) -> impl IntoElement + use<> {
    let picture = |id: &'static str, path: &'static str| {
        Image::new(id, picture(path))
            .fit(ObjectFit::Cover)
            .size_full()
    };
    section(
        "BeforeAfter",
        "One picture twice: flat as it came, and graded. Drag the divider, press anywhere on the picture, or use Left and Right.",
        cx,
    )
    .child(div().flex().child(probe(
        "before-after",
        BeforeAfter::new(
            "before-after",
            picture("before-flat", asset!("atrium-before.jpg")),
            picture("before-graded", asset!("atrium.jpg")),
        )
        .w(px(480.))
        .h(px(320.)),
    )))
}

pub fn watermark(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let read = keep("watermark-read", || false, window, cx);
    let (done, mark) = (*read.read(cx), read.clone());
    let theme = cx.theme();
    section(
        "Watermark",
        "Faint text over a page, for drafts and shared screens. The rows run level, and presses reach the page below.",
        cx,
    )
    .child(
        Watermark::new("watermark", "Confidential · Ada Lovelace")
            .w(px(560.))
            .p_6()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .child(Title::new("Plan for the fourth quarter"))
            .child(
                div().mt_3().child(Paragraph::new(
                    "Three launches, one migration, and a quieter support queue. The launches share a date so the notes can go out once; the migration waits for the new year.",
                )),
            )
            .child(
                div()
                    .mt_4()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        Button::new("watermark-read", "Mark as read")
                            .variant(ButtonVariant::Secondary)
                            .on_click(move |_, _, cx| set(&mark, true, cx)),
                    )
                    .child(Caption::new(if done { "Read." } else { "Unread." })),
            ),
    )
}
