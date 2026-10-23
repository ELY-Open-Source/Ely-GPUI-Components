use ely_gpui_component::{
    files::{FileGrid, FileIcon, FileItem},
    forms::{DropZone, InlineEdit},
    lists::{DirEntry, SelectableList},
    menus::{ContextMenu, Menu, MenuItem},
    primitives::IconName,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use jiff::Timestamp;

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

pub fn context_menu(cx: &mut App) -> impl IntoElement + use<> {
    let item = |label: &'static str, icon: IconName| {
        MenuItem::new(label)
            .icon(icon)
            .on_click(move |_, _| log::info!("gallery: {label}"))
    };
    let menu = Menu::new()
        .item(item("Open", IconName::FolderOpen))
        .item(item("Rename", IconName::PenLine))
        .item(item("Duplicate", IconName::Copy))
        .separator()
        .item(item("Move to Trash", IconName::Trash2));
    section(
        "FileContextMenu",
        "What a right press on files offers, as menus::ContextMenu draws it: open, rename, duplicate, and the trash apart.",
        cx,
    )
    .child(probe(
        "files-context",
        div().w(px(640.)).child(
            ContextMenu::new("files-context", menu)
                .child(FileGrid::new("files-context-grid", super::entries().into_iter().take(6))),
        ),
    ))
}

pub fn rename(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let name = keep(
        "files-rename",
        || SharedString::from("Brief.md"),
        window,
        cx,
    );
    let now = name.read(cx).clone();
    section(
        "RenameInline",
        "A name edited in place, as forms::InlineEdit does it: a press or Enter edits, Enter keeps and Escape drops; the icon follows the new name.",
        cx,
    )
    .child(probe(
        "files-rename",
        div()
            .w(px(320.))
            .flex()
            .items_center()
            .gap_2()
            .child(FileIcon::file(&now))
            .child(div().flex_1().min_w_0().child(
                InlineEdit::new("files-rename", now).on_commit(move |text, _, cx| {
                    change(&name, cx, |name| *name = text.clone())
                }),
            )),
    ))
}

pub fn drop_files(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let dropped = keep("files-dropped", Vec::<DirEntry>::new, window, cx);
    let now = dropped.read(cx).clone();
    let take = dropped.clone();
    let list = now
        .iter()
        .fold(SelectableList::new("files-dropped"), |list, entry| {
            let key = entry.name().clone();
            let id = SharedString::from(format!("files-dropped-{key}"));
            list.row(key, FileItem::new(id, entry.clone()).into())
        });
    section(
        "DragDropFiles",
        "Files dropped in, as forms::DropZone takes them; each lands in the list below.",
        cx,
    )
    .child(probe(
        "files-drop",
        div()
            .w(px(420.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                DropZone::new("files-drop")
                    .multiple()
                    .hint("Any kind of file")
                    .on_drop(move |paths, _, cx| {
                        let entries = paths.iter().map(|path| {
                            let bytes = path.metadata().expect("a dropped file reads").len();
                            let name = path.file_name().expect("a file name");
                            DirEntry::file(
                                name.to_string_lossy().to_string(),
                                bytes,
                                Timestamp::now(),
                            )
                        });
                        let entries: Vec<DirEntry> = entries.collect();
                        change(&take, cx, |dropped| {
                            for entry in entries {
                                if !dropped.iter().any(|kept| kept.name() == entry.name()) {
                                    dropped.push(entry);
                                }
                            }
                        })
                    }),
            )
            .child(list),
    ))
}
