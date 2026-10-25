use ely_gpui_component::{
    debug::HexViewer,
    documents::{Favorite, Favorites},
    files::{
        ArchiveEntry, ArchiveViewer, DuplicateFinder, Duplicates, FileGrid, FilePreview,
        FileSearch, FoundFile, RecentFile, RecentFiles,
    },
    forms::TextInput,
    lists::DirEntry,
    overlays::Dialog,
};
use gpui::{
    App, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
};
use jiff::{SignedDuration, ToSpan};

use crate::{
    probe::probe,
    ui::{change, keep, picture, section},
};

const ATRIUM: &str = asset!("atrium.jpg");
const BRIEF: &str = "# Atrium\n\nA quiet hall for the new library.\n\n- Light from above, all day\n- One stair, curved, in plaster\n- Olive trees by the windows\n\nNext: the model walkthrough on Friday.";

fn entry(name: &str, bytes: u64, hours: i64) -> DirEntry {
    DirEntry::file(
        name.to_string(),
        bytes,
        ely_gpui_component::typography::format::now() - hours.hours(),
    )
}

pub fn previews(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let looking = keep("files-looking", || None::<SharedString>, window, cx);
    let picked = keep("files-look-picked", || None::<SharedString>, window, cx);
    let (open, pick) = (looking.clone(), picked.clone());
    let chosen = picked.read(cx).clone();
    let preview = |name: &SharedString| match name.as_ref() {
        "Atrium.jpg" => FilePreview::new("files-look", entry("Atrium.jpg", 2_140_000, 50))
            .picture(picture(ATRIUM), 1.5),
        "Brief.md" => FilePreview::new("files-look", entry("Brief.md", 4_800, 2)).text(BRIEF),
        other => FilePreview::new("files-look", entry(other, 48_000_000, 700)),
    };
    let tiles = [
        entry("Atrium.jpg", 2_140_000, 50),
        entry("Brief.md", 4_800, 2),
        entry("Archive.zip", 48_000_000, 700),
    ];
    let grid = FileGrid::new("files-look-grid", tiles)
        .on_select(move |name, _, cx| change(&pick, cx, |now| *now = Some(name.clone())));
    let grid = match chosen.clone() {
        Some(name) => grid.selected(name),
        None => grid,
    };
    let shown = looking.read(cx).clone();
    let close = looking.clone();
    section(
        "FilePreview / QuickLook",
        "A file shown large: its picture in its shape, its text's first lines, or its kind. Space on a picked tile shows it over the page, as a quick look does; Escape puts it away.",
        cx,
    )
    .child(probe(
        "files-previews",
        div()
            .w(px(640.))
            .flex()
            .gap_4()
            .child(div().flex_1().min_w_0().child(preview(&"Atrium.jpg".into())))
            .child(div().flex_1().min_w_0().child(preview(&"Brief.md".into())))
            .child(div().flex_1().min_w_0().child(preview(&"Archive.zip".into()))),
    ))
    .child(probe(
        "files-quick",
        div()
            .w(px(640.))
            .on_key_down(move |event, _, cx| {
                if event.keystroke.key == "space" {
                    let picked = chosen.clone();
                    change(&open, cx, |now| *now = picked);
                }
            })
            .child(grid),
    ))
    .children(shown.map(|name| {
        Dialog::new("files-quick-look", name.clone(), move |_, cx| {
            change(&close, cx, |now| *now = None)
        })
        .child(div().w(px(420.)).child(preview(&name)))
    }))
}

pub fn recent(cx: &mut App) -> impl IntoElement + use<> {
    let now = ely_gpui_component::typography::format::now();
    let file = |name: &str, folder: &str, minutes: i64| RecentFile {
        name: SharedString::from(name.to_string()),
        folder: SharedString::from(folder.to_string()),
        opened: now - SignedDuration::from_mins(minutes),
    };
    let files = [
        file("Brief.md", "Projects/Atrium", 12),
        file("Budget.csv", "Projects/Atrium", 95),
        file("Walkthrough.mov", "Projects/Atrium/Recordings", 60 * 26),
        file("Invoice.pdf", "Documents", 60 * 30),
        file("Survey.csv", "Projects/Dunes", 60 * 24 * 4),
    ];
    section(
        "RecentFiles",
        "Files opened lately, newest first, under a header for each day. Enter or a double press opens one.",
        cx,
    )
    .child(probe(
        "files-recent",
        div().w(px(420.)).child(
            RecentFiles::new("files-recent", files)
                .on_open(|file, _, _| log::info!("gallery: open {}", file.name)),
        ),
    ))
}

pub fn favorites(cx: &mut App) -> impl IntoElement + use<> {
    let pin = |key: &str, icon: &str, title: &str| Favorite {
        key: key.to_string().into(),
        icon: icon.to_string().into(),
        title: title.to_string().into(),
    };
    section(
        "Favorites / Bookmarks",
        "Places pinned for quick reach, as documents::Favorites keeps them: a press opens one, a drag reorders, the star lets one go.",
        cx,
    )
    .child(probe(
        "files-favorites",
        div().w(px(280.)).child(
            Favorites::new(
                "files-favorites",
                [
                    pin("atrium", "📐", "Atrium"),
                    pin("roadmap", "🗺️", "Roadmap.md"),
                    pin("music", "🎧", "Music"),
                ],
            )
            .on_open(|key, _, _| log::info!("gallery: open {key}")),
        ),
    ))
}

