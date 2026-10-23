use ely_gpui_component::{
    buttons::Button,
    editor::{
        ActionKind, Call, CallHierarchy, CodeAction, CodeActionMenu, CompletionItem,
        CompletionMenu, FileHits, GoToSymbol, Hit, HoverInfo, ReferencesPanel, RenameInput,
        Signature, SignatureHelp, Symbol, SymbolKind, SymbolOutline, completions,
    },
    forms::TextInput,
    primitives::Severity,
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::ui::{keep, row, section, set};

fn items() -> Vec<CompletionItem> {
    vec![
        CompletionItem::new("lookup", SymbolKind::Method)
            .detail("fn(&self, &str, f32) -> Color")
            .docs("The color for a name, blended toward white by lift."),
        CompletionItem::new("insert", SymbolKind::Method).detail("fn(&mut self, &str, Color)"),
        CompletionItem::new("colors", SymbolKind::Field).detail("HashMap<String, Color>"),
        CompletionItem::new("fallback", SymbolKind::Field).detail("Color"),
        CompletionItem::new("len", SymbolKind::Method).detail("fn(&self) -> usize"),
        CompletionItem::new("Palette", SymbolKind::Struct).detail("struct Palette"),
        CompletionItem::new("loop", SymbolKind::Keyword),
    ]
}

pub fn hints(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let overload = keep("intel-overload", || 0usize, window, cx);
    let told = keep("intel-told", || None::<SharedString>, window, cx);
    let (over, said) = (*overload.read(cx), told.read(cx).clone());
    let chosen = completions(&items(), "lo")
        .iter()
        .position(|(item, _)| item.label.as_ref() == "lookup")
        .expect("lookup fits lo");
    let param = |label: &str, text: &str| {
        let start = label.find(text).expect("the parameter is in the label");
        start..start + text.len()
    };
    let (long, short) = (
        "fn lookup(&self, name: &str, lift: f32) -> Color",
        "fn lookup(&self, name: &str) -> Color",
    );
    let (pick, step, tell) = (told.clone(), overload.clone(), told.clone());
    let signatures = [
        Signature {
            label: long.into(),
            params: vec![param(long, "name: &str"), param(long, "lift: f32")],
            docs: Some("The color for a name, blended toward white by lift.".into()),
        },
        Signature {
            label: short.into(),
            params: vec![param(short, "name: &str")],
            docs: Some("The color for a name as it is.".into()),
        },
    ];
    section(
        "CompletionMenu / SignatureHelp / HoverInfo",
        "Completions as you type, the typed letters marked and the chosen one's documentation beside; the call's signature with the parameter at the cursor; and what a hover over a symbol reads.",
        cx,
    )
    .child(
        row()
            .items_start()
            .gap_8()
            .child(
                CompletionMenu::new("intel-menu", items())
                    .typed("lo")
                    .selected(chosen)
                    .on_pick(move |item, _, cx| set(&pick, Some(format!("Picked {}.", item.label).into()), cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        SignatureHelp::new("intel-signature", signatures)
                            .at(over, 1)
                            .on_step(move |next, _, cx| set(&step, next, cx)),
                    )
                    .child(
                        HoverInfo::new("intel-hover", "pub fn lookup(&self, name: &str, lift: f32) -> Color")
                            .problem(Severity::Warning, "`lift` outside 0..=1 is clamped")
                            .docs("The color for `name`, blended toward white by `lift`.")
                            .docs("Unknown names read as the fallback color.")
                            .link("Go to definition", move |_, cx| set(&tell, Some("Went to palette.rs:23.".into()), cx))
                            .link("Find references", |_, _| {}),
                    ),
            ),
    )
    .children(said.map(Caption::new))
}

fn symbols() -> Vec<Symbol> {
    vec![
        Symbol::new("Palette", SymbolKind::Struct, 3).children([
            Symbol::new("colors", SymbolKind::Field, 4).detail("HashMap<String, Color>"),
            Symbol::new("fallback", SymbolKind::Field, 5).detail("Color"),
        ]),
        Symbol::new("Color", SymbolKind::Struct, 9),
        Symbol::new("impl Palette", SymbolKind::Module, 15).children([
            Symbol::new("new", SymbolKind::Method, 16).detail("fn(Color) -> Self"),
            Symbol::new("insert", SymbolKind::Method, 23),
            Symbol::new("lookup", SymbolKind::Method, 28).detail("fn(&self, &str, f32) -> Color"),
        ]),
        Symbol::new("main", SymbolKind::Function, 42),
    ]
}

pub fn navigate(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let cursor = keep("intel-cursor", || 30usize, window, cx);
    let palette = keep("intel-goto", || false, window, cx);
    let incoming = keep("intel-incoming", || true, window, cx);
    let rename_open = keep("intel-rename", || false, window, cx);
    let field = window.use_keyed_state("intel-rename-field", cx, |window, cx| {
        let mut field = TextInput::new(window, cx);
        field.set_text("lookup", cx);
        field
    });
    let (line, open, inbound, renaming) = (
        *cursor.read(cx),
        *palette.read(cx),
        *incoming.read(cx),
        *rename_open.read(cx),
    );
    let (pick, go, close, flip) = (
        cursor.clone(),
        cursor.clone(),
        palette.clone(),
        incoming.clone(),
    );
    let opener = palette.clone();
    let (renamer, done, cancel) = (
        rename_open.clone(),
        rename_open.clone(),
        rename_open.clone(),
    );
    let hit = |line: usize, text: &str| {
        let start = text.find("lookup").expect("the line names lookup");
        let name = start..start + "lookup".len();
        Hit {
            line,
            text: text.to_string().into(),
            ranges: vec![name],
        }
    };
    let references = [
        FileHits {
            path: "src/palette.rs".into(),
            hits: vec![hit(
                27,
                "    pub fn lookup(&self, name: &str, lift: f32) -> Color {",
            )],
        },
        FileHits {
            path: "src/main.rs".into(),
            hits: vec![
                hit(
                    44,
                    "    let tints: Vec<Color> = (0..5u8).map(|step| palette.lookup(\"accent\", step as f32 / 4.0)).collect();",
                ),
                hit(51, "    let text = palette.lookup(\"fg\", 0.0);"),
            ],
        },
    ];
    let callers = Call {
        name: "lookup".into(),
        kind: SymbolKind::Method,
        place: "palette.rs:28".into(),
        calls: vec![
            Call {
                name: "main".into(),
                kind: SymbolKind::Function,
                place: "main.rs:44".into(),
                calls: Vec::new(),
            },
            Call {
                name: "paint_theme".into(),
                kind: SymbolKind::Function,
                place: "theme.rs:12".into(),
                calls: vec![Call {
                    name: "render".into(),
                    kind: SymbolKind::Method,
                    place: "app.rs:88".into(),
                    calls: Vec::new(),
                }],
            },
        ],
    };
    let fixes = [
        CodeAction::new("Change type of `count` to `usize`", ActionKind::QuickFix).preferred(),
        CodeAction::new("Convert with `u32::try_from`", ActionKind::QuickFix),
        CodeAction::new("Extract into function", ActionKind::Refactor),
        CodeAction::new("Inline variable", ActionKind::Refactor),
        CodeAction::new("Organize imports", ActionKind::Source),
    ];
    section(
        "CodeActionMenu / RenameInput / PeekView / ReferencesPanel / SymbolOutline / GoToSymbol / CallHierarchy / TypeHierarchy",
        "Fixes and refactors behind a lightbulb; renaming in place; where a symbol is used; a file's symbols as a tree that follows the cursor, or as a palette to jump by name; and who calls a function.",
        cx,
    )
    .child(
        row()
            .gap_4()
            .child(CodeActionMenu::new("intel-actions", fixes))
            .child(Button::new("intel-goto-open", "Go to symbol…").on_click(move |_, _, cx| set(&opener, true, cx)))
            .child(Button::new("intel-rename-open", "Rename lookup").on_click(move |_, _, cx| set(&renamer, true, cx)))
            .child(Caption::new(format!("Cursor on line {}.", line + 1))),
    )
    .children(renaming.then(|| {
        RenameInput::new("intel-rename-box", &field)
            .on_rename(move |_, _, cx| set(&done, false, cx))
            .on_cancel(move |_, cx| set(&cancel, false, cx))
    }))
    .child(
        row()
            .items_start()
            .gap_8()
            .child(
                div().w(px(300.)).h(px(240.)).child(
                    SymbolOutline::new("intel-outline", symbols())
                        .cursor(line)
                        .on_pick(move |line, _, cx| set(&pick, line, cx)),
                ),
            )
            .child(
                div().w(px(300.)).h(px(240.)).child(
                    CallHierarchy::new("intel-calls", callers, inbound)
                        .on_flip(move |incoming, _, cx| set(&flip, incoming, cx)),
                ),
            ),
    )
    .child(div().w(px(840.)).child(ReferencesPanel::new("intel-references", "lookup", references)))
    .children(open.then(|| {
        GoToSymbol::new("intel-goto-palette", symbols(), move |_, cx| set(&close, false, cx))
            .on_pick(move |line, _, cx| set(&go, line, cx))
    }))
}
