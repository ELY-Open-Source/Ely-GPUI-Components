use ely_gpui_component::misc::{IframeEmbed, WebSource, WebView};
use gpui::{App, IntoElement, ParentElement, Styled, div, px};

use crate::probe::probe;
use crate::ui::{blocked, section};

const PAGE: &str = r#"<!doctype html>
<meta charset="utf-8">
<style>
  :root { color-scheme: light dark; font: 15px -apple-system, sans-serif; }
  body { margin: 0; height: 100vh; display: grid; place-items: center;
         background: linear-gradient(135deg, #5b8cff33, #b35bff33); }
  h1 { font-size: 22px; margin: 0 0 6px; }
  p { margin: 0; opacity: .7; }
</style>
<main>
  <h1>Hello from WebKit</h1>
  <p>HTML the owner hands in, drawn by a native web view.</p>
</main>"#;

pub fn render(cx: &App) -> impl IntoElement + use<> {
    div()
        .child(
            section(
                "WebView / HTMLPreview",
                "A native web view laid over its box: an address, or HTML the owner hands in, which makes it an HTML preview. It draws above everything gpui paints and follows the box each frame.",
                cx,
            )
            .child(probe(
                "web",
                div()
                    .w(px(480.0))
                    .h(px(200.0))
                    .child(WebView::new("web", WebSource::Html(PAGE.into()))),
            )),
        )
        .child(
            section(
                "IframeEmbed / EmbedCard",
                "A page from another address in a frame of its own, with Open in browser. EmbedCard is chat::LinkPreviewCard, shown on the AI Chat page.",
                cx,
            )
            .child(div().w(px(480.0)).child(IframeEmbed::new(
                "iframe",
                "https://example.com",
                16.0 / 9.0,
            ))),
        )
        .child(
            section(
                "PrintButton · ShareSheet",
                "ShareSheet is buttons::ShareButton, shown on the Buttons & Actions page.",
                cx,
            )
            .child(blocked(
                "PrintButton: gpui has no print API.",
                cx,
            )),
        )
}
