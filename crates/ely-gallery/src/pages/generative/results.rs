use std::time::Duration;

use ely_gpui_component::{
    data_display::BeforeAfter,
    generative::{
        ABCompareView, GenerationGrid, GenerationQueue, Job, JobState, Outcome, VariationPicker,
        Verdict,
    },
    primitives::Image,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, picture, section, set},
};

fn jobs() -> Vec<Job> {
    let job = |key: &str, prompt: &str, state, picture: Option<&str>| Job {
        key: key.to_string().into(),
        prompt: prompt.to_string().into(),
        state,
        picture: picture.map(|path| path.to_string().into()),
    };
    vec![
        job(
            "stair",
            "a spiral staircase in a white atrium",
            JobState::Done(Duration::from_secs(34)),
            Some(asset!("atrium-stair.jpg")),
        ),
        job(
            "dunes",
            "dunes at dusk, grass in the wind, a low sun",
            JobState::Running(0.62, Some(Duration::from_secs(12))),
            None,
        ),
        job(
            "olive",
            "an olive tree in a limestone courtyard",
            JobState::Queued(0),
            None,
        ),
        job(
            "hall",
            "a long hall of arches, noon light",
            JobState::Queued(1),
            None,
        ),
        job(
            "tide",
            "a tide pool at low water, pale rocks",
            JobState::Failed("out of memory on the GPU".into()),
            None,
        ),
    ]
}

pub fn queue(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let list = keep("gen-jobs", jobs, window, cx);
    let now = list.read(cx).clone();
    let (canceled, retried, removed) = (list.clone(), list.clone(), list);
    let drop = |jobs: &mut Vec<Job>, key: &SharedString| jobs.retain(|job| &job.key != key);
    section(
        "GenerationQueue",
        "Generations in line: each shows its picture or where it stands, its prompt, and how far it has come. Cancel while it waits or runs, Retry once it fails, Remove once it ends; a retried job folds back in at the end.",
        cx,
    )
    .child(probe(
        "gen-queue",
        div().w(px(480.)).child(
            GenerationQueue::new("gen-queue", now)
                .on_cancel(move |key, _, cx| {
                    let mut next = canceled.read(cx).clone();
                    drop(&mut next, key);
                    set(&canceled, next, cx)
                })
                .on_retry(move |key, _, cx| {
                    let mut next = retried.read(cx).clone();
                    let ahead = next.iter().filter(|job| matches!(job.state, JobState::Queued(_) | JobState::Running(..))).count();
                    let job = next.iter().find(|job| &job.key == key).cloned().expect("a listed job");
                    drop(&mut next, key);
                    next.push(Job { key: format!("{key}-again").into(), state: JobState::Queued(ahead), ..job });
                    set(&retried, next, cx)
                })
                .on_remove(move |key, _, cx| {
                    let mut next = removed.read(cx).clone();
                    drop(&mut next, key);
                    set(&removed, next, cx)
                }),
        ),
    ))
}

pub fn results(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tiles = keep(
        "gen-results",
        || {
            vec![
                Outcome::Done(asset!("atrium.jpg").into()),
                Outcome::Done(asset!("dunes.jpg").into()),
                Outcome::Pending(Some(0.4)),
                Outcome::Failed("Stopped by the content filter".into()),
            ]
        },
        window,
        cx,
    );
    let opened = keep("gen-opened", || None::<usize>, window, cx);
    let (now, seen) = (tiles.read(cx).clone(), *opened.read(cx));
    section(
        "GenerationGrid / ImageCompare",
        "A run's results in its shape: a breathing placeholder with its percent while one comes, the picture once done, or why it failed with Retry. Beside it, an upscale against its source; drag the divider or use Left and Right.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div().w(px(420.)).flex().flex_col().gap_2().child(probe(
                    "gen-grid",
                    div().w(px(420.)).child(
                        GenerationGrid::new("gen-grid", now, 1.5)
                            .on_open(move |ix, _, cx| set(&opened, Some(ix), cx))
                            .on_retry(move |ix, _, cx| {
                                let mut next = tiles.read(cx).clone();
                                next[ix] = Outcome::Pending(Some(0.05));
                                set(&tiles, next, cx)
                            }),
                    ),
                ))
                .children(seen.map(|ix| format!("Opened result {}", ix + 1))),
            )
            .child(
                BeforeAfter::new(
                    "gen-compare",
                    Image::new("gen-compare-before", picture(asset!("atrium-before.jpg"))).size_full(),
                    Image::new("gen-compare-after", picture(asset!("atrium.jpg"))).size_full(),
                )
                .w(px(420.))
                .h(px(280.)),
            ),
    )
}

pub fn variations(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chosen = keep("gen-variation", || 0_usize, window, cx);
    let varied = keep("gen-varied", || None::<SharedString>, window, cx);
    let (now, said) = (*chosen.read(cx), varied.read(cx).clone());
    let (strong, up) = (varied.clone(), varied);
    section(
        "VariationPicker / ABCompareView",
        "Variations of one picture, the chosen one large; vary it a little or a lot, or upscale it. Then two models' answers side by side, blind until you pick the better or call a tie.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div().w(px(420.)).flex().flex_col().gap_2().child(
                    VariationPicker::new(
                        "gen-variations",
                        [asset!("atrium.jpg"), asset!("atrium-olive.jpg"), asset!("atrium-stair.jpg"), asset!("atrium-before.jpg")],
                        1.5,
                        now,
                    )
                    .on_choose(move |ix, _, cx| set(&chosen, ix, cx))
                    .on_vary(move |strong_one, _, cx| {
                        let word = if strong_one { "Varying strongly" } else { "Varying subtly" };
                        set(&strong, Some(word.into()), cx)
                    })
                    .on_upscale(move |_, cx| set(&up, Some("Upscaling".into()), cx)),
                )
                .children(said),
            )
            .child(comparison(window, cx)),
    )
}

fn comparison(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let verdict = keep("gen-verdict", || None::<Verdict>, window, cx);
    let now = *verdict.read(cx);
    let compare = ABCompareView::new("gen-ab")
        .side("Orchid 3", "Morning light pools on a limestone stair as it turns up into a white atrium.")
        .side("Juniper 2", "A white spiral stair rises through a quiet atrium, each step catching the first light.")
        .blind()
        .on_verdict(move |next, _, cx| set(&verdict, Some(next), cx));
    let compare = match now {
        Some(now) => compare.verdict(now),
        None => compare,
    };
    probe("gen-ab", div().w(px(460.)).child(compare))
}
