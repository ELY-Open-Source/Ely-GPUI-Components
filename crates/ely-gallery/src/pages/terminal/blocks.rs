use std::time::Duration;

use ely_gpui_component::{
    buttons::Button,
    terminal::{AnsiText, CommandBlock, CommandHistory, CommandState, PastCommand},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::ui::{keep, row, section, set};

const TESTS: &str = "running 5 tests\r\ntest palette::lookup_blends_toward_white ... \x1b[32mok\x1b[0m\r\ntest palette::unknown_names_fall_back ... \x1b[32mok\x1b[0m\r\n\r\ntest result: \x1b[32mok\x1b[0m. 5 passed; 0 failed; finished in 0.02s";

const FAILED: &str = "\x1b[1;31merror[E0308]\x1b[0m\x1b[1m: mismatched types\x1b[0m\r\n  \x1b[1;34m-->\x1b[0m src/main.rs:66:21\r\n   \x1b[1;34m|\x1b[0m\r\n\x1b[1;34m66\x1b[0m \x1b[1;34m|\x1b[0m     let count: u32 = tints.len();\r\n   \x1b[1;34m|\x1b[0m                \x1b[1;34m---\x1b[0m   \x1b[1;31m^^^^^^^^^^^\x1b[0m \x1b[1;31mexpected `u32`, found `usize`\x1b[0m";

fn history() -> Vec<PastCommand> {
    let past = |command: &str, cwd: &str, when: &str, code| PastCommand {
        command: command.to_string().into(),
        cwd: cwd.to_string().into(),
        when: when.to_string().into(),
        code,
    };
    vec![
        past("cargo test --lib palette", "~/palette", "just now", Some(0)),
        past("cargo build", "~/palette", "2 min ago", Some(101)),
        past("git status --short", "~/palette", "5 min ago", Some(0)),
        past("rg lookup src", "~/palette", "12 min ago", Some(0)),
        past(
            "cargo run -p ely-gallery",
            "~/Ely-GPUI-Components",
            "1 h ago",
            None,
        ),
        past(
            "git log --oneline -5",
            "~/Ely-GPUI-Components",
            "yesterday",
            Some(0),
        ),
    ]
}

pub fn blocks(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let folded = keep("terminal-folded", || false, window, cx);
    let open = keep("terminal-history", || false, window, cx);
    let said = keep("terminal-said", || None::<SharedString>, window, cx);
    let (now_folded, now_open, told) = (*folded.read(cx), *open.read(cx), said.read(cx).clone());
    let (fold, opener, close, pick, rerun) = (
        folded.clone(),
        open.clone(),
        open.clone(),
        said.clone(),
        said.clone(),
    );
    section(
        "CommandBlock / CommandHistory",
        "Each command a block: where it ran, how it ended and how long it took, its output folding away; copy it or run it again. Past commands open as a palette, found by any of their letters.",
        cx,
    )
    .child(
        div()
            .w(px(840.))
            .flex()
            .flex_col()
            .gap_3()
            .child(CommandBlock::new(
                "terminal-block-tests",
                "~/palette",
                "cargo test --lib palette",
                TESTS,
                CommandState::Done {
                    code: 0,
                    took: Duration::from_secs(3),
                },
            ))
            .child(
                CommandBlock::new(
                    "terminal-block-build",
                    "~/palette",
                    "cargo build",
                    FAILED,
                    CommandState::Done {
                        code: 101,
                        took: Duration::from_secs(9),
                    },
                )
                .folded(now_folded)
                .on_fold(move |folded, _, cx| set(&fold, folded, cx))
                .on_rerun(move |_, cx| set(&rerun, Some("Ran cargo build again.".into()), cx)),
            )
            .child(CommandBlock::new(
                "terminal-block-running",
                "~/Ely-GPUI-Components",
                "cargo run -p ely-gallery",
                "",
                CommandState::Running,
            )),
    )
    .child(row().child(Button::new("terminal-history-open", "Search history…").on_click(move |_, _, cx| set(&opener, true, cx))))
    .children(told.map(Caption::new))
    .children(now_open.then(|| {
        CommandHistory::new("terminal-history-palette", history(), move |_, cx| set(&close, false, cx))
            .on_pick(move |command, _, cx| set(&pick, Some(format!("Ran {command}.").into()), cx))
    }))
}

pub fn ansi(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "AnsiText",
        "Text colored by the escape codes in it, as a terminal prints it: the sixteen theme colors, bold, dim, italic, underline, inverse, the 256-color cube and true color. Other codes drop away.",
        cx,
    )
    .child(
        div()
            .w(px(840.))
            .flex()
            .flex_col()
            .gap_1()
            .child(AnsiText::new("\x1b[30m■\x1b[31m■\x1b[32m■\x1b[33m■\x1b[34m■\x1b[35m■\x1b[36m■\x1b[37m■ \x1b[90m■\x1b[91m■\x1b[92m■\x1b[93m■\x1b[94m■\x1b[95m■\x1b[96m■\x1b[97m■\x1b[0m  the sixteen, dark and bright"))
            .child(AnsiText::new("\x1b[1mbold\x1b[0m  \x1b[2mdim\x1b[0m  \x1b[3mitalic\x1b[0m  \x1b[4munderline\x1b[0m  \x1b[9mstruck\x1b[0m  \x1b[7m inverse \x1b[0m"))
            .child(AnsiText::new("\x1b[38;5;24m██\x1b[38;5;30m██\x1b[38;5;36m██\x1b[38;5;72m██\x1b[38;5;108m██\x1b[38;5;144m██\x1b[38;5;180m██\x1b[38;5;216m██\x1b[0m  the cube  \x1b[38;2;81;130;193m██\x1b[38;2;0;149;137m██\x1b[38;2;170;115;43m██\x1b[0m  true color"))
            .child(AnsiText::new("\x1b[42;30m PASS \x1b[0m palette  \x1b[41;97m FAIL \x1b[0m tokens  \x1b[43;30m SKIP \x1b[0m motion")),
    )
}
