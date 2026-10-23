use ely_gpui_component::{
    lists::{DirEntry, DirectoryListing, FileTree, GitStatus},
    theme::{ActiveTheme, Radius},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::{Timestamp, ToSpan};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const PROJECT: [&str; 12] = [
    "Cargo.toml",
    "README.md",
    "LICENSE",
    "src/main.rs",
    "src/app.rs",
    "src/ui/sidebar.rs",
    "src/ui/editor.rs",
    "src/ui/toolbar.rs",
    "src/sync/cloud.rs",
    "assets/logo.svg",
    "assets/fonts/Inter.ttf",
    "tests/editing.rs",
];

pub fn file_tree(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let opened = keep("file-opened", || None::<SharedString>, window, cx);
    let (now, store) = (opened.read(cx).clone(), opened.clone());
    let theme = cx.theme();
    section(
        "FileTree",
        "A project's files: folders first, an icon for each kind, git status by letter and tint, and a dot on folders with changes. Type to filter; matches keep their folders, which open.",
        cx,
    )
    .child(probe(
        "file-tree",
        div()
            .w(px(320.))
            .h(px(360.))
            .p_2()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .child(
                FileTree::new("files", PROJECT)
                    .status("src/ui/sidebar.rs", GitStatus::Modified)
                    .status("src/sync/cloud.rs", GitStatus::Added)
                    .status("tests/editing.rs", GitStatus::Untracked)
                    .status("LICENSE", GitStatus::Deleted)
                    .on_open(move |path, _, cx| set(&store, Some(path.clone()), cx))
                    .size_full(),
            ),
    ))
    .child(Caption::new(match now {
        Some(path) => format!("Opened {path}"),
        None => "Double press a file, or press Enter, to open it.".to_string(),
    }))
}

/// The demo disk: each folder's path and what it holds.
fn listing(path: &[&str], now: Timestamp) -> Vec<DirEntry> {
    let ago = |hours: i64| now.checked_sub(hours.hours()).expect("a time in range");
    match path.last().copied() {
        Some("Projects") => vec![
            DirEntry::folder("Ely", ago(2)),
            DirEntry::file("Roadmap.key", 18_400_000, ago(30)),
            DirEntry::file("Sketches.fig", 6_200_000, ago(90)),
        ],
        _ => vec![
            DirEntry::folder("Projects", ago(3)),
            DirEntry::folder("Receipts", ago(240)),
            DirEntry::file("Budget 2026.numbers", 2_310_000, ago(26)),
            DirEntry::file("Letter.pages", 410_000, ago(5)),
            DirEntry::file("Scan 12.pdf", 1_120_000, ago(120)),
            DirEntry::file("notes.txt", 3_200, ago(1)),
        ],
    }
}

pub fn directory(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let opened = keep("dir-opened", Timestamp::now, window, cx);
    let at = keep(
        "dir-path",
        || vec!["Macintosh HD", "Users", "grace", "Documents"],
        window,
        cx,
    );
    let (path, into, back) = (at.read(cx).clone(), at.clone(), at.clone());
    let entries = listing(&path, *opened.read(cx));
    section(
        "DirectoryListing",
        "A folder's contents in columns. A header sorts by its column and a second press reverses it; folders stay first. Open a folder with a double press or Enter; the path above climbs back.",
        cx,
    )
    .child(
        div().w(px(560.)).child(
            DirectoryListing::new("directory", path.clone(), entries)
                .on_open(move |entry, _, cx| {
                    if entry.is_folder() && entry.name().as_ref() == "Projects" {
                        let mut next = into.read(cx).clone();
                        next.push("Projects");
                        set(&into, next, cx);
                    }
                })
                .on_climb(move |level, _, cx| {
                    let mut next = back.read(cx).clone();
                    next.truncate(level + 1);
                    set(&back, next, cx);
                }),
        ),
    )
}
