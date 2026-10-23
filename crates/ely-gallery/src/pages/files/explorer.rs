use ely_gpui_component::{
    files::{FileExplorer, FileView},
    lists::{DirEntry, FileTree},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, ToSpan};

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

/// What the demo's folder at `path` holds, the root first.
pub fn listing(path: &[SharedString]) -> Vec<DirEntry> {
    let now = Timestamp::now();
    let file = |name: &str, bytes: u64, hours: i64| {
        DirEntry::file(name.to_string(), bytes, now - hours.hours())
    };
    let folder = |name: &str, hours: i64| DirEntry::folder(name.to_string(), now - hours.hours());
    let joined: Vec<&str> = path.iter().map(|level| level.as_ref()).collect();
    match joined.join("/").as_str() {
        "Home" => vec![
            folder("Projects", 2),
            folder("Documents", 30),
            folder("Music", 400),
            file("Notes.txt", 1_200, 5),
        ],
        "Home/Projects" => vec![
            folder("Atrium", 1),
            folder("Dunes", 90),
            file("Roadmap.md", 6_400, 20),
        ],
        "Home/Projects/Atrium" => super::entries(),
        "Home/Projects/Atrium/Design" => vec![
            file("Moodboard.png", 3_200_000, 4),
            file("Palette.json", 2_100, 8),
            file("Display.ttf", 180_000, 300),
        ],
        "Home/Projects/Atrium/Recordings" => vec![
            file("Walkthrough.mov", 184_000_000, 120),
            file("Voiceover.m4a", 12_000_000, 60),
        ],
        "Home/Projects/Dunes" => vec![
            file("Survey.csv", 42_000, 100),
            file("Dunes.jpg", 2_800_000, 200),
        ],
        "Home/Documents" => vec![
            file("Invoice.pdf", 88_000, 40),
            file("Contract.docx", 64_000, 700),
        ],
        "Home/Music" => vec![
            file("Theme.mp3", 7_400_000, 800),
            file("Ambient.flac", 31_000_000, 900),
        ],
        other => panic!("the demo has no folder {other}"),
    }
}

/// Where the demo's explorer stands, how it shows it, and its pick.
#[derive(Clone)]
struct Browsing {
    path: Vec<SharedString>,
    view: FileView,
    selected: Option<SharedString>,
}

pub fn explorer(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep(
        "files-explorer",
        || Browsing {
            path: ["Home", "Projects", "Atrium"]
                .map(SharedString::from)
                .to_vec(),
            view: FileView::Columns,
            selected: None,
        },
        window,
        cx,
    );
    let now = state.read(cx).clone();
    let levels = (1..=now.path.len()).map(|depth| listing(&now.path[..depth]));
    let (view, go, pick) = (state.clone(), state.clone(), state);
    let explorer = FileExplorer::new("files-explorer", now.path.clone(), levels, now.view)
        .on_view(move |to, _, cx| change(&view, cx, |it| it.view = to))
        .on_navigate(move |path, _, cx| {
            change(&go, cx, |it| {
                it.path = path.to_vec();
                it.selected = None;
            })
        })
        .on_select(move |name, _, cx| change(&pick, cx, |it| it.selected = Some(name.clone())))
        .on_open(|entry, _, _| log::info!("gallery: open {}", entry.name()));
    let explorer = match now.selected {
        Some(name) => explorer.selected(name),
        None => explorer,
    };
    section(
        "FileExplorer",
        "A folder three ways: a list that sorts by column, tiles, or columns that walk the path. Its name, its count and a switch sit above the path, which climbs; a folder opens in place.",
        cx,
    )
    .child(probe("files-explorer", div().w(px(640.)).child(explorer)))
}

/// Every file of the demo under `path`, by its path below the root.
fn files_under(path: Vec<SharedString>, out: &mut Vec<String>) {
    for entry in listing(&path) {
        let mut next = path.clone();
        next.push(entry.name().clone());
        if entry.is_folder() {
            files_under(next, out);
        } else {
            let below: Vec<&str> = next[1..].iter().map(|level| level.as_ref()).collect();
            out.push(below.join("/"));
        }
    }
}

pub fn tree(cx: &mut App) -> impl IntoElement + use<> {
    let mut paths = Vec::new();
    files_under(vec!["Home".into()], &mut paths);
    section(
        "FolderTree",
        "The same folders as lists::FileTree draws them: folders first, a filter above, Enter or a double press opens a file.",
        cx,
    )
    .child(probe(
        "files-tree",
        div().w(px(320.)).h(px(320.)).child(
            FileTree::new("files-tree", paths)
                .size_full()
                .on_open(|path, _, _| log::info!("gallery: open {path}")),
        ),
    ))
}
