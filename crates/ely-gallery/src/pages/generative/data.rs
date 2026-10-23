use std::time::Duration;

use ely_gpui_component::{
    data_display::{DescriptionList, Tone},
    forms::TextInput,
    generative::{
        Embedded, EmbeddingVisualizer, PromptPlayground, PromptVersion, PromptVersionHistory,
    },
    tables::{Cell, Column, DataTable, Row},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px, rems};
use jiff::{SignedDuration, Timestamp};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const SAMPLES: [(&str, &str, bool, f64); 12] = [
    (
        "Summarize the meeting notes in two lines.",
        "The launch moves to May. Design owns the new onboarding flow.",
        true,
        38.0,
    ),
    (
        "Turn this list into a friendly reminder.",
        "A quick note: the lab closes at six on Friday. Please take your samples home.",
        true,
        41.0,
    ),
    (
        "Translate 'good morning' into Spanish.",
        "Buenos días.",
        true,
        12.0,
    ),
    (
        "Write a subject line for a refund email.",
        "Your refund is on its way",
        false,
        17.0,
    ),
    (
        "Explain a mutex to a new engineer.",
        "A mutex lets one thread touch shared data at a time. The others wait their turn.",
        true,
        36.0,
    ),
    ("Suggest a name for a hiking app.", "Trailhead", true, 11.0),
    (
        "Rewrite without jargon: 'We will leverage synergies.'",
        "We will work together.",
        false,
        24.0,
    ),
    (
        "List three checks before a deploy.",
        "Tests pass. Migrations run. Someone is on call.",
        true,
        29.0,
    ),
    ("Answer in one word: is 91 prime?", "No.", false, 13.0),
    (
        "Draft a two-line product update.",
        "Search is twice as fast. Filters now remember your last choice.",
        true,
        33.0,
    ),
    (
        "Give a calm reply to an angry customer.",
        "I'm sorry for the trouble. I've refunded the order and will check in tomorrow.",
        true,
        40.0,
    ),
    ("Name the capital of Canada.", "Ottawa.", false, 10.0),
];

pub fn dataset(cx: &mut App) -> impl IntoElement + use<> {
    let rows: Vec<Row> = SAMPLES
        .iter()
        .enumerate()
        .map(|(ix, (prompt, answer, train, tokens))| {
            let split = if *train {
                ("Train", Tone::Neutral)
            } else {
                ("Held out", Tone::Info)
            };
            Row::new(
                SharedString::from(format!("sample-{ix}")),
                [
                    Cell::Text((*prompt).into()),
                    Cell::Text((*answer).into()),
                    Cell::Tag(split.0.into(), split.1),
                    Cell::Number(*tokens),
                ],
            )
        })
        .collect();
    section(
        "DatasetViewer → tables::DataTable",
        "Samples of a dataset: a prompt, its answer, its split and its length. Sort by any column, page through, and open a sample to read it whole.",
        cx,
    )
    .child(probe(
        "gen-dataset",
        div().w(px(680.)).child(
            DataTable::new(
                "gen-dataset",
                [
                    Column::new("prompt", "Prompt"),
                    Column::new("answer", "Answer"),
                    Column::new("split", "Split").width(rems(6.)),
                    Column::new("tokens", "Tokens").width(rems(5.)).end().decimals(0),
                ],
            )
            .rows(rows)
            .paged(5)
            .detail(|key, _, _| {
                let ix: usize = key.trim_start_matches("sample-").parse().expect("a sample's key");
                let (prompt, answer, _, _) = SAMPLES[ix];
                DescriptionList::new().stacked().item("Prompt", prompt).item("Answer", answer).into_any_element()
            }),
        ),
    ))
}

