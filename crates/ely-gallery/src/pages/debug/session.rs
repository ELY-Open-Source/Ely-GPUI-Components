use ely_gpui_component::{
    debug::{
        Breakpoint, BreakpointList, CallStack, DebugCommand, DebugState, DebugToolbar, StackFrame,
        Thread, ThreadList, ThreadState, ValueKind, Variable, VariablesPanel, Watch, WatchPanel,
    },
    forms::{InputEvent, TextInput},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::ui::{keep, row, section, set};

fn frames() -> Vec<StackFrame> {
    let frame = |function: &str, path: &str, line, library| StackFrame {
        function: function.to_string().into(),
        path: path.to_string().into(),
        line,
        library,
    };
    vec![
        frame("palette::Palette::lookup", "src/palette.rs", 48, false),
        frame("palette::tints::{{closure}}", "src/main.rs", 65, false),
        frame(
            "core::iter::adapters::map::map_fold",
            "core/src/iter/adapters/map.rs",
            88,
            true,
        ),
        frame(
            "core::iter::traits::iterator::Iterator::fold",
            "core/src/iter/traits/iterator.rs",
            2587,
            true,
        ),
        frame(
            "alloc::vec::Vec::extend_trusted",
            "alloc/src/vec/mod.rs",
            3301,
            true,
        ),
        frame("palette::main", "src/main.rs", 65, false),
        frame("std::rt::lang_start", "std/src/rt.rs", 206, true),
    ]
}

fn scopes() -> Vec<(SharedString, Vec<Variable>)> {
    let value = |name: &str, value: &str, kind, type_name: Option<&str>, children| Variable {
        name: name.to_string().into(),
        value: value.to_string().into(),
        kind,
        type_name: type_name.map(|name| name.to_string().into()),
        children,
    };
    vec![
        (
            "Locals".into(),
            vec![
                value(
                    "self",
                    "&Palette { colors: {…}, fallback: Color }",
                    ValueKind::Object,
                    Some("&Palette"),
                    vec![
                        value(
                            "colors",
                            "{\"accent\": Color, \"fg\": Color}",
                            ValueKind::Object,
                            Some("BTreeMap<String, Color>"),
                            vec![],
                        ),
                        value(
                            "fallback",
                            "Color { red: 0.1, green: 0.1, blue: 0.1 }",
                            ValueKind::Object,
                            Some("Color"),
                            vec![
                                value("red", "0.1", ValueKind::Number, Some("f32"), vec![]),
                                value("green", "0.1", ValueKind::Number, Some("f32"), vec![]),
                                value("blue", "0.1", ValueKind::Number, Some("f32"), vec![]),
                            ],
                        ),
                    ],
                ),
                value("name", "\"accent\"", ValueKind::Text, Some("&str"), vec![]),
                value("lift", "0.75", ValueKind::Number, Some("f32"), vec![]),
                value("clamped", "false", ValueKind::Bool, Some("bool"), vec![]),
            ],
        ),
        (
            "Statics".into(),
            vec![value(
                "WHITE",
                "Color { red: 1.0, green: 1.0, blue: 1.0 }",
                ValueKind::Object,
                Some("Color"),
                vec![],
            )],
        ),
    ]
}

pub fn session(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("debug-state", || DebugState::Paused, window, cx);
    let said = keep("debug-said", || None::<SharedString>, window, cx);
    let (now_state, told) = (*state.read(cx), said.read(cx).clone());
    let (command, tell) = (state.clone(), said.clone());
    section(
        "DebugToolbar",
        "The floating row that drives a session: continue or pause, the three steps while paused, restart and stop. Each tooltip names the usual key; the app binds it.",
        cx,
    )
    .child(row().child(DebugToolbar::new("debug-toolbar", now_state).on_command(move |which, _, cx| {
        let next = match which {
            DebugCommand::Pause => DebugState::Paused,
            DebugCommand::Continue | DebugCommand::Restart => DebugState::Running,
            _ => *command.read(cx),
        };
        set(&command, next, cx);
        set(&tell, Some(format!("{which:?}").into()), cx);
    })))
    .children(told.map(Caption::new))
}

pub fn breakpoints_and_stack(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let points = keep(
        "debug-breakpoints",
        || {
            let point = |path: &str, line, enabled, condition: Option<&str>, hits| Breakpoint {
                path: path.to_string().into(),
                line,
                enabled,
                condition: condition.map(|text| text.to_string().into()),
                hits,
            };
            vec![
                point("src/palette.rs", 48, true, None, 3),
                point("src/main.rs", 65, true, Some("step > 2"), 1),
                point("src/theme/tokens.rs", 112, false, None, 0),
            ]
        },
        window,
        cx,
    );
    let current = keep("debug-frame", || 0usize, window, cx);
    let libraries = keep("debug-libraries", || false, window, cx);
    let thread = keep("debug-thread", || 1u64, window, cx);
    let (now_points, now_current, now_libraries, now_thread) = (
        points.read(cx).clone(),
        *current.read(cx),
        *libraries.read(cx),
        *thread.read(cx),
    );
    let (toggle, remove, pick, show, choose) = (
        points.clone(),
        points.clone(),
        current.clone(),
        libraries.clone(),
        thread.clone(),
    );
    let threads = [
        Thread {
            id: 1,
            name: "main".into(),
            state: ThreadState::Stopped,
        },
        Thread {
            id: 2,
            name: "renderer".into(),
            state: ThreadState::Paused,
        },
        Thread {
            id: 3,
            name: "io-worker".into(),
            state: ThreadState::Paused,
        },
    ];
    section(
        "BreakpointList / BreakpointGutter / CallStack / ThreadList",
        "Every breakpoint with its condition and hits; the stack where the program waits, library frames folded until pressed; and the threads, one picked. The gutter's breakpoints are the editor's (CodeEditor).",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(
                div().w(px(400.)).child(
                    BreakpointList::new("debug-breakpoint-list", now_points)
                        .on_toggle(move |ix, on, _, cx| {
                            let mut next = toggle.read(cx).clone();
                            next[ix].enabled = on;
                            set(&toggle, next, cx)
                        })
                        .on_remove(move |ix, _, cx| {
                            let mut next = remove.read(cx).clone();
                            next.remove(ix);
                            set(&remove, next, cx)
                        }),
                ),
            )
            .child(
                div().w(px(400.)).flex().flex_col().gap_4().child(
                    CallStack::new("debug-stack", frames())
                        .current(now_current)
                        .libraries(now_libraries)
                        .on_pick(move |ix, _, cx| set(&pick, ix, cx))
                        .on_libraries(move |on, _, cx| set(&show, on, cx)),
                )
                .child(ThreadList::new("debug-threads", threads).selected(now_thread).on_pick(move |id, _, cx| set(&choose, id, cx))),
            ),
    )
}

pub fn values(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = keep(
        "debug-open",
        || vec![SharedString::from("Locals"), "Locals/self".into()],
        window,
        cx,
    );
    let watches = keep(
        "debug-watches",
        || {
            vec![
                Watch {
                    expression: "lift * 2.0".into(),
                    value: Ok(("1.5".into(), ValueKind::Number)),
                },
                Watch {
                    expression: "self.colors.len()".into(),
                    value: Ok(("2".into(), ValueKind::Number)),
                },
                Watch {
                    expression: "missing".into(),
                    value: Err("cannot find value `missing` in this scope".into()),
                },
            ]
        },
        window,
        cx,
    );
    let field = window.use_keyed_state("debug-watch-field", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Add an expression")
    });
    window.use_keyed_state("debug-watch-sub", cx, {
        let (field, watches) = (field.clone(), watches.clone());
        move |_, cx| {
            cx.subscribe(&field, move |_, input, event: &InputEvent, cx| {
                if *event != InputEvent::Submit {
                    return;
                }
                let text = input.read(cx).text().trim().to_string();
                if text.is_empty() {
                    return;
                }
                let mut next = watches.read(cx).clone();
                next.push(Watch {
                    expression: text.into(),
                    value: Ok(("0.75".into(), ValueKind::Number)),
                });
                set(&watches, next, cx);
                input.update(cx, |input, cx| input.set_text("", cx));
            })
        }
    });
    let (now_open, now_watches) = (open.read(cx).clone(), watches.read(cx).clone());
    let (toggle, remove) = (open.clone(), watches.clone());
    section(
        "VariablesPanel / WatchPanel / ConsoleREPL",
        "Variables by scope as a tree, colored by kind, with the value that changed at the last step washed; expressions watched at every step, errors in red. The REPL beside a paused program is the editor's DebugConsole.",
        cx,
    )
    .child(
        div()
            .flex()
            .gap_8()
            .child(
                div().w(px(460.)).child(
                    VariablesPanel::new("debug-variables", scopes())
                        .open(now_open)
                        .changed([SharedString::from("Locals/lift")])
                        .on_toggle(move |key, on, _, cx| {
                            let mut next = toggle.read(cx).clone();
                            next.retain(|kept| kept != key);
                            if on {
                                next.push(key.clone());
                            }
                            set(&toggle, next, cx)
                        }),
                ),
            )
            .child(
                div().w(px(340.)).child(
                    WatchPanel::new("debug-watch", now_watches, &field).on_remove(move |ix, _, cx| {
                        let mut next = remove.read(cx).clone();
                        next.remove(ix);
                        set(&remove, next, cx)
                    }),
                ),
            ),
    )
}
