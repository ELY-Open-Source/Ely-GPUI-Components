use ely_gpui_component::{
    data_display::UsageBar,
    files::{
        DownloadManager, FileOperation, FileOperationProgress, Transfer, TransferQueue,
        TransferState,
    },
    typography::format,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

fn transfer(
    key: &str,
    name: &str,
    upload: bool,
    (moved, total): (u64, u64),
    state: TransferState,
) -> Transfer {
    Transfer {
        key: SharedString::from(key.to_string()),
        name: SharedString::from(name.to_string()),
        upload,
        moved,
        total,
        state,
    }
}

pub fn operations(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let copy = keep(
        "files-copy",
        || TransferState::Moving { rate: 38_000_000 },
        window,
        cx,
    );
    let now = copy.read(cx).clone();
    let (pause, resume, cancel) = (copy.clone(), copy.clone(), copy);
    let running = FileOperationProgress::new(
        "files-copy",
        FileOperation::Copy,
        12,
        "Atrium",
        (46_000_000, 184_000_000),
        now,
    )
    .current("Recordings/Walkthrough.mov")
    .on_pause(move |_, cx| change(&pause, cx, |state| *state = TransferState::Paused))
    .on_resume(move |_, cx| {
        change(&resume, cx, |state| {
            *state = TransferState::Moving { rate: 38_000_000 }
        })
    })
    .on_cancel(move |_, cx| {
        change(&cancel, cx, |state| {
            *state = TransferState::Failed("Cancelled".into())
        })
    });
    let failed = FileOperationProgress::new(
        "files-move-failed",
        FileOperation::Move,
        3,
        "Backup",
        (820_000, 4_800_000),
        TransferState::Failed("The disk Backup is full.".into()),
    );
    let done = FileOperationProgress::new(
        "files-move-done",
        FileOperation::Move,
        1,
        "Documents",
        (88_000, 88_000),
        TransferState::Done,
    );
    section(
        "FileOperationProgress",
        "A copy or a move under way: what it does and where to, a bar, the bytes done with the time left, and the file it takes now. Pause, resume and cancel while it runs.",
        cx,
    )
    .child(probe(
        "files-operations",
        div()
            .w(px(480.))
            .flex()
            .flex_col()
            .gap_3()
            .child(running)
            .child(failed)
            .child(done),
    ))
}

pub fn queue(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let transfers = keep(
        "files-queue",
        || {
            vec![
                transfer(
                    "walkthrough",
                    "Walkthrough.mov",
                    true,
                    (61_000_000, 184_000_000),
                    TransferState::Moving { rate: 4_200_000 },
                ),
                transfer(
                    "budget",
                    "Budget.csv",
                    true,
                    (0, 18_300),
                    TransferState::Queued,
                ),
                transfer(
                    "moodboard",
                    "Moodboard.png",
                    false,
                    (1_100_000, 3_200_000),
                    TransferState::Paused,
                ),
                transfer(
                    "survey",
                    "Survey.csv",
                    false,
                    (9_000, 42_000),
                    TransferState::Failed("The server hung up.".into()),
                ),
                transfer(
                    "brief",
                    "Brief.md",
                    true,
                    (4_800, 4_800),
                    TransferState::Done,
                ),
            ]
        },
        window,
        cx,
    );
    let now = transfers.read(cx).clone();
    let set = |to: TransferState| {
        let transfers = transfers.clone();
        move |key: &SharedString, _: &mut Window, cx: &mut App| {
            let to = to.clone();
            change(&transfers, cx, |all| {
                for each in all.iter_mut().filter(|each| each.key == *key) {
                    each.state = to.clone();
                }
            })
        }
    };
    let drop = transfers.clone();
    section(
        "TransferQueue",
        "Uploads and downloads in one queue: which way each goes, its bar and status, and pause, resume, retry or cancel as its state allows.",
        cx,
    )
    .child(probe(
        "files-queue",
        div().w(px(480.)).child(
            TransferQueue::new("files-queue", now)
                .pause(set(TransferState::Paused))
                .resume(set(TransferState::Moving { rate: 4_200_000 }))
                .retry(set(TransferState::Moving { rate: 2_000_000 }))
                .cancel(move |key, _, cx| change(&drop, cx, |all| all.retain(|each| each.key != *key))),
        ),
    ))
}

pub fn downloads(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let downloads = keep(
        "files-downloads",
        || {
            vec![
                transfer(
                    "inter",
                    "Inter-4.1.zip",
                    false,
                    (2_600_000, 6_900_000),
                    TransferState::Moving { rate: 1_800_000 },
                ),
                transfer(
                    "rust",
                    "rust-1.95.pkg",
                    false,
                    (0, 312_000_000),
                    TransferState::Queued,
                ),
                transfer(
                    "atrium",
                    "Atrium.jpg",
                    false,
                    (2_140_000, 2_140_000),
                    TransferState::Done,
                ),
                transfer(
                    "notes",
                    "Release notes.pdf",
                    false,
                    (430_000, 430_000),
                    TransferState::Done,
                ),
            ]
        },
        window,
        cx,
    );
    let now = downloads.read(cx).clone();
    let (drop, clear) = (downloads.clone(), downloads);
    section(
        "DownloadManager",
        "Downloads as a browser keeps them: those on their way above, the finished below under Clear. A finished one opens with a press and shows its folder.",
        cx,
    )
    .child(probe(
        "files-downloads",
        div().w(px(480.)).child(
            DownloadManager::new("files-downloads", now)
                .open(|key, _, _| log::info!("gallery: open {key}"))
                .show(|key, _, _| log::info!("gallery: show {key} in its folder"))
                .cancel(move |key, _, cx| change(&drop, cx, |all| all.retain(|each| each.key != *key)))
                .on_clear(move |_, cx| {
                    change(&clear, cx, |all| all.retain(|each| each.state != TransferState::Done))
                }),
        ),
    ))
}

pub fn storage(cx: &mut App) -> impl IntoElement + use<> {
    const GB: f64 = 1_000_000_000.0;
    section(
        "StorageUsage",
        "Where a disk's room goes, as data_display::UsageBar shows it: a part a kind, and what is left free.",
        cx,
    )
    .child(probe(
        "files-storage",
        div().w(px(480.)).child(
            UsageBar::new(512.0 * GB)
                .part("Documents", 38.0 * GB)
                .part("Photos", 142.0 * GB)
                .part("Music", 61.0 * GB)
                .part("Apps", 96.0 * GB)
                .amounts(|bytes| format::file_size(bytes as u64, false)),
        ),
    ))
}
