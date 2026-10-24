use std::rc::Rc;

use ely_gpui_component::{
    documents::{
        Chapter, DocPage, DocumentViewer, EpubReader, Footnote, Glossary, MarkdownRenderer,
        PageHit, PageNote, PrintPreview, PrintSettings, ReadingProgress, RichTextEditor,
        TooltipTerm, ZenMode, markdown_highlights, outline,
    },
    editor::FindWidget,
    forms::TextInput,
    navigation::{Sections, TableOfContents},
    theme::{ActiveTheme, Radius},
};
use gpui::{
    App, Bounds, Entity, InteractiveElement, IntoElement, ParentElement, ScrollHandle,
    StatefulInteractiveElement, Styled, Window, div, point, px, size,
};
use serde::Deserialize;

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const PAGES: [&str; 2] = [asset!("pages/lift-1.png"), asset!("pages/lift-2.png")];
const SLIDES: [&str; 2] = [asset!("pages/slide-1.png"), asset!("pages/slide-2.png")];
const WORDS: &str = include_str!("../../../assets/pages/lift.json");

const GUIDE: &str = "# Lift\n\nA lift blends a color toward white.\n\n## In light\n\nPaper white under ink near black; accents at half a lift.\n\n## In dark\n\nWarm charcoal under off-white text; accents lift further, so they read with the same weight.\n\n### Surfaces\n\nA card sits one lift above the page, a menu one above the card.\n\n## In motion\n\nHover, press and focus are lifts of the accent, eased on one scale of durations.\n\n## Closing\n\nWhatever the hue, a step of lift means the same step of emphasis.";

/// The words the app found on its pages, with their boxes in points.
#[derive(Deserialize)]
struct PageWords {
    page: usize,
    words: Vec<(String, f32, f32, f32, f32)>,
}

/// Every word on the pages that holds `query`, as a host's text layer would find them.
fn search(query: &str) -> Vec<PageHit> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let pages: Vec<PageWords> = serde_json::from_str(WORDS).expect("the page words parse");
    let mut hits = Vec::new();
    for page in pages {
        for (word, x, y, width, height) in page.words {
            if word.to_lowercase().contains(&query) {
                hits.push(PageHit {
                    page: page.page,
                    bounds: Bounds::new(point(x, y), size(width, height)),
                });
            }
        }
    }
    hits
}

