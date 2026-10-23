use std::rc::Rc;

use ely_gpui_component::{
    agent::{AgentState, AgentStatus, CostBreakdown, HumanInputRequest},
    charts::{BarChart, Series},
    chat::{continue_button, stop_button},
    data_display::Tone,
    debug::{Span, TimelineProfiler},
    forms::TextInput,
    tables::{Cell, Column, DataTable, Row},
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px, rems};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

pub fn interrupt_section(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("agent-run", || AgentState::Working, window, cx);
    let answer = keep("agent-answer", || None::<String>, window, cx);
    let field = window.use_keyed_state("agent-answer-field", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Or write an answer")
    });
    let (now_state, now_answer) = (*state.read(cx), answer.read(cx).clone());
    let (stop, go) = (state.clone(), state);
    let control = match now_state {
        AgentState::Working => {
            stop_button("agent-stop").on_click(move |_, _, cx| set(&stop, AgentState::Waiting, cx))
        }
        _ => continue_button("agent-stop")
            .on_click(move |_, _, cx| set(&go, AgentState::Working, cx)),
    };
    let request = HumanInputRequest::new(
        "agent-ask",
        "The dark accents read 4.1:1 on the card. Lift them to pass 4.5:1, or keep the brand color?",
        &field,
        move |text, _, cx| set(&answer, Some(text.to_string()), cx),
    )
    .choices(["Lift to 4.5:1", "Keep the brand color"]);
    let request = match now_answer {
        Some(text) => request.answered(text),
        None => request,
    };
    section(
        "InterruptButton / HumanInputRequest",
        "A way to stop an agent mid-task and pick it up again; and a question it waits on, with answers to pick or a line to write one. Once answered, the card keeps the answer.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .child(AgentStatus::new("agent-run-status", now_state))
            .child(probe("agent-stop", control)),
    )
    .child(probe("agent-ask", div().w(px(460.)).child(request)))
}

pub fn cost_section(_: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let turns = ["Turn 1", "Turn 2", "Turn 3", "Turn 4", "Turn 5", "Turn 6"];
    section(
        "TokenUsageChart / CostBreakdown",
        "Tokens each turn spent, split into what the model read, what it wrote and what came from the cache; and what the session cost, part by part.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_8()
            .child(
                div().w(px(420.)).child(
                    BarChart::new("agent-tokens", turns)
                        .series(Series::new("Input", [3_200., 8_400., 12_800., 6_100., 15_300., 9_700.]))
                        .series(Series::new("Output", [900., 2_100., 3_400., 1_200., 4_800., 2_600.]))
                        .series(Series::new("Cached", [0., 2_800., 7_900., 5_400., 11_200., 8_300.]))
                        .stacked(),
                ),
            )
            .child(
                div().w(px(320.)).child(
                    CostBreakdown::new("This session", "USD")
                        .part("Input", 0.42)
                        .part("Output", 1.18)
                        .part("Cache reads", 0.07)
                        .part("Tools", 0.25),
                ),
            ),
    )
}

fn trace() -> Rc<Vec<Span>> {
    let span = |track, name: &str, start, end| Span {
        track,
        name: name.to_string().into(),
        start,
        end,
    };
    Rc::new(vec![
        span(0, "lift the accents", 0.0, 4_200.0),
        span(1, "turn 1", 0.0, 2_600.0),
        span(1, "turn 2", 2_600.0, 4_200.0),
        span(2, "plan", 0.0, 900.0),
        span(2, "read_file", 950.0, 1_100.0),
        span(2, "edit", 1_150.0, 2_600.0),
        span(2, "run_tests", 2_650.0, 3_900.0),
        span(2, "answer", 3_950.0, 4_200.0),
    ])
}

fn evals() -> Vec<Row> {
    let row = |case: &'static str, pass: bool, score: f32, tokens: f64, seconds: f64| {
        let (word, tone) = match pass {
            true => ("Pass", Tone::Success),
            false => ("Fail", Tone::Danger),
        };
        Row::new(
            case,
            [
                case.into(),
                Cell::Tag(word.into(), tone),
                Cell::Progress(score),
                Cell::Number(tokens),
                Cell::Number(seconds),
            ],
        )
    };
    vec![
        row("Fix a failing test", true, 0.96, 18_400., 42.1),
        row("Rename across files", true, 0.91, 9_800., 18.6),
        row("Add a settings page", false, 0.48, 31_200., 96.4),
        row("Explain a stack trace", true, 0.88, 4_100., 7.9),
        row("Migrate a schema", false, 0.62, 22_700., 71.3),
    ]
}

pub fn trace_section(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let range = keep("agent-trace-range", || (0.0_f64, 4_600.0_f64), window, cx);
    let picked = keep("agent-trace-span", || Some(5_usize), window, cx);
    let (now_range, now_picked) = (*range.read(cx), *picked.read(cx));
    let timeline = TimelineProfiler::new(
        "agent-trace",
        ["Session", "Turn", "Call"],
        trace(),
        now_range,
    )
    .on_range(move |next, _, cx| set(&range, next, cx))
    .on_select(move |ix, _, cx| set(&picked, Some(ix), cx));
    let timeline = match now_picked {
        Some(ix) => timeline.selected(ix),
        None => timeline,
    };
    let columns = vec![
        Column::new("case", "Case"),
        Column::new("result", "Result").width(rems(6.)),
        Column::new("score", "Score").width(rems(10.)),
        Column::new("tokens", "Tokens").width(rems(6.)).end(),
        Column::new("time", "Time")
            .width(rems(6.))
            .end()
            .decimals(1)
            .suffix(" s"),
    ];
    section(
        "TraceViewer / EvalResultTable",
        "A run as spans on a track for each depth: the session, its turns, and the calls in each; the wheel zooms and a press picks a span. Then how an agent did on a set of cases, sortable by any column.",
        cx,
    )
    .child(div().w(px(760.)).child(timeline))
    .child(div().w(px(760.)).child(DataTable::new("agent-evals", columns).rows(evals())))
}
