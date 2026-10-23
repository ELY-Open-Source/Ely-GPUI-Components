use std::time::Duration;

use ely_gpui_component::{
    agent::{ChangeState, FileChange, MultiFileDiffReview},
    collab::Decision,
    terminal::{CommandBlock, CommandState},
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const LIFT_OLD: &str =
    "pub fn lift(color: Hsla, by: f32) -> Hsla {\n    color.mix(white(), by)\n}\n";
const LIFT_NEW: &str = "pub fn lift(color: Hsla, by: f32, dark: bool) -> Hsla {\n    let by = if dark { by * 1.5 } else { by };\n    color.mix(white(), by.min(1.0))\n}\n";
const TOKENS_OLD: &str =
    "pub const ACCENT_LIFT: f32 = 0.12;\npub const SURFACE_LIFT: f32 = 0.04;\n";
const TOKENS_NEW: &str =
    "pub const ACCENT_LIFT: f32 = 0.18;\npub const SURFACE_LIFT: f32 = 0.04;\n";
const NOTE_NEW: &str = "## 0.9\n\nDark themes lift their accents further.\n";

fn changes() -> Vec<FileChange> {
    [
        ("src/theme/lift.rs", LIFT_OLD, LIFT_NEW),
        ("src/theme/tokens.rs", TOKENS_OLD, TOKENS_NEW),
        ("CHANGELOG.md", "", NOTE_NEW),
    ]
    .into_iter()
    .map(|(path, old, new)| FileChange {
        path: path.into(),
        old: old.into(),
        new: new.into(),
        state: ChangeState::Proposed,
    })
    .collect()
}

/// `changes` after `decision`.
fn decided(mut changes: Vec<FileChange>, decision: Decision) -> Vec<FileChange> {
    let verdict = |accepted| {
        if accepted {
            ChangeState::Accepted
        } else {
            ChangeState::Rejected
        }
    };
    match decision {
        Decision::Accept(ix) => changes[ix].state = verdict(true),
        Decision::Reject(ix) => changes[ix].state = verdict(false),
        Decision::AcceptAll | Decision::RejectAll => {
            let state = verdict(decision == Decision::AcceptAll);
            for change in changes
                .iter_mut()
                .filter(|change| change.state == ChangeState::Proposed)
            {
                change.state = state;
            }
        }
    }
    changes
}

pub fn changes_section(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let files = keep("agent-changes", changes, window, cx);
    let now = files.read(cx).clone();
    let narrow = files.clone();
    section(
        "FileChangeCard / MultiFileDiff Review / CommandExecutionCard",
        "What an agent would change, file by file: how much moves, the diff a press away, and a verdict each or for all; in a side panel the counts and verdicts wrap below. A command it ran is terminal::CommandBlock.",
        cx,
    )
    .child(
        probe(
            "agent-changes",
            div().w(px(640.)).child(
                MultiFileDiffReview::new("agent-review", now.clone()).on_decide(move |decision, _, cx| {
                    let next = decided(files.read(cx).clone(), decision);
                    set(&files, next, cx)
                }),
            ),
        ),
    )
    .child(
        div().w(px(280.)).child(
            MultiFileDiffReview::new("agent-review-narrow", now).on_decide(move |decision, _, cx| {
                let next = decided(narrow.read(cx).clone(), decision);
                set(&narrow, next, cx)
            }),
        ),
    )
    .child(
        div().w(px(640.)).child(CommandBlock::new(
            "agent-command",
            "~/ely",
            "cargo test --lib theme",
            "\u{1b}[32m   Compiling\u{1b}[0m ely-gpui-component v0.9.0\n\u{1b}[32m    Finished\u{1b}[0m test profile in 8.4s\nrunning 12 tests\ntest theme::lift_keeps_hue ... \u{1b}[32mok\u{1b}[0m\ntest theme::dark_lifts_further ... \u{1b}[32mok\u{1b}[0m\n\ntest result: \u{1b}[32mok\u{1b}[0m. 12 passed; 0 failed",
            CommandState::Done { code: 0, took: Duration::from_millis(9_200) },
        )),
    )
}