/// A number from 0 to 1 that looks random and stays the same for its seed.
fn noise(seed: u32) -> f32 {
    let mut x = (seed + 1).wrapping_mul(0x9E37_79B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
}

/// Notes in four topics, gathered near each topic's middle.
fn notes() -> Vec<Embedded> {
    let middles = [
        [-0.5, 0.4, 0.2],
        [0.45, 0.35, -0.3],
        [0.1, -0.5, 0.4],
        [-0.3, -0.2, -0.5],
    ];
    (0..64)
        .map(|ix| {
            let group = ix % 4;
            let seed = (ix * 6) as u32;
            let jitter =
                |axis: u32| (noise(seed + axis * 2) + noise(seed + axis * 2 + 1) - 1.0) * 0.3;
            Embedded {
                key: format!("note-{ix}").into(),
                label: format!("Note {}", ix + 1).into(),
                group,
                at: [0, 1, 2]
                    .map(|axis| (middles[group][axis as usize] + jitter(axis)).clamp(-1.0, 1.0)),
            }
        })
        .collect()
}

pub fn embeddings(cx: &mut App) -> impl IntoElement + use<> {
    let topics = ["Recipes", "Travel", "Finance", "Code"];
    section(
        "EmbeddingVisualizer",
        "Notes placed by what they mean, colored by topic, flat or turned in three dimensions. Drag the turned one, or use the arrow keys. The pointer names a note; the legend hides a topic.",
        cx,
    )
    .child(probe(
        "gen-embed",
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(div().w(px(340.)).child(EmbeddingVisualizer::new("gen-embed-flat", notes(), topics)))
            .child(div().w(px(340.)).child(EmbeddingVisualizer::new("gen-embed-turned", notes(), topics).three_dimensions(true))),
    ))
}

/// What the playground shows: running, and the last answer with its tokens and time.
#[derive(Clone, Default)]
struct Play {
    running: bool,
    answer: Option<(SharedString, usize, Duration)>,
}

pub fn playground(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let template = window.use_keyed_state("gen-play-template", cx, |window, cx| {
        let mut input = TextInput::new(window, cx).multi_line(2, 5);
        input.set_text("Write three short notes on {topic} in a {tone} voice.", cx);
        input
    });
    let play = keep("gen-play", Play::default, window, cx);
    let now = play.read(cx).clone();
    let playground = PromptPlayground::new("gen-play", &template)
        .running(now.running)
        .on_run(move |_, _, cx| {
            set(
                &play,
                Play {
                    running: true,
                    answer: None,
                },
                cx,
            );
            let play = play.clone();
            cx.spawn(async move |cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(900))
                    .await;
                cx.update(|cx| {
                    let answer =
                        "Start with what you saw. Keep each line short. End on one plain fact.";
                    set(
                        &play,
                        Play {
                            running: false,
                            answer: Some((answer.into(), 18, Duration::from_millis(1240))),
                        },
                        cx,
                    )
                });
            })
            .detach();
        });
    let playground = match now.answer {
        Some((text, tokens, took)) => playground.output(text, tokens, took),
        None => playground,
    };
    section(
        "PromptPlayground",
        "A prompt to try. Each {name} in the template gets a field, and the prompt below marks what is still blank. Run waits for every value, then shows the answer with its tokens and time.",
        cx,
    )
    .child(probe("gen-play", div().w(px(520.)).child(playground)))
}

fn history() -> Vec<PromptVersion> {
    let now = Timestamp::now();
    let version = |key: &str, text: &str, note: &str, hours: i64| PromptVersion {
        key: key.to_string().into(),
        text: text.to_string().into(),
        author: if hours > 30 { "Mira" } else { "Theo" }.into(),
        at: now - SignedDuration::from_hours(hours),
        note: note.to_string().into(),
    };
    vec![
        version(
            "v4",
            "You answer support questions. Write plain, short replies. Say sorry once. Offer the next step.",
            "Offer the next step",
            3,
        ),
        version(
            "v3",
            "You answer support questions. Write plain, short replies. Say sorry once.",
            "Sorry once, not every line",
            26,
        ),
        version(
            "v2",
            "You answer support questions. Write plain replies. Always say sorry.",
            "Plain words",
            50,
        ),
        version(
            "v1",
            "You are a helpful assistant. Always say sorry.",
            "First draft",
            98,
        ),
    ]
}

pub fn versions(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let kept = keep("gen-versions", || (history(), 1usize), window, cx);
    let (list, chosen) = kept.read(cx).clone();
    let (choose, restore) = (kept.clone(), kept.clone());
    section(
        "PromptDiff / PromptVersionHistory",
        "A prompt's versions, newest first, and what the chosen one changed: words taken out struck through, words put in washed green. Restore brings an older one back as the newest.",
        cx,
    )
    .child(probe(
        "gen-versions",
        div().w(px(640.)).child(
            PromptVersionHistory::new("gen-versions", list, chosen)
                .on_choose(move |ix, _, cx| {
                    let list = choose.read(cx).0.clone();
                    set(&choose, (list, ix), cx)
                })
                .on_restore(move |ix, _, cx| {
                    let mut list = restore.read(cx).0.clone();
                    let back = list[ix].clone();
                    let place = list.len() - ix;
                    list.insert(
                        0,
                        PromptVersion {
                            key: format!("v{}", list.len() + 1).into(),
                            at: Timestamp::now(),
                            note: format!("Restored version {place}").into(),
                            ..back
                        },
                    );
                    set(&restore, (list, 0), cx)
                }),
        ),
    ))
}
