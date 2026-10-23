use ely_gpui_component::{
    agent::{ArtifactPanel, BrowserPreview, ComputerUseViewer, LivePreview},
    buttons::{Button, ButtonVariant},
    chat::CodeBlock,
    documents::MarkdownRenderer,
    primitives::IconName,
    theme::{ActiveTheme, TextSize},
    typography::format::plural,
};
use gpui::{
    App, ClipboardItem, FontWeight, IntoElement, ParentElement, Styled, Window, div, point, px,
};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const FRAMES: [&str; 3] = [
    asset!("frames/frame-1.jpg"),
    asset!("frames/frame-2.jpg"),
    asset!("frames/frame-3.jpg"),
];

const ADDRESSES: [&str; 3] = [
    "localhost:5173/navigation",
    "localhost:5173/git",
    "localhost:5173/chat",
];

/// Where the agent points on the screen, and what it does there.
const ACTIONS: [(f32, f32, &str); 3] = [
    (0.42, 0.31, "Click Full name"),
    (0.42, 0.31, "Type Ada Lovelace"),
    (0.34, 0.82, "Click Password"),
];

pub fn watching(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let shown = keep("agent-frame", || 2_usize, window, cx);
    let step = keep("agent-pointer", || 0_usize, window, cx);
    let (now, at) = (*shown.read(cx), *step.read(cx));
    let (x, y, action) = ACTIONS[at];
    section(
        "BrowserPreview / ScreenshotStream / ComputerUseViewer",
        "What an agent sees as it works, in frames from the host: a browser with its address and the frames so far, a press on one shows it; and a screen with where the agent points, a ring for each new point, and what it does there.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_start()
            .gap_6()
            .child(
                div().w(px(480.)).child(
                    BrowserPreview::new("agent-browser", ADDRESSES[now], FRAMES, now)
                        .loading(now == FRAMES.len() - 1)
                        .on_show(move |ix, _, cx| set(&shown, ix, cx)),
                ),
            )
            .child(
                div()
                    .w(px(420.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        ComputerUseViewer::new("agent-screen", asset!("frames/screen.jpg"))
                            .pointer(point(x, y))
                            .action(action),
                    )
                    .child(probe(
                        "agent-pointer-next",
                        Button::new("agent-pointer-next", "Next action")
                            .variant(ButtonVariant::Secondary)
                            .on_click(move |_, _, cx| {
                                let next = (*step.read(cx) + 1) % ACTIONS.len();
                                set(&step, next, cx)
                            }),
                    )),
            ),
    )
}

const NOTE: &str = "# Lift\n\nA **lift** blends a color toward white. Dark themes lift their accents further, because saturated color on charcoal looks louder.\n\n- Accents lift by 18%\n- Surfaces lift by 4%";

const LIFT: &str =
    "pub fn lift(color: Hsla, by: f32) -> Hsla {\n    color.mix(white(), by.clamp(0.0, 1.0))\n}";

/// A small page as the host would draw it.
fn page(builds: usize, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    div()
        .p_6()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .text_size(theme.text_size(TextSize::Xl))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Lift"),
        )
        .child(
            div()
                .text_color(theme.colors.fg_muted)
                .child("Blend a color toward white."),
        )
        .child(
            div()
                .flex()
                .child(Button::new("agent-page-try", "Try it").variant(ButtonVariant::Primary)),
        )
        .child(
            div()
                .text_size(theme.text_size(TextSize::Xs))
                .text_color(theme.colors.fg_subtle)
                .child(format!("Built {}", plural(builds as u64, "time", "times"))),
        )
}

pub fn artifacts(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let version = keep("agent-version", || 2_usize, window, cx);
    let builds = keep("agent-builds", || 1_usize, window, cx);
    let (now, built) = (*version.read(cx), *builds.read(cx));
    section(
        "ArtifactPanel / ArtifactVersionSwitcher / LivePreview",
        "What an agent made, in a panel of its own: the version shown among all, copy and download, the artifact as preview or source; the version switcher is chat::BranchNavigator. A live preview holds the host's page with its address and reload.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_start()
            .gap_6()
            .child(
                div().w(px(460.)).h(px(360.)).child(
                    ArtifactPanel::new("agent-note", "Notes on lift", IconName::FileText)
                        .version(now, 3, move |to, _, cx| set(&version, to, cx))
                        .preview(div().p_4().child(MarkdownRenderer::new("agent-note-text", NOTE)))
                        .source(div().p_3().child(CodeBlock::new("agent-note-source", NOTE).language("markdown")))
                        .on_copy(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string(NOTE.to_string()))),
                ),
            )
            .child(
                div().w(px(420.)).h(px(360.)).child(
                    ArtifactPanel::new("agent-code", "lift.rs", IconName::Code)
                        .source(div().p_3().child(CodeBlock::new("agent-code-source", LIFT).language("rust"))),
                ),
            ),
    )
    .child(
        div()
            .w(px(460.))
            .h(px(280.))
            .child(
                LivePreview::new("agent-live", "localhost:5173/lift", page(built, cx))
                    .on_reload(move |_, cx| {
                        let next = *builds.read(cx) + 1;
                        set(&builds, next, cx)
                    }),
            ),
    )
}
