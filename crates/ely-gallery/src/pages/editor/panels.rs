use ely_gpui_component::{
    editor::{
        ConsoleEntry, DebugConsole, Extension, ExtensionAction, ExtensionState, ExtensionsPanel,
        OutputPanel, Problem, ProblemsPanel,
    },
    forms::TextInput,
    primitives::Severity,
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::ui::{keep, section, set};

fn problems() -> Vec<Problem> {
    let problem = |path: &str, line, column, severity, message: &str, source: &str| Problem {
        path: path.to_string().into(),
        line,
        column,
        severity,
        message: message.to_string().into(),
        source: Some(source.to_string().into()),
    };
    vec![
        problem(
            "src/main.rs",
            45,
            21,
            Severity::Danger,
            "mismatched types: expected `u32`, found `usize`",
            "rustc",
        ),
        problem(
            "src/main.rs",
            46,
            8,
            Severity::Warning,
            "unused variable: `spare`",
            "rustc",
        ),
        problem(
            "src/palette.rs",
            0,
            23,
            Severity::Info,
            "a BTreeMap keeps a steady order",
            "clippy",
        ),
        problem(
            "src/theme/tokens.rs",
            112,
            4,
            Severity::Warning,
            "this function has too many arguments (8/7)",
            "clippy",
        ),
    ]
}

pub fn problems_and_output(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let shown = keep(
        "panels-shown",
        || vec![Severity::Danger, Severity::Warning, Severity::Info],
        window,
        cx,
    );
    let channel = keep("panels-channel", || SharedString::from("Build"), window, cx);
    let follow = keep("panels-follow", || true, window, cx);
    let cleared = keep("panels-cleared", || false, window, cx);
    let opened = keep("panels-problem", || None::<SharedString>, window, cx);
    let (now_shown, now_channel, now_follow, now_cleared, said) = (
        shown.read(cx).clone(),
        channel.read(cx).clone(),
        *follow.read(cx),
        *cleared.read(cx),
        opened.read(cx).clone(),
    );
    let lines: Vec<&str> = if now_cleared {
        Vec::new()
    } else if now_channel.as_ref() == "Build" {
        vec![
            "   Compiling ely-gpui-component v0.1.0",
            "warning: unused variable: `spare`",
            "error[E0308]: mismatched types",
            "   --> src/main.rs:46:22",
            "error: could not compile `palette` due to 1 previous error",
        ]
    } else {
        vec![
            "[info] rust-analyzer started",
            "[debug] indexing 214 files",
            "[info] indexed in 1.2 s",
        ]
    };
    let (show, pick, keep_up, clear, open) = (
        shown.clone(),
        channel.clone(),
        follow.clone(),
        cleared.clone(),
        opened.clone(),
    );
    let reset = cleared.clone();
    section(
        "ProblemsPanel / DiagnosticsList / OutputPanel",
        "Every problem by file, worst first, with toggles by severity; and a tool's log by channel, levels tinted, following the newest line.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(
                div().flex_1().child(
                    ProblemsPanel::new("panels-problems", problems())
                        .shown(now_shown)
                        .on_shown(move |next, _, cx| set(&show, next, cx))
                        .on_open(move |problem, _, cx| {
                            set(&open, Some(format!("Opened {}:{}.", problem.path, problem.line + 1).into()), cx)
                        }),
                ),
            )
            .child(
                div().w(px(400.)).h(px(220.)).child(
                    OutputPanel::new("panels-output", ["Build", "Language server"], now_channel, lines)
                        .follow(now_follow)
                        .on_channel(move |name, _, cx| {
                            set(&pick, name.clone(), cx);
                            set(&reset, false, cx);
                        })
                        .on_follow(move |on, _, cx| set(&keep_up, on, cx))
                        .on_clear(move |_, cx| set(&clear, true, cx)),
                ),
            ),
    )
    .children(said.map(Caption::new))
}

pub fn console_and_extensions(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let field = window.use_keyed_state("panels-console-field", cx, TextInput::new);
    let entries = keep(
        "panels-console",
        || {
            vec![
                ConsoleEntry::Input("tints.len()".into()),
                ConsoleEntry::Value("5".into(), "usize".into()),
                ConsoleEntry::Input("palette.lookup(\"accent\", 0.5).red".into()),
                ConsoleEntry::Value("0.605".into(), "f32".into()),
                ConsoleEntry::Input("spare + missing".into()),
                ConsoleEntry::Error("cannot find value `missing` in this scope".into()),
            ]
        },
        window,
        cx,
    );
    let states = keep(
        "panels-extensions",
        || {
            vec![
                ExtensionState::Installed,
                ExtensionState::Available,
                ExtensionState::Outdated,
                ExtensionState::Disabled,
            ]
        },
        window,
        cx,
    );
    let now_entries = entries.read(cx).clone();
    let now_states = states.read(cx).clone();
    let extension = |ix: usize,
                     id: &str,
                     name: &str,
                     publisher: &str,
                     description: &str,
                     version: &str,
                     installs: u64,
                     rating: f32| Extension {
        id: id.to_string().into(),
        name: name.to_string().into(),
        publisher: publisher.to_string().into(),
        description: description.to_string().into(),
        version: version.to_string().into(),
        installs,
        rating,
        state: now_states[ix],
    };
    let extensions = vec![
        extension(
            0,
            "rust",
            "Rust",
            "Ely Tools",
            "Syntax, completions and checks for Rust.",
            "1.4.2",
            2_410_000,
            4.8,
        ),
        extension(
            1,
            "toml",
            "Even Better TOML",
            "Tamasfe",
            "TOML with schemas and formatting.",
            "0.19.2",
            830_000,
            4.6,
        ),
        extension(
            2,
            "lens",
            "Git Lens",
            "Line Labs",
            "Who changed each line, and when.",
            "14.1.0",
            5_900_000,
            4.4,
        ),
        extension(
            3,
            "vim",
            "Vim",
            "Modal Folk",
            "Modal editing with the keys you know.",
            "1.27.3",
            1_200_000,
            4.2,
        ),
    ];
    let (eval, act) = (entries.clone(), states.clone());
    section(
        "DebugConsole / ExtensionsPanel / PluginList",
        "An expression prompt beside a paused program; and extensions with their reach, rating, and the one action each needs next.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(
                div().w(px(380.)).child(DebugConsole::new("panels-console", &field, now_entries).on_eval(move |text, _, cx| {
                    let mut next = eval.read(cx).clone();
                    next.push(ConsoleEntry::Input(text.clone()));
                    next.push(match text.trim().parse::<i64>() {
                        Ok(value) => ConsoleEntry::Value(value.to_string().into(), "i64".into()),
                        Err(_) => ConsoleEntry::Error(format!("cannot find value `{}` in this scope", text.trim()).into()),
                    });
                    set(&eval, next, cx)
                })),
            )
            .child(div().flex_1().child(ExtensionsPanel::new("panels-extensions", extensions).on_action(move |id, action, _, cx| {
                let ix = ["rust", "toml", "lens", "vim"].iter().position(|known| *known == id.as_ref()).expect("a listed extension");
                let mut next = act.read(cx).clone();
                next[ix] = match action {
                    ExtensionAction::Install | ExtensionAction::Enable | ExtensionAction::Update => ExtensionState::Installed,
                    ExtensionAction::Uninstall => ExtensionState::Available,
                };
                set(&act, next, cx)
            }))),
    )
}