/// Every file of the demo whose name holds `text`, ignoring case.
fn matching(text: &str) -> Vec<FoundFile> {
    let text = text.trim().to_lowercase();
    if text.is_empty() {
        return Vec::new();
    }
    let mut found = Vec::new();
    let mut walk = vec![vec![SharedString::from("Home")]];
    while let Some(path) = walk.pop() {
        for entry in super::explorer::listing(&path) {
            let mut next = path.clone();
            next.push(entry.name().clone());
            if entry.is_folder() {
                walk.push(next);
            } else if entry.name().to_lowercase().contains(&text) {
                let folder: Vec<&str> = path.iter().map(|level| level.as_ref()).collect();
                found.push(FoundFile {
                    name: entry.name().clone(),
                    folder: folder.join("/").into(),
                });
            }
        }
    }
    found
}

pub fn search(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let query = window.use_keyed_state("files-search-query", cx, |window, cx| {
        TextInput::new(window, cx).label("Search files")
    });
    let found = matching(query.read(cx).text());
    section(
        "FileSearch",
        "A search over files: the host searches as the name is typed, and each file found shows its folder. Enter or a double press opens one.",
        cx,
    )
    .child(probe(
        "files-search",
        div().w(px(420.)).child(
            FileSearch::new("files-search", &query, found)
                .on_open(|file, _, _| log::info!("gallery: open {}", file.name)),
        ),
    ))
}

pub fn duplicates(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let group = |name: &str, size: u64, paths: &[&str]| Duplicates {
        name: name.to_string().into(),
        size,
        paths: paths
            .iter()
            .map(|path| SharedString::from(path.to_string()))
            .collect(),
    };
    let groups = vec![
        group(
            "Atrium.jpg",
            2_140_000,
            &[
                "Projects/Atrium/Atrium.jpg",
                "Downloads/Atrium.jpg",
                "Desktop/Atrium copy.jpg",
            ],
        ),
        group(
            "Walkthrough.mov",
            184_000_000,
            &[
                "Projects/Atrium/Recordings/Walkthrough.mov",
                "Downloads/Walkthrough.mov",
            ],
        ),
    ];
    let marked = keep(
        "files-duplicates",
        || {
            vec![
                SharedString::from("Downloads/Atrium.jpg"),
                "Desktop/Atrium copy.jpg".into(),
                "Downloads/Walkthrough.mov".into(),
            ]
        },
        window,
        cx,
    );
    let now = marked.read(cx).clone();
    let (mark, remove) = (marked.clone(), marked);
    section(
        "DuplicateFinder",
        "Files found more than once, each copy with a mark; the last copy a group keeps stays. What the marked copies free sits beside Remove.",
        cx,
    )
    .child(probe(
        "files-duplicates",
        div().w(px(560.)).child(
            DuplicateFinder::new("files-duplicates", groups)
                .marked(now)
                .on_mark(move |path, on, _, cx| {
                    change(&mark, cx, |marked| {
                        marked.retain(|kept| kept != path);
                        if on {
                            marked.push(path.clone());
                        }
                    })
                })
                .on_remove(move |paths, _, cx| {
                    log::info!("gallery: remove {} copies", paths.len());
                    change(&remove, cx, |marked| marked.clear())
                }),
        ),
    ))
}

pub fn archive(cx: &mut App) -> impl IntoElement + use<> {
    let entry = |path: &str, size: u64, packed: u64| ArchiveEntry {
        path: SharedString::from(path.to_string()),
        size,
        packed,
    };
    let entries = [
        entry("Atrium/Brief.md", 4_800, 1_900),
        entry("Atrium/Budget.csv", 18_300, 4_100),
        entry("Atrium/Design/Moodboard.png", 3_200_000, 3_050_000),
        entry("Atrium/Design/Palette.json", 2_100, 600),
        entry("Atrium/main.rs", 9_200, 2_700),
        entry("README.md", 1_400, 700),
    ];
    section(
        "ArchiveViewer",
        "What an archive holds, as a tree of its folders: each file with its size and how much packing saved.",
        cx,
    )
    .child(probe(
        "files-archive",
        div().w(px(420.)).h(px(300.)).child(
            ArchiveViewer::new("files-archive", "Atrium.zip", entries)
                .on_open(|entry, _, _| log::info!("gallery: open {}", entry.path)),
        ),
    ))
}

pub fn hex(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let bytes = keep(
        "files-hex",
        || include_bytes!("../../../assets/atrium.jpg")[..512].to_vec(),
        window,
        cx,
    );
    let bytes = bytes.read(cx).clone();
    section(
        "HexViewer",
        "A file's first bytes, as debug::HexViewer shows them: offsets, hex and text side by side.",
        cx,
    )
    .child(probe(
        "files-hex",
        div()
            .w(px(640.))
            .h(px(220.))
            .child(HexViewer::new("files-hex", bytes)),
    ))
}
