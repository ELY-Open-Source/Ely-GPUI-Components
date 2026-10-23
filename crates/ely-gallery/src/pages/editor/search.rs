use ely_gpui_component::{
    editor::{
        CodeEditor, FileHits, FindOptions, FindWidget, Hit, SearchFilters, SearchPanel, find_all,
        passes,
    },
    forms::TextInput,
    theme::ActiveTheme,
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::code::{SAMPLE, framed};
use crate::ui::{keep, section, set};

fn field(
    key: &'static str,
    text: &str,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Entity<TextInput> {
    let seed = text.to_string();
    window.use_keyed_state(key, cx, move |window, cx| {
        let mut field = TextInput::new(window, cx);
        field.set_text(seed, cx);
        field
    })
}

pub fn find(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let editor = window.use_keyed_state("search-editor", cx, |window, cx| {
        CodeEditor::new(SAMPLE, window, cx).language("Rust")
    });
    let query = field("search-find", "lift", window, cx);
    let replacement = field("search-replace", "blend", window, cx);
    let options = keep("search-options", FindOptions::default, window, cx);
    let current = keep("search-current", || 0usize, window, cx);
    let replacing = keep("search-replacing", || true, window, cx);
    let (now, at, showing) = (*options.read(cx), *current.read(cx), *replacing.read(cx));
    let text = editor.read(cx).text().to_string();
    let found = find_all(&text, query.read(cx).text(), now);
    let colors = cx.theme().colors.clone();
    let (widget, current_range) = match &found {
        Ok(ranges) => {
            let at = if ranges.is_empty() {
                None
            } else {
                Some(at.min(ranges.len() - 1))
            };
            let marks: Vec<_> = ranges
                .iter()
                .enumerate()
                .map(|(ix, range)| {
                    let wash = if Some(ix) == at {
                        colors.warning.opacity(0.45)
                    } else {
                        colors.warning.opacity(0.2)
                    };
                    (range.clone(), wash)
                })
                .collect();
            editor.update(cx, |editor, cx| editor.set_backgrounds(marks, cx));
            (
                FindWidget::new("search-widget", &query, at, ranges.len()),
                at.map(|ix| ranges[ix].clone()),
            )
        }
        Err(message) => {
            editor.update(cx, |editor, cx| editor.set_backgrounds(Vec::new(), cx));
            (
                FindWidget::new("search-widget", &query, None, 0)
                    .error(message.lines().last().unwrap_or("bad pattern").to_string()),
                None,
            )
        }
    };
    if let Some(range) = current_range.clone() {
        editor.update(cx, |editor, cx| editor.select([range], cx));
    }
    let total = found.as_ref().map_or(0, Vec::len);
    let (set_options, step, toggle) = (options.clone(), current.clone(), replacing.clone());
    let (swap, swap_editor, swap_query, swap_with) = (
        current.clone(),
        editor.clone(),
        query.clone(),
        replacement.clone(),
    );
    let widget = widget
        .options(now)
        .on_options(move |next, _, cx| set(&set_options, next, cx))
        .on_step(move |forward, _, cx| {
            let at = *step.read(cx);
            let next = if total == 0 {
                0
            } else if forward {
                (at + 1) % total
            } else {
                (at + total - 1) % total
            };
            set(&step, next, cx)
        })
        .on_toggle_replace(move |show, _, cx| set(&toggle, show, cx))
        .on_replace(move |all, _, cx| {
            let text = swap_editor.read(cx).text().to_string();
            let Ok(ranges) = find_all(&text, swap_query.read(cx).text(), now) else {
                return;
            };
            let with = swap_with.read(cx).text().to_string();
            let chosen: Vec<_> = if all {
                ranges
                    .into_iter()
                    .map(|range| (range, with.clone()))
                    .collect()
            } else {
                let at = (*swap.read(cx)).min(ranges.len().saturating_sub(1));
                ranges
                    .into_iter()
                    .nth(at)
                    .map(|range| (range, with.clone()))
                    .into_iter()
                    .collect()
            };
            swap_editor.update(cx, |editor, cx| editor.edit(chosen, cx));
        });
    let widget = if showing {
        widget.replace(&replacement)
    } else {
        widget
    };
    section(
        "FindWidget / ReplaceWidget",
        "Find over code by case, whole word or pattern; the current match glows among the rest, and the arrows step between them. The replace row swaps one or every match.",
        cx,
    )
    .child(
        framed(840.0, 320.0, cx)
            .relative()
            .child(editor)
            .child(div().absolute().top_1().right_4().child(widget)),
    )
}

fn results() -> Vec<FileHits> {
    let hit = |line: usize, text: &str, needle: &str| {
        let start = text.find(needle).expect("the needle is in the line");
        let found = start..start + needle.len();
        Hit {
            line,
            text: text.to_string().into(),
            ranges: vec![found],
        }
    };
    vec![
        FileHits {
            path: "src/palette.rs".into(),
            hits: vec![
                hit(
                    27,
                    "    /// The color for `name`, blended toward white by `lift`.",
                    "lift",
                ),
                hit(
                    28,
                    "    pub fn lookup(&self, name: &str, lift: f32) -> Color {",
                    "lift",
                ),
                hit(30, "        match lift {", "lift"),
            ],
        },
        FileHits {
            path: "src/theme/tokens.rs".into(),
            hits: vec![hit(
                112,
                "    pub fn surface_lift(&self, level: u8) -> f32 {",
                "lift",
            )],
        },
        FileHits {
            path: "tests/palette.rs".into(),
            hits: vec![hit(
                8,
                "    assert_eq!(palette.lookup(\"accent\", 1.0), WHITE, \"a full lift is white\");",
                "lift",
            )],
        },
    ]
}

pub fn panel(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let include = field("search-include", "src/**", window, cx);
    let exclude = field("search-exclude", "", window, cx);
    let ignore = keep("search-ignore", || true, window, cx);
    let folded = keep("search-folded", Vec::<SharedString>::new, window, cx);
    let opened = keep("search-opened", || None::<SharedString>, window, cx);
    let (now_ignore, now_folded, said) = (
        *ignore.read(cx),
        folded.read(cx).clone(),
        opened.read(cx).clone(),
    );
    let (include_text, exclude_text) = (
        include.read(cx).text().to_string(),
        exclude.read(cx).text().to_string(),
    );
    let shown: Vec<FileHits> = results()
        .into_iter()
        .filter(|file| passes(&file.path, &include_text, &exclude_text))
        .collect();
    let (flip, fold, open) = (ignore.clone(), folded.clone(), opened.clone());
    section(
        "SearchPanel / SearchResultItem / SearchFilters",
        "Matches across files, each file folding away, each line trimmed near its match; globs choose which files count.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(
                div().w(px(280.)).child(
                    SearchFilters::new("search-filters", &include, &exclude)
                        .ignore(now_ignore)
                        .on_ignore(move |on, _, cx| set(&flip, on, cx)),
                ),
            )
            .child(
                div().flex_1().child(
                    SearchPanel::new("search-panel", shown)
                        .folded(now_folded)
                        .on_fold(move |path, _, cx| {
                            let mut now = fold.read(cx).clone();
                            match now.iter().position(|folded| folded == path) {
                                Some(ix) => drop(now.remove(ix)),
                                None => now.push(path.clone()),
                            }
                            set(&fold, now, cx)
                        })
                        .on_open(move |path, line, _, cx| set(&open, Some(format!("Opened {path}:{}.", line + 1).into()), cx)),
                ),
            ),
    )
    .children(said.map(Caption::new))
}
