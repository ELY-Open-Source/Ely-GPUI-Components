use ely_gpui_component::{
    documents::{
        BlockData, BlockEditor, BlockKind, Checklist, MarkdownEditor, MarkdownMode, Media,
        RichTextEditor, markdown_highlights,
    },
    forms::TextInput,
};
use gpui::{App, Entity, IntoElement, ParentElement, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const ATRIUM: &str = asset!("atrium.jpg");

const PEOPLE: [&str; 4] = ["ada", "grace", "linus", "margaret"];

const NOTES: &str = "## Release notes\n\nBlend every accent **toward white** by *lift*, clamped in gamma space. See [the palette docs](https://ely.dev/palette) or ask @ada.\n\n- Tokens stay in `src/theme`\n- ~~Raw pixels~~ never\n- [x] Light and dark";

const ESSAY: &str = "# Lift\n\nA lift blends a color toward white. **Half** a lift sits midway, and a *full* one is white.\n\n> Tokens live in one place; components only read them.\n\n| Token | Light | Dark |\n|:--|--:|--:|\n| accent | 0.50 | 0.62 |\n| focus | 0.40 | 0.55 |\n\n```rust\nfn lift(color: Color, by: f32) -> Color {\n    color.mix(WHITE, by)\n}\n```\n\n$$c' = c + (1 - c)\\,t$$";

/// A markdown field of several lines, styled as it is typed.
fn markdown(
    key: &'static str,
    text: &'static str,
    rows: usize,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TextInput> {
    window.use_keyed_state(key, cx, move |window, cx| {
        let mut field = TextInput::new(window, cx)
            .multi_line(rows, rows * 3)
            .highlighter(markdown_highlights);
        field.set_text(text, cx);
        field
    })
}

/// Draws `graph LR; a --> b --> c` as boxes in a row, as a host's renderer would.
fn draw(source: &str) -> Result<String, String> {
    let (head, body) = source
        .split_once(';')
        .ok_or("start with `graph LR;` then name the steps")?;
    if head.trim() != "graph LR" {
        return Err(format!("only `graph LR` draws here, not `{}`", head.trim()));
    }
    let nodes: Vec<&str> = body.split("-->").map(str::trim).collect();
    if nodes.iter().any(|node| node.is_empty()) {
        return Err("each arrow needs a step on both sides".into());
    }
    let (width, gap, height) = (120, 48, 44);
    let total = nodes.len() * width + (nodes.len() - 1) * gap;
    let mut svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{total}' height='{height}' viewBox='0 0 {total} {height}'>"
    );
    for (ix, node) in nodes.iter().enumerate() {
        let x = ix * (width + gap);
        svg.push_str(&format!("<rect x='{x}' y='1' width='{width}' height='{}' rx='8' fill='#f4f1ec' stroke='#8a8580'/><text x='{}' y='27' font-family='Helvetica' font-size='14' text-anchor='middle' fill='#2b2926'>{node}</text>", height - 2, x + width / 2));
        if ix + 1 < nodes.len() {
            let from = x + width;
            svg.push_str(&format!("<path d='M{} 22 H{}' stroke='#8a8580' stroke-width='1.5'/><path d='M{} 17 L{} 22 L{} 27' fill='none' stroke='#8a8580' stroke-width='1.5'/>", from + 4, from + gap - 4, from + gap - 10, from + gap - 4, from + gap - 10));
        }
    }
    svg.push_str("</svg>");
    Ok(svg)
}

fn block(kind: BlockKind, texts: &[&str]) -> BlockData {
    BlockData::new(kind, texts.iter().copied())
}

fn notes() -> Vec<BlockData> {
    vec![
        block(BlockKind::Heading(1), &["Project notes"]),
        block(
            BlockKind::Paragraph,
            &[
                "Type **/** on an empty line for every kind of block; `#`, `-`, `[]` and `>` at a line's start change it as you type.",
            ],
        ),
        block(BlockKind::Todo(true), &["Draft the palette"]),
        block(BlockKind::Todo(false), &["Check contrast in dark"]),
        block(
            BlockKind::Callout,
            &["Blocks drag by their handle; its menu turns, copies or deletes one."],
        ),
        block(
            BlockKind::Toggle(true),
            &[
                "Why gamma?",
                "Blending in linear light washes out the midtones; gamma keeps them.",
            ],
        ),
    ]
}

fn kinds() -> Vec<BlockData> {
    vec![
        block(
            BlockKind::Code,
            &["fn lift(color: Color, by: f32) -> Color {\n    color.mix(WHITE, by)\n}"],
        ),
        block(BlockKind::Math, &["c' = c + (1 - c)\\,t"]),
        block(
            BlockKind::Diagram,
            &["graph LR; tokens --> components --> app"],
        ),
        block(
            BlockKind::Table {
                columns: 3,
                merged: Vec::new(),
            },
            &[
                "Token", "Light", "Dark", "accent", "0.50", "0.62", "focus", "0.40", "0.55",
            ],
        ),
        block(
            BlockKind::Image(Media {
                width: 0.7,
                ..Media::new(ATRIUM)
            }),
            &["The atrium at noon"],
        ),
        block(
            BlockKind::Columns(2),
            &[
                "**Light**: paper white, ink near black.",
                "**Dark**: warm charcoal, text off-white.",
            ],
        ),
        block(
            BlockKind::Synced("palette-note".into()),
            &["Synced: edit either copy and both change."],
        ),
        block(BlockKind::Divider, &[]),
        block(
            BlockKind::Synced("palette-note".into()),
            &["Synced: edit either copy and both change."],
        ),
        block(
            BlockKind::Embed(Media::new("https://ely.dev")),
            &["The project site"],
        ),
    ]
}

pub fn editing(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let rich = markdown("doc-notes", NOTES, 6, window, cx);
    let essay = markdown("doc-essay", ESSAY, 24, window, cx);
    let mode = keep("doc-mode", || MarkdownMode::Visual, window, cx);
    let now_mode = *mode.read(cx);
    let notes = window.use_keyed_state("doc-blocks", cx, |window, cx| {
        BlockEditor::new(notes(), window, cx).people(PEOPLE)
    });
    let kinds = window.use_keyed_state("doc-kinds", cx, |window, cx| {
        BlockEditor::new(kinds(), window, cx)
            .people(PEOPLE)
            .diagrams(draw)
    });
    let tasks = keep(
        "doc-tasks",
        || {
            vec![
                ("Tokens in one place", true),
                ("Light and dark", true),
                ("Motion that settles", false),
                ("A site that shows it all", false),
            ]
        },
        window,
        cx,
    );
    let now_tasks = tasks.read(cx).clone();
    div()
        .child(
            section(
                "RichTextEditor / FixedFormatToolbar / FloatingFormatToolbar / LinkEditor / MentionMenu / EmojiAutocomplete / WordCount / ReadingTime",
                "Markdown that reads as it is written: styles show as they are typed and their marks recede. The toolbar above and the one over a selection apply formats; ⌘K edits the link at the caret. @ suggests people, a colon and two letters suggest emoji. The foot counts words, characters and minutes.",
                cx,
            )
            .child(probe("doc-rich", div().w(px(760.)).child(RichTextEditor::new("doc-rich", &rich).people(PEOPLE)))),
        )
        .child(
            section(
                "MarkdownEditor",
                "One document three ways: visual, where it reads as it will and a pressed block opens as its source; source, styled as typed; and split, the source beside what it makes.",
                cx,
            )
            .child(probe(
                "doc-markdown",
                div().w(px(840.)).child(
                    MarkdownEditor::new("doc-markdown", &essay, now_mode)
                        .people(PEOPLE)
                        .on_mode(move |next, _, cx| set(&mode, next, cx)),
                ),
            )),
        )
        .child(
            section(
                "BlockEditor / BlockHandle / SlashMenu / PlaceholderText",
                "Blocks like a notebook's. Hover one for its handle: drag it to move the block, press it to turn, copy or delete it. / on an empty line lists every kind, and an empty line says so. Undo covers text and blocks alike.",
                cx,
            )
            .child(probe(
                "doc-blocks",
                div().w(px(760.)).h(px(360.)).child(notes),
            )),
        )
        .child(
            section(
                "Block types / TableEditor / ImageEditorBlock / MathEditor / DiagramBlock / CodeBlockEditor",
                "Code colored as code, math set as it is typed, diagrams the app draws from their source, dividers, tables that grow and merge cells, pictures to resize and align, pages to open, columns, and synced copies that change together.",
                cx,
            )
            .child(div().w(px(760.)).h(px(560.)).child(kinds)),
        )
        .child(
            section(
                "TodoItem / Checklist",
                "To-dos under a title, with how many are done as a count and a bar; a done one goes quiet and struck through.",
                cx,
            )
            .child(
                div().w(px(360.)).child(
                    Checklist::new("doc-checklist", "Launch", now_tasks.clone()).on_toggle(move |ix, done, _, cx| {
                        let mut next = tasks.read(cx).clone();
                        next[ix].1 = done;
                        set(&tasks, next, cx);
                    }),
                ),
            ),
        )
}