fn pages(paths: &[&'static str], width: f32, height: f32) -> Rc<Vec<DocPage>> {
    Rc::new(
        paths
            .iter()
            .map(|path| DocPage {
                source: (*path).into(),
                width,
                height,
            })
            .collect(),
    )
}

fn field(
    key: &'static str,
    hint: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TextInput> {
    window.use_keyed_state(key, cx, move |window, cx| {
        TextInput::new(window, cx).placeholder(hint)
    })
}

pub fn reading(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let scroll = keep("read-scroll", ScrollHandle::new, window, cx)
        .read(cx)
        .clone();
    let sections = window.use_keyed_state("read-sections", cx, {
        let scroll = scroll.clone();
        move |_, _| Sections::new(scroll)
    });
    let toc = outline(GUIDE).into_iter().fold(
        TableOfContents::new("read-toc", &sections),
        |toc, (value, text, depth)| toc.entry(value, text, depth),
    );
    let query = field("read-query", "Find in document", window, cx);
    let hits = search(query.read(cx).text());
    let current = keep("read-hit", || None::<usize>, window, cx);
    let now_hit = current.read(cx).filter(|hit| *hit < hits.len());
    let total = hits.len();
    let find =
        FindWidget::new("read-find", &query, now_hit, total).on_step(move |forward, _, cx| {
            let next = match current.read(cx).filter(|hit| *hit < total) {
                Some(hit) if forward => (hit + 1) % total,
                Some(hit) => (hit + total - 1) % total,
                None if forward => 0,
                None => total - 1,
            };
            set(&current, Some(next), cx)
        });
    let zoom = keep("read-zoom", || 0.75_f32, window, cx);
    let notes = keep(
        "read-notes",
        || {
            vec![PageNote {
                page: 0,
                at: point(470.0, 180.0),
                text: "Gamma, not linear light.".into(),
            }]
        },
        window,
        cx,
    );
    let (now_zoom, now_notes) = (*zoom.read(cx), notes.read(cx).clone());
    let slide_zoom = keep("read-slide-zoom", || 0.5_f32, window, cx);
    let now_slide_zoom = *slide_zoom.read(cx);
    let print = keep("read-print", PrintSettings::default, window, cx);
    let now_print = *print.read(cx);
    let zen = keep("read-zen", || false, window, cx);
    let now_zen = *zen.read(cx);
    let draft = window.use_keyed_state("read-draft", cx, |window, cx| {
        let mut draft = TextInput::new(window, cx).multi_line(6, 18).highlighter(markdown_highlights).label("Draft");
        draft.set_text("# A quiet page\n\nEverything else folds away while you write. Press Escape to come back.", cx);
        draft
    });
    let theme = cx.theme();
    let chapters = vec![
        Chapter { title: "A color and its lift".into(), markdown: "# A color and its lift\n\nA lift blends a color toward white. Half a lift sits midway between the color and white, and a full lift is white itself.\n\nEvery accent in the library is a base color and a lift. The base carries the hue; the lift carries how loud the color is.".into() },
        Chapter { title: "Tokens in one place".into(), markdown: "# Tokens in one place\n\nComponents never write a color, a size or a radius. They read tokens from the theme.\n\nThe palette has two sides. Neither is the other inverted.".into() },
        Chapter { title: "Motion that settles".into(), markdown: "# Motion that settles\n\nMotion in the library is short and settles without bounce. When reduced motion is on, every repeating motion holds still.".into() },
    ];
    div()
        .child(
            section(
                "MarkdownRenderer / TableOfContents / DocumentOutline / ReadingProgress",
                "Markdown as a document, its headings marked for a table of contents beside it: the section in view is lit as it scrolls, and a press brings a section to the top. The bar above fills as the text scrolls. DocumentOutline is the same TableOfContents.",
                cx,
            )
            .child(
                div()
                    .w(px(760.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(ReadingProgress::new(&scroll))
                    .child(
                div()
                    .flex()
                    .gap_6()
                    .h(px(320.))
                    .child(div().w(px(200.)).child(toc))
                    .child(
                        div()
                            .id("read-guide")
                            .flex_1()
                            .overflow_y_scroll()
                            .track_scroll(&scroll)
                            .child(MarkdownRenderer::new("read-guide-text", GUIDE).sections(&sections)),
                    ),
            ),
            ),
        )
        .child(
            section(
                "DocumentViewer / PDFViewer / PageThumbnailList",
                "Pages the app drew, on a desk: the page in view and its count, zoom by steps or to the width, the pages as thumbnails beside, a find bar whose hits light on the page, and notes pinned where a press puts them. The app reads its PDF; the viewer shows it.",
                cx,
            )
            .child(probe(
                "read-viewer",
                div().w(px(840.)).h(px(520.)).child(
                    DocumentViewer::new("read-viewer", "on-lift.pdf", pages(&PAGES, 612.0, 792.0))
                        .zoom(now_zoom, move |next, _, cx| set(&zoom, next, cx))
                        .thumbnails()
                        .find(find, hits, now_hit)
                        .notes(now_notes, move |page, at, _, cx| {
                            let mut next = notes.read(cx).clone();
                            next.push(PageNote { page, at, text: "A new note.".into() });
                            set(&notes, next, cx);
                        }),
                ),
            )),
        )
        .child(
            section(
                "OfficePreview",
                "A slide deck, a sheet or a document the app turned into pages, shown by the same viewer.",
                cx,
            )
            .child(
                div().w(px(760.)).h(px(420.)).child(
                    DocumentViewer::new("read-slides", "release.pptx", pages(&SLIDES, 960.0, 540.0))
                        .zoom(now_slide_zoom, move |next, _, cx| set(&slide_zoom, next, cx)),
                ),
            ),
        )
        .child(
            section(
                "EpubReader",
                "A book a chapter at a time, in a column sized for reading: how far through the chapter, the chapters listed, the type a step larger or smaller, and the next chapter a press away.",
                cx,
            )
            .child(div().w(px(760.)).h(px(420.)).child(EpubReader::new("read-book", "On Lift", chapters))),
        )
        .child(
            section(
                "Footnote / Glossary / TooltipTerm",
                "A note's number in running text, the note a rest of the pointer away; a term underlined in dashes that explains itself; and every term in order under its letter.",
                cx,
            )
            .child(
                div()
                    .w(px(560.))
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_baseline()
                            .child("A ")
                            .child(TooltipTerm::new("read-lift", "lift", "How far a color is blended toward white, from none to all of it."))
                            .child(" blends in gamma space")
                            .child(Footnote::new("read-note", 1, "Where a small step moves the eye as much as a large one moves the meter."))
                            .child(", so each step reads as even."),
                    )
                    .child(Glossary::new([
                        ("Lift", "How far a color is blended toward white."),
                        ("Accent", "A base color and a lift: the hue and how loud it is."),
                        ("Token", "A value every component reads from the theme."),
                        ("Leading", "The height of a line of text."),
                    ])),
            ),
        )
        .child(
            section(
                "FocusMode / ZenMode",
                "Zen mode folds the tools away and sets the text in a quiet column; Escape brings them back.",
                cx,
            )
            .child(probe(
                "read-zen",
                div()
                    .w(px(760.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .rounded(theme.radius(Radius::Lg))
                    .border_1()
                    .border_color(theme.colors.border)
                    .overflow_hidden()
                    .child(
                        ZenMode::new("read-zen", now_zen)
                            .on_exit({
                                let zen = zen.clone();
                                move |_, cx| set(&zen, false, cx)
                            })
                            .chrome(
                                div().p_2().flex().child(
                                    ely_gpui_component::buttons::Button::new("read-zen-on", "Focus on the text")
                                        .on_click(move |_, _, cx| set(&zen, true, cx)),
                                ),
                            )
                            .child(div().p_2().child(RichTextEditor::new("read-draft", &draft))),
                    ),
            )),
        )
        .child(
            section(
                "PrintPreview",
                "The pages the app laid out for the settings, small on a desk, with paper, turn, margins and copies beside; each change asks the app for new pages.",
                cx,
            )
            .child(
                div().w(px(840.)).h(px(440.)).child(PrintPreview::new(
                    "read-print",
                    pages(&PAGES, 612.0, 792.0),
                    now_print,
                    move |next, _, cx| set(&print, next, cx),
                    |settings, _, _| log::info!("gallery: print {settings:?}"),
                    |_, _| log::info!("gallery: print cancelled"),
                )),
            ),
        )
}
