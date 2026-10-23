use std::time::Duration;

use ely_gpui_component::{
    chat::{
        AudioMessage, CodeBlock, FileMessage, ImageGrid, ImageMessage, LinkPreviewCard,
        VideoMessage,
    },
    documents::MarkdownRenderer,
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::ui::{keep, noise, section, set};

const SHORT: &str = "fn lift(c: f32, t: f32) -> f32 {\n    c + t * (1.0 - c)\n}";

const LONG: &str = "/// Every accent in the library, lifted for its theme.\npub struct Accents {\n    base: Hsla,\n    light: f32,\n    dark: f32,\n}\n\nimpl Accents {\n    pub fn new(base: Hsla) -> Self {\n        Self { base, light: 0.5, dark: 0.62 }\n    }\n\n    /// The accent as the theme shows it.\n    pub fn shown(&self, dark: bool) -> Hsla {\n        let t = if dark { self.dark } else { self.light };\n        lift(self.base, t)\n    }\n\n    /// A hover, one step louder.\n    pub fn hover(&self, dark: bool) -> Hsla {\n        lift(self.shown(dark), 0.08)\n    }\n\n    /// A press, two steps louder.\n    pub fn press(&self, dark: bool) -> Hsla {\n        lift(self.shown(dark), 0.16)\n    }\n}";

pub fn code(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    section(
        "CodeBlock",
        "Code in a message: its language, copy, and the host's run, apply and download. Long code folds to its first lines until opened.",
        cx,
    )
    .child(
        div()
            .w(px(620.))
            .flex()
            .flex_col()
            .gap_4()
            .child(
                CodeBlock::new("chat-code-short", SHORT)
                    .language("rust")
                    .on_run(|_, _| log::info!("gallery: run"))
                    .on_apply(|_, _| log::info!("gallery: apply"))
                    .on_download(|_, _| log::info!("gallery: download")),
            )
            .child(CodeBlock::new("chat-code-long", LONG).language("rust")),
    )
}

const RICH: &str = "| Theme | Lift | Reads as |\n| --- | ---: | --- |\n| Light | 0.50 | half toward white |\n| Dark | 0.62 | a little louder |\n\n$$c' = c + t\\,(1 - c)$$";

pub fn rich(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    section(
        "MathBlock / TableBlock / MermaidRenderer",
        "Math is typography::Latex and tables are the markdown renderer's, in a message as in a document. A diagram arrives as the picture the host drew from its source.",
        cx,
    )
    .child(
        div()
            .w(px(620.))
            .flex()
            .flex_col()
            .gap_4()
            .child(MarkdownRenderer::new("chat-rich", RICH))
            .child(ImageMessage::new("chat-diagram", asset!("flow.svg"), 560.0, 160.0).caption("Drawn by the host from a diagram's source")),
    )
}

pub fn media(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let mut wave = noise(7);
    let peaks: Vec<f32> = (0..56)
        .map(|ix| {
            let swell = (ix as f32 / 56.0 * std::f32::consts::PI).sin();
            (0.25 + 0.75 * swell * wave() as f32).clamp(0.08, 1.0)
        })
        .collect();
    let playing = keep(
        "chat-audio",
        || (Duration::from_secs(12), false),
        window,
        cx,
    );
    let (played, on) = *playing.read(cx);
    let (toggle, seek) = (playing.clone(), playing);
    section(
        "ImageMessage / ImageGrid / FileMessage / AudioMessage / VideoMessage / LinkPreviewCard",
        "What a message can carry: a picture in its own shape, pictures as a grid, a file, a voice note to play and scrub, a video over the host's frames, and a link unfolded.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_6()
                .child(
                    div()
                        .w(px(320.))
                        .flex()
                        .flex_col()
                        .gap_4()
                        .child(ImageMessage::new("chat-image", asset!("atrium.jpg"), 960.0, 640.0).caption("The atrium at noon"))
                        .child(FileMessage::new("chat-file", "lift-notes.pdf", 482_000).on_download(|_, _| log::info!("gallery: download file")))
                        .child(
                            AudioMessage::new("chat-audio", peaks, Duration::from_secs(42))
                                .played(played, on)
                                .on_toggle(move |next, _, cx| {
                                    let at = toggle.read(cx).0;
                                    set(&toggle, (at, next), cx)
                                })
                                .on_seek(move |share, _, cx| {
                                    let on = seek.read(cx).1;
                                    set(&seek, (Duration::from_secs_f32(42.0 * share), on), cx)
                                }),
                        ),
                )
                .child(
                    div()
                        .w(px(280.))
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(ImageMessage::new("chat-narrow-image", asset!("atrium-stair.jpg"), 600.0, 400.0).caption("In a narrow column, in its own shape"))
                        .child(VideoMessage::new("chat-narrow-video", asset!("dunes-sun.jpg"), 600.0, 400.0, Duration::from_secs(31)))
                        .child(FileMessage::new("chat-narrow-file", "a-long-name-for-the-lift-notes.pdf", 482_000)),
                ),
            )
            .child(
                div()
                    .w(px(520.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(ImageGrid::new(
                        "chat-grid",
                        [
                            asset!("dunes.jpg"),
                            asset!("atrium-stair.jpg"),
                            asset!("dunes-sun.jpg"),
                            asset!("atrium-olive.jpg"),
                            asset!("dunes-grass.jpg"),
                        ],
                    ))
                    .child(VideoMessage::new("chat-video", asset!("dunes.jpg"), 960.0, 640.0, Duration::from_secs(84)).on_toggle(|next, _, _| log::info!("gallery: video playing {next}")))
                    .child(
                        LinkPreviewCard::new("chat-link", "example.com", "Color, in light and dark")
                            .description("How one accent reads the same on paper and on charcoal.")
                            .picture(asset!("dunes-square.jpg"))
                            .on_open(|_, _| log::info!("gallery: open link")),
                    ),
            )
    )
}
