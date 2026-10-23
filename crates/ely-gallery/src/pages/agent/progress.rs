use std::time::Duration;

use ely_gpui_component::{
    agent::{AgentPlan, AgentProgress, AgentState, AgentStatus, AgentStep, AgentStepList},
    buttons::{Button, ButtonVariant},
    chat::StepState,
    data_display::Tone,
    lists::{Tree, TreeNode},
    primitives::IconName,
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

const PLAN: [&str; 5] = [
    "Read the theme's palette",
    "Measure contrast in both themes",
    "Lift the accents for dark",
    "Run the theme tests",
    "Write the release note",
];

/// Where each item of the plan stands once `done` have finished; the next one works.
fn plan(done: usize) -> Vec<(&'static str, StepState)> {
    PLAN.iter()
        .enumerate()
        .map(|(ix, label)| {
            let state = match ix.cmp(&done) {
                std::cmp::Ordering::Less => StepState::Done,
                std::cmp::Ordering::Equal => StepState::Working,
                std::cmp::Ordering::Greater => StepState::Waiting,
            };
            (*label, state)
        })
        .collect()
}

/// A step as the agent reports it.
fn step(title: &str, state: StepState, detail: Option<&str>, took: Option<u64>) -> AgentStep {
    AgentStep {
        title: title.to_string().into(),
        state,
        detail: detail.map(|text| text.to_string().into()),
        took: took.map(Duration::from_millis),
    }
}

pub fn steps(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let done = keep("agent-plan-done", || 2_usize, window, cx);
    let now = *done.read(cx);
    let state = if now == PLAN.len() {
        AgentState::Done
    } else {
        AgentState::Working
    };
    let current = PLAN.get(now).copied().unwrap_or("All done");
    let steps = [
        step(
            "Read src/theme/palette.rs",
            StepState::Done,
            None,
            Some(340),
        ),
        step(
            "Searched the web for oklch lift",
            StepState::Done,
            Some("Five sources, two on contrast in dark themes."),
            Some(2_100),
        ),
        step(
            "Ran the theme tests",
            StepState::Failed,
            Some("2 failed: lift_keeps_hue, dark_lifts_further"),
            Some(12_400),
        ),
        step("Fixing the lift for dark", StepState::Working, None, None),
        step("Run the tests again", StepState::Waiting, None, None),
    ];
    section(
        "AgentStepList / TaskTimeline / AgentPlan / TodoList / AgentStatus / AgentProgress",
        "An agent's work as it goes: its steps down a rail, its plan ticking off as each item finishes, where it stands in a word, and how far it has come. The button finishes the next item.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_center()
            .gap_3()
            .child(AgentStatus::new("agent-status", state))
            .child(AgentStatus::new("agent-status-waiting", AgentState::Waiting))
            .child(AgentStatus::new("agent-status-failed", AgentState::Failed)),
    )
    .child(
        div()
            .flex()
            .items_start()
            .gap_8()
            .child(
                div()
                    .w(px(320.))
                    .flex()
                    .flex_col()
                    .gap_5()
                    .child(AgentPlan::new("agent-plan", "Lift the dark theme", plan(now)))
                    .child(
                        AgentProgress::new("agent-progress", current, now, PLAN.len())
                            .elapsed(Duration::from_secs(95 + 40 * now as u64)),
                    ),
            )
            .child(div().w(px(360.)).child(AgentStepList::new("agent-steps", steps))),
    )
    .child(probe(
        "agent-advance",
        Button::new("agent-advance", "Finish the next item")
            .variant(ButtonVariant::Secondary)
            .disabled(now == PLAN.len())
            .on_click(move |_, _, cx| {
                let next = (*done.read(cx) + 1).min(PLAN.len());
                set(&done, next, cx)
            }),
    ))
}

pub fn agents(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let agent = |key: &str, name: &str, state: StepState| {
        let (note, tone) = match state {
            StepState::Waiting => ("waiting", Tone::Neutral),
            StepState::Working => ("working", Tone::Neutral),
            StepState::Done => ("done", Tone::Neutral),
            StepState::Failed => ("failed", Tone::Danger),
        };
        TreeNode::new(key.to_string(), name.to_string())
            .icon(IconName::Bot)
            .note(note)
            .tone(tone)
    };
    section(
        "SubAgentTree",
        "Agents an agent sent out, and theirs in turn, each with where it stands: lists::Tree, a node per agent with its state as its note, a failed one tinted.",
        cx,
    )
    .child(
        div().w(px(420.)).h(px(220.)).child(
            Tree::new(
                "agent-tree",
                [agent("lead", "Lead: lift the dark theme", StepState::Working).children([
                    agent("research", "Research: contrast in dark themes", StepState::Done),
                    agent("code", "Code: change the lift", StepState::Working).children([
                        agent("tests", "Tests: run the theme suite", StepState::Failed),
                        agent("fix", "Fix: dark lifts further", StepState::Working),
                    ]),
                    agent("notes", "Notes: write the release note", StepState::Waiting),
                ])],
            )
            .open(["lead", "code"])
            .size_full(),
        ),
    )
}
