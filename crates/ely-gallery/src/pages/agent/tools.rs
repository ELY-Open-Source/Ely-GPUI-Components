use std::time::Duration;

use ely_gpui_component::{
    agent::{Permission, PermissionPrompt, ToolApprovalDialog, ToolCallCard, ToolCallGroup},
    buttons::{Button, ButtonVariant},
    chat::StepState,
    theme::{ActiveTheme, TextSize},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const READ: &str = r#"{ "path": "src/theme/palette.rs" }"#;
const PALETTE: &str = "pub const ACCENT_LIFT: f32 = 0.18;\npub const SURFACE_LIFT: f32 = 0.04;";
const SEARCH: &str = "{\n  \"query\": \"oklch lift contrast\",\n  \"limit\": 5\n}";
const TESTS: &str = r#"{ "command": "cargo test --lib theme" }"#;
const WRITE: &str =
    "{\n  \"path\": \"src/theme/lift.rs\",\n  \"content\": \"pub fn lift(color, by) -> Hsla\"\n}";

pub fn calls(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    section(
        "ToolCallCard / ToolCallGroup",
        "Each call an assistant makes: a mark for where it stands, the tool, what it touched and how long it took; a press opens its arguments and what came back. Several calls fold under one line.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_2()
            .child(probe(
                "agent-call",
                div().w(px(560.)).child(
                    ToolCallCard::new("agent-read", "read_file", StepState::Done)
                        .summary("src/theme/palette.rs")
                        .took(Duration::from_millis(340))
                        .arguments(READ)
                        .result(PALETTE),
                ),
            ))
            .child(
                ToolCallCard::new("agent-search", "web_search", StepState::Working)
                    .summary("oklch lift contrast")
                    .arguments(SEARCH),
            )
            .child(
                ToolCallCard::new("agent-tests", "run_tests", StepState::Failed)
                    .summary("cargo test --lib theme")
                    .took(Duration::from_millis(12_400))
                    .arguments(TESTS)
                    .result("2 tests failed: lift_keeps_hue, dark_lifts_further"),
            )
            .child(
                ToolCallCard::new("agent-write", "write_file", StepState::Waiting)
                    .summary("src/theme/lift.rs")
                    .arguments(WRITE),
            )
            .child(
                div().w(px(280.)).child(
                    ToolCallCard::new("agent-narrow", "mcp__filesystem__read_multiple_files", StepState::Done)
                        .summary("3 files")
                        .took(Duration::from_millis(1_100))
                        .arguments(READ),
                ),
            )
            .child(
                ToolCallGroup::new("agent-group", "Read 3 files")
                    .child(ToolCallCard::new("agent-group-1", "read_file", StepState::Done).summary("src/theme/mod.rs").took(Duration::from_millis(120)))
                    .child(ToolCallCard::new("agent-group-2", "read_file", StepState::Done).summary("src/theme/tokens.rs").took(Duration::from_millis(95)))
                    .child(ToolCallCard::new("agent-group-3", "read_file", StepState::Done).summary("src/motion/curve.rs").took(Duration::from_millis(80))),
            ),
    )
}

pub fn approval(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let asking = keep("agent-asking", || false, window, cx);
    let heard = keep("agent-heard", || None::<SharedString>, window, cx);
    let allowed = keep("agent-allowed", || None::<Permission>, window, cx);
    let (open, now_heard, now_allowed) =
        (*asking.read(cx), heard.read(cx).clone(), *allowed.read(cx));
    let (opener, closer) = (asking.clone(), asking);
    let theme = cx.theme();
    let note = |text: String| {
        div()
            .text_size(theme.text_size(TextSize::Sm))
            .text_color(theme.colors.fg_muted)
            .child(text)
    };
    section(
        "ToolApprovalDialog / PermissionPrompt",
        "Asking first: a dialog before one call runs, with its arguments, where Escape denies; and a prompt in place that allows once, always, or not at all.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_center()
            .gap_3()
            .child(probe(
                "agent-approve-open",
                Button::new("agent-approve", "Review the call…")
                    .variant(ButtonVariant::Secondary)
                    .on_click(move |_, _, cx| set(&opener, true, cx)),
            ))
            .children(now_heard.map(|text| note(text.to_string()))),
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_2()
            .child(
                PermissionPrompt::new("agent-permission", "Let the assistant run commands in this folder?", move |answer, _, cx| {
                    set(&allowed, Some(answer), cx)
                })
                .detail("cargo test --lib theme"),
            )
            .children(now_allowed.map(|answer| {
                note(match answer {
                    Permission::Once => "Allowed once.".to_string(),
                    Permission::Always => "Always allowed in this folder.".to_string(),
                    Permission::Deny => "Denied.".to_string(),
                })
            })),
    )
    .children(open.then(|| {
        ToolApprovalDialog::new("agent-approval", "write_file", WRITE, move |approved, _, cx| {
            let verdict = if approved { "Ran write_file." } else { "Denied write_file." };
            set(&heard, Some(verdict.into()), cx);
            set(&closer, false, cx)
        })
        .detail("Writes a new file in src/theme.")
    }))
}
