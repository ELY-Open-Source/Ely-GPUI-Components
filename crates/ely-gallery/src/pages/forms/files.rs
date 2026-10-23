use std::path::PathBuf;

use ely_gpui_component::forms::{DropZone, FileInput};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set, specimen, specimens},
};

fn names(paths: &[PathBuf]) -> String {
    let names: Vec<String> = paths
        .iter()
        .map(|path| {
            path.file_name()
                .expect("a file has a name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.join(", ")
}

pub fn files(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let one = keep("file-one", Vec::<PathBuf>::new, window, cx);
    let many = keep(
        "file-many",
        || {
            ["Cargo.toml", "README.md", "AGENTS.md"]
                .map(PathBuf::from)
                .to_vec()
        },
        window,
        cx,
    );
    let taken = keep("drop-taken", Vec::<PathBuf>::new, window, cx);
    let (one_now, many_now, taken_now) = (
        one.read(cx).clone(),
        many.read(cx).clone(),
        taken.read(cx).clone(),
    );
    section(
        "FileInput / DropZone",
        "Click a field or Browse for the system's dialog, or drop files from Finder. A folder, or two files on a one-file target, turns it red.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "one file",
                probe(
                    "file-one",
                    div().w(px(260.0)).child(
                        FileInput::new("file-one", one_now)
                            .on_change(move |next, _, cx| set(&one, next, cx)),
                    ),
                ),
                cx,
            ))
            .child(specimen(
                "several",
                div().w(px(260.0)).child(
                    FileInput::new("file-many", many_now)
                        .multiple()
                        .on_change(move |next, _, cx| set(&many, next, cx)),
                ),
                cx,
            )),
    )
    .child(specimen(
        if taken_now.is_empty() {
            "nothing dropped yet".to_string()
        } else {
            format!("took {}", names(&taken_now))
        },
        probe(
            "drop-zone",
            div().w(px(420.0)).child(
                DropZone::new("drop-zone")
                    .multiple()
                    .hint("Any kind of file")
                    .on_drop(move |next, _, cx| set(&taken, next, cx)),
            ),
        ),
        cx,
    ))
}
