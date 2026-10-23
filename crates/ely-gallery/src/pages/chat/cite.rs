use ely_gpui_component::chat::{
    CitationBadge, DocumentChunkPreview, SearchProgress, Source, SourceList, StepState,
    WebResultCard,
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::{probe::probe, ui::section};

fn sources() -> Vec<Source> {
    [
        (
            "example.com",
            "Color, in light and dark",
            "https://example.com/color/light-and-dark",
            "How one accent reads the same on paper and on charcoal.",
        ),
        (
            "example.org",
            "Gamma and the eye",
            "https://example.org/notes/gamma",
            "Why even steps in gamma space look even to the eye.",
        ),
        (
            "example.net",
            "Designing dark themes",
            "https://example.net/dark-themes",
            "Lift surfaces rather than shade them; depth reads as light.",
        ),
    ]
    .into_iter()
    .map(|(site, title, url, snippet)| Source {
        site: site.into(),
        title: title.into(),
        url: url.into(),
        snippet: Some(snippet.into()),
    })
    .collect()
}

pub fn citations(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let all = sources();
    let cite = |ix: usize| {
        CitationBadge::new(("chat-cite", ix), ix + 1, all[ix].clone())
            .on_open(move |_, _| log::info!("gallery: open source {ix}"))
    };
    section(
        "CitationBadge / InlineCitation / SourceCard / SourceList / SourcesPanel",
        "Where an answer's words came from: a numbered pill after the claim, its source a rest of the pointer away; the sources folded under the answer, each a card.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_1()
                    .child("A lift blends a color toward white, in gamma space.")
                    .child(probe("chat-cite-1", cite(1)))
                    .child("Dark themes lift further.")
                    .child(cite(0))
                    .child(cite(2)),
            )
            .child(probe(
                "chat-sources",
                div().w(px(560.)).child(
                    SourceList::new("chat-sources", all.clone())
                        .on_open(|ix, _, _| log::info!("gallery: open source {ix}")),
                ),
            )),
    )
}

const PASSAGE: &str = "Accents in dark carry more lift than in light, because a saturated color on charcoal looks louder than the same color on paper.";

pub fn search(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let all = sources();
    let louder = PASSAGE
        .find("looks louder")
        .expect("the passage holds its match");
    let more = PASSAGE
        .find("more lift")
        .expect("the passage holds its match");
    let lifted = more..more + "more lift".len();
    section(
        "SearchProgress / WebResultCard / DocumentChunkPreview",
        "A search as it runs, step by step; a result as the web gives it; and a passage found in a document, the words that matched washed.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_4()
            .child(SearchProgress::new(
                "chat-search",
                [
                    ("Searching for “lift in dark themes”", StepState::Done),
                    ("Reading 3 sources", StepState::Working),
                    ("Writing the answer", StepState::Waiting),
                ],
            ))
            .child(
                WebResultCard::new("chat-result", all[2].clone())
                    .date("Sep 2026")
                    .on_open(|_, _| log::info!("gallery: open result")),
            )
            .child(
                DocumentChunkPreview::new(
                    "lift-notes.pdf",
                    "Page 1 · Dark is not inverted",
                    PASSAGE,
                    vec![more..more + "more lift".len(), louder..louder + "looks louder".len()],
                )
                .score(0.86),
            )
            .child(
                div()
                    .w(px(280.))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(SourceList::new("chat-sources-narrow", all.clone()))
                    .child(
                        DocumentChunkPreview::new(
                            "a-long-name-for-the-lift-notes.pdf",
                            "Page 1 · Dark is not inverted",
                            PASSAGE,
                            vec![lifted],
                        )
                        .score(0.86),
                    ),
            ),
    )
}
