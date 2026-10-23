use ely_gpui_component::{
    buttons::{Button, SegmentedControl},
    editor::{
        CodeEditor, CodeLens, CursorShape, Diagnostic, DiffHunk, GhostText, GitMark, InlayHint,
        LineNumbers, VimMode, VimModeIndicator,
    },
    navigation::{Breadcrumb, Crumb},
    primitives::Severity,
    theme::{ActiveTheme, Radius},
    typography::Caption,
};
use gpui::{App, Div, Entity, IntoElement, ParentElement, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, row, section, set},
};

/// The code the editors open with.
pub(super) const SAMPLE: &str = r#"use std::collections::HashMap;

/// A palette of named colors, looked up by the component that paints.
pub struct Palette {
    colors: HashMap<String, Color>,
    fallback: Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Palette {
    pub fn new(fallback: Color) -> Self {
        Self {
            colors: HashMap::new(),
            fallback,
        }
    }

    pub fn insert(&mut self, name: &str, color: Color) {
        self.colors.insert(name.to_string(), color);
    }

    /// The color for `name`, blended toward white by `lift`.
    pub fn lookup(&self, name: &str, lift: f32) -> Color {
        let base = self.colors.get(name).copied().unwrap_or(self.fallback);
        match lift {
            lift if lift <= 0.0 => base,
            lift if lift >= 1.0 => Color { red: 1.0, green: 1.0, blue: 1.0 },
            lift => Color {
                red: base.red + (1.0 - base.red) * lift,
                green: base.green + (1.0 - base.green) * lift,
                blue: base.blue + (1.0 - base.blue) * lift,
            },
        }
    }
}

fn main() {
    let mut palette = Palette::new(Color { red: 0.1, green: 0.1, blue: 0.1 });
    palette.insert("accent", Color { red: 0.21, green: 0.45, blue: 0.73 });
    let tints: Vec<Color> = (0..5u8).map(|step| palette.lookup("accent", step as f32 / 4.0)).collect();
    let count: u32 = tints.len();
    let spare = 12;
    println!("{count} tints of accent, the last {:?}", tints.last());
}
"#;

/// The byte where `needle` first starts in `text`.
pub(super) fn at(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("{needle:?} is in the sample"))
}

/// The line `needle` first sits on in `text`.
fn line_of(text: &str, needle: &str) -> usize {
    text[..at(text, needle)].matches('\n').count()
}

/// A bordered box an editor fills.
pub(super) fn framed(width: f32, height: f32, cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .w(px(width))
        .h(px(height))
        .rounded(theme.radius(Radius::Md))
        .border_1()
        .border_color(theme.colors.border)
        .overflow_hidden()
}

fn main_editor(window: &mut Window, cx: &mut App) -> Entity<CodeEditor> {
    window.use_keyed_state("editor-main", cx, |window, cx| {
        let mut editor = CodeEditor::new(SAMPLE, window, cx)
            .language("Rust")
            .minimap()
            .sticky_scroll()
            .rainbow_brackets()
            .rulers([100]);
        let map = at(SAMPLE, "HashMap;");
        let len = at(SAMPLE, "tints.len();");
        let spare = at(SAMPLE, "spare");
        let step = at(SAMPLE, "step as f32");
        editor.set_diagnostics(
            vec![
                Diagnostic {
                    range: map..map + 7,
                    severity: Severity::Info,
                    message: "a BTreeMap keeps a steady order".into(),
                },
                Diagnostic {
                    range: spare..spare + 5,
                    severity: Severity::Warning,
                    message: "unused variable: `spare`".into(),
                },
                Diagnostic {
                    range: len..len + 11,
                    severity: Severity::Danger,
                    message: "mismatched types: expected `u32`, found `usize`".into(),
                },
            ],
            cx,
        );
        editor.set_inlay_hints(
            vec![
                InlayHint {
                    offset: at(SAMPLE, "let base") + 8,
                    text: ": Color".into(),
                },
                InlayHint {
                    offset: at(SAMPLE, "\"accent\", step"),
                    text: "name: ".into(),
                },
                InlayHint {
                    offset: step,
                    text: "lift: ".into(),
                },
            ],
            cx,
        );
        editor.set_code_lenses(
            vec![CodeLens {
                line: line_of(SAMPLE, "    pub fn lookup"),
                items: vec!["3 references".into(), "Run test".into(), "Debug".into()],
            }],
            cx,
        );
        let main = line_of(SAMPLE, "fn main");
        editor.set_git_marks(
            vec![
                (main + 2, GitMark::Added),
                (main + 3, GitMark::Modified),
                (4, GitMark::Deleted),
            ],
            cx,
        );
        editor.toggle_breakpoint(main + 3, cx);
        editor
    })
}

