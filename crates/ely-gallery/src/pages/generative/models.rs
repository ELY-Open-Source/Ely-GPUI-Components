use ely_gpui_component::generative::{
    Download, DownloadState, FineTuneJobCard, HardwareMonitor, ModelCard, ModelDownloadManager,
    ModelState, ModelStatus, Reading, TunePhase,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

pub fn cards(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let using = keep(
        "gen-model-used",
        || SharedString::from("orchid"),
        window,
        cx,
    );
    let now = using.read(cx).clone();
    let card = |key: &'static str, name: &str, maker: &str, about: &str, size: &str| {
        let pick = using.clone();
        ModelCard::new(
            SharedString::from(format!("gen-model-{key}")),
            name.to_string(),
            maker.to_string(),
            about.to_string(),
        )
        .fact("Parameters", size.to_string())
        .fact("Context", "128K tokens")
        .fact("License", "Apache-2.0")
        .in_use(now == key)
        .on_use(move |_, cx| set(&pick, SharedString::from(key), cx))
    };
    section(
        "ModelCard",
        "A model to choose: its maker, what it does, facts such as its size and license, and what it can do. The one in use says so; the others offer Use.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_4()
            .child(div().w(px(340.)).child(card("orchid", "Orchid 8B", "Orchid Labs", "Writes and reasons in plain words; runs on one card.", "8B").tag("Tools").tag("Vision")))
            .child(div().w(px(340.)).child(card("juniper", "Juniper 70B", "Juniper Co", "Long, careful answers for research and code.", "70B").tag("Tools"))),
    )
}

fn downloads() -> Vec<Download> {
    let download = |key: &str, name: &str, bytes: u64, state| Download {
        key: key.to_string().into(),
        name: name.to_string().into(),
        bytes,
        state,
    };
    vec![
        download(
            "orchid",
            "Orchid 8B · Q4",
            4_900_000_000,
            DownloadState::Fetching(0.43, 38_000_000),
        ),
        download(
            "juniper",
            "Juniper 70B · Q4",
            39_000_000_000,
            DownloadState::Paused(0.12),
        ),
        download(
            "voice",
            "Iris voice pack",
            310_000_000,
            DownloadState::Queued,
        ),
        download(
            "embed",
            "Lichen embeddings",
            640_000_000,
            DownloadState::Failed("the disk is full".into()),
        ),
        download("tiny", "Orchid 1B · Q8", 1_300_000_000, DownloadState::Done),
    ]
}

pub fn downloads_section(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let list = keep("gen-downloads", downloads, window, cx);
    let now = list.read(cx).clone();
    let change = |list: &gpui::Entity<Vec<Download>>| {
        let list = list.clone();
        move |key: &SharedString, state: Option<DownloadState>, cx: &mut App| {
            let mut next = list.read(cx).clone();
            match state {
                Some(state) => {
                    let download = next
                        .iter_mut()
                        .find(|download| &download.key == key)
                        .expect("a listed download");
                    download.state = state;
                }
                None => next.retain(|download| &download.key != key),
            }
            set(&list, next, cx)
        }
    };
    let (pause, resume, cancel, retry, delete) = (
        change(&list),
        change(&list),
        change(&list),
        change(&list),
        change(&list),
    );
    let (at_pause, at_resume) = (list.clone(), list.clone());
    section(
        "ModelDownloadManager",
        "Models on their way to this machine: how much of how much, how fast and how long is left, with Pause and Resume, Cancel, Retry, and Delete once a model is here.",
        cx,
    )
    .child(probe(
        "gen-downloads",
        div().w(px(480.)).child(
            ModelDownloadManager::new("gen-downloads", now)
                .on_pause(move |key, _, cx| {
                    let at = held(&at_pause, key, cx);
                    pause(key, Some(DownloadState::Paused(at)), cx)
                })
                .on_resume(move |key, _, cx| {
                    let at = held(&at_resume, key, cx);
                    resume(key, Some(DownloadState::Fetching(at, 38_000_000)), cx)
                })
                .on_cancel(move |key, _, cx| cancel(key, None, cx))
                .on_retry(move |key, _, cx| retry(key, Some(DownloadState::Queued), cx))
                .on_delete(move |key, _, cx| delete(key, None, cx)),
        ),
    ))
}

/// The share a download has fetched, or none yet.
fn held(list: &gpui::Entity<Vec<Download>>, key: &SharedString, cx: &App) -> f32 {
    match list
        .read(cx)
        .iter()
        .find(|download| &download.key == key)
        .map(|download| &download.state)
    {
        Some(DownloadState::Fetching(share, _) | DownloadState::Paused(share)) => *share,
        _ => 0.0,
    }
}

pub fn machine(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let history = |values: &[f32]| values.to_vec();
    section(
        "ModelStatus / HardwareMonitor",
        "Models on this machine, loaded, loading or not, with the GPU memory each holds; and the machine at work, each part's meter turning amber then red as it fills, beside its recent history.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(
                div()
                    .w(px(320.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(ModelStatus::new("gen-status-orchid", "Orchid 8B", ModelState::Ready).memory(7_800_000_000, 24_000_000_000))
                    .child(ModelStatus::new("gen-status-juniper", "Juniper 70B", ModelState::Loading(0.62)))
                    .child(ModelStatus::new("gen-status-iris", "Iris voice", ModelState::Idle)),
            )
            .child(
                div().w(px(360.)).child(HardwareMonitor::new(
                    "gen-machine",
                    [
                        Reading { name: "GPU".into(), share: 0.72, detail: "72%".into(), history: history(&[0.2, 0.35, 0.6, 0.55, 0.7, 0.8, 0.72]) },
                        Reading { name: "GPU memory".into(), share: 0.93, detail: "22.3 GB of 24 GB".into(), history: history(&[0.5, 0.6, 0.8, 0.9, 0.92, 0.93, 0.93]) },
                        Reading { name: "CPU".into(), share: 0.31, detail: "31%".into(), history: history(&[0.4, 0.3, 0.25, 0.35, 0.3, 0.28, 0.31]) },
                    ],
                )),
            ),
    )
}

pub fn tuning(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let canceled = keep("gen-tune-canceled", || false, window, cx);
    let stopped = *canceled.read(cx);
    let phase = if stopped {
        TunePhase::Canceled
    } else {
        TunePhase::Running(2, 3, 0.64)
    };
    section(
        "FineTuneJobCard",
        "A fine-tuning job: the base model and data it learns from, where it stands, how far it has come by epoch, and its loss on training and held-out data.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_4()
            .child(
                div().w(px(380.)).child(
                    FineTuneJobCard::new("gen-tune", "Plain-words tone", "Orchid 8B", "notes-2026.jsonl · 1,240 examples", phase)
                        .loss(vec![1.9, 1.4, 1.1, 0.92, 0.81, 0.74], vec![2.0, 1.5, 1.2, 1.02, 0.95, 0.91])
                        .on_cancel(move |_, cx| set(&canceled, true, cx)),
                ),
            )
            .child(
                div().w(px(380.)).child(
                    FineTuneJobCard::new("gen-tune-done", "Support replies", "Juniper 70B", "tickets.jsonl · 8,900 examples", TunePhase::Done)
                        .loss(vec![1.6, 1.1, 0.8, 0.62, 0.55, 0.51], vec![1.7, 1.2, 0.9, 0.74, 0.69, 0.68])
                        .on_open(|_, _| log::info!("gallery: open the tuned model")),
                ),
            ),
    )
}