pub fn editor(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let editor = main_editor(window, cx);
    let (line, column) = editor.read(cx).position();
    let crumbs = Breadcrumb::new(
        "editor-crumbs",
        [
            Crumb::new("src", "src"),
            Crumb::new("palette", "palette.rs"),
            Crumb::new("impl", "impl Palette"),
            Crumb::new("lookup", "lookup"),
        ],
    );
    section(
        "CodeEditor / EditorGutter / LineNumbers / FoldingControl / Minimap / Ruler / IndentGuide / BracketMatcher / RainbowBrackets / MultiCursor / SelectionHighlight / StickyScroll / Breadcrumb / InlayHints / CodeLens / InlineDecoration",
        "Rust, colored as it reads. The gutter holds breakpoints, problems, numbers, folds and git marks, and the blocks around the top line stay pinned as it scrolls. Cmd-D selects the next match, Cmd-Alt-Down adds a cursor, Alt-click adds one anywhere, Cmd-/ comments, Tab indents.",
        cx,
    )
    .child(crumbs)
    .child(probe("editor-main", framed(840.0, 460.0, cx).child(editor)))
    .child(Caption::new(format!("Ln {line}, Col {column}")))
}

pub fn looks(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let shape = keep("editor-shape", || CursorShape::Block, window, cx);
    let numbers = keep("editor-numbers", || LineNumbers::Relative, window, cx);
    let (now_shape, now_numbers) = (*shape.read(cx), *numbers.read(cx));
    let editor = window.use_keyed_state("editor-looks", cx, |window, cx| {
        CodeEditor::new(&SAMPLE[..at(SAMPLE, "impl Palette")], window, cx)
            .language("Rust")
            .show_whitespace()
    });
    editor.update(cx, |editor, cx| {
        editor.set_cursor_shape(now_shape, cx);
        editor.set_line_numbers(now_numbers, cx);
    });
    let (pick_shape, pick_numbers) = (shape.clone(), numbers.clone());
    let shapes = SegmentedControl::new(
        "editor-shapes",
        match now_shape {
            CursorShape::Line => "line",
            CursorShape::Block => "block",
            CursorShape::Underline => "underline",
        },
    )
    .segment("line", "Line", None)
    .segment("block", "Block", None)
    .segment("underline", "Underline", None)
    .on_change(move |key, _, cx| {
        let shape = match key.as_ref() {
            "line" => CursorShape::Line,
            "underline" => CursorShape::Underline,
            _ => CursorShape::Block,
        };
        set(&pick_shape, shape, cx)
    });
    let counting = SegmentedControl::new(
        "editor-counting",
        match now_numbers {
            LineNumbers::Absolute => "absolute",
            LineNumbers::Relative => "relative",
            LineNumbers::Hidden => "hidden",
        },
    )
    .segment("absolute", "Absolute", None)
    .segment("relative", "Relative", None)
    .segment("hidden", "Hidden", None)
    .on_change(move |key, _, cx| {
        let numbers = match key.as_ref() {
            "absolute" => LineNumbers::Absolute,
            "hidden" => LineNumbers::Hidden,
            _ => LineNumbers::Relative,
        };
        set(&pick_numbers, numbers, cx)
    });
    section(
        "WhitespaceRenderer / CursorRenderer",
        "Dots for spaces, and the cursor as a line, a block or an underline; it blinks, and holds still under reduced motion. Numbers count from the top or from the cursor.",
        cx,
    )
    .child(row().gap_4().child(shapes).child(counting))
    .child(framed(840.0, 260.0, cx).child(editor))
}

pub fn changes(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let text = &SAMPLE[at(SAMPLE, "impl Palette")..at(SAMPLE, "fn main")];
    let editor = window.use_keyed_state("editor-changes", cx, |window, cx| {
        let mut editor = CodeEditor::new(text, window, cx).language("Rust");
        editor.set_diff(
            vec![DiffHunk {
                line: line_of(text, "        let base"),
                added: 1,
                removed: vec!["        let base = self.colors[name];".into()],
            }],
            cx,
        );
        let end = at(text, "        match lift");
        editor.set_ghost_text(
            Some(GhostText {
                offset: end,
                text: "        // a lift past one stays white\n".into(),
            }),
            cx,
        );
        let cursor = end..end;
        editor.select([cursor], cx);
        editor
    });
    let readonly = window.use_keyed_state("editor-readonly", cx, |window, cx| {
        CodeEditor::new(&SAMPLE[..at(SAMPLE, "impl Palette")], window, cx)
            .language("Rust")
            .read_only()
    });
    let open = editor.read(cx).chat_line().is_some();
    let chat_editor = editor.clone();
    let toggle = Button::new(
        "editor-chat",
        if open {
            "Close inline chat"
        } else {
            "Open inline chat"
        },
    )
    .on_click(move |_, window, cx| {
        chat_editor.update(cx, |editor, cx| {
            let next = editor.chat_line().is_none().then_some(3);
            editor.set_chat(next, window, cx)
        })
    });
    section(
        "InlineDiff / GhostText / InlineChat / ReadOnlyBanner / VimModeIndicator",
        "A changed line under the one it replaced; a suggestion in grey that Tab takes; a prompt between lines; a banner over code that cannot change here; and the mode of a modal editor.",
        cx,
    )
    .child(row().child(toggle))
    .child(framed(840.0, 300.0, cx).child(editor))
    .child(framed(840.0, 180.0, cx).child(readonly))
    .child(
        row()
            .gap_4()
            .child(VimModeIndicator::new(VimMode::Normal))
            .child(VimModeIndicator::new(VimMode::Insert))
            .child(VimModeIndicator::new(VimMode::Visual).pending("d2"))
            .child(VimModeIndicator::new(VimMode::Replace)),
    )
}
