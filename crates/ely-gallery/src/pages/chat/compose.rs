use ely_gpui_component::{
    buttons::SegmentedControl,
    chat::{
        Attachment, AttachmentButton, AttachmentChip, ContextChips, CostEstimator, InputHint,
        Parameter, ParameterPanel, PromptInput, SystemPromptEditor, TokenCounter, VoiceInputButton,
        VoiceWaveform,
    },
    data_display::UsageBar,
    forms::{Choice, Select, TextInput},
    menus::{DropdownMenu, Menu, MenuItem},
    primitives::IconName,
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};
use web_time::Instant;

use crate::{
    probe::probe,
    ui::{keep, noise, section, set},
};

const SYSTEM: &str = "You answer about color in plain words. Cite sources.";

/// The composer's text, which the prompt menu fills.
pub fn draft(window: &mut Window, cx: &mut App) -> Entity<TextInput> {
    window.use_keyed_state("chat-compose", cx, |window, cx| {
        let mut input = TextInput::new(window, cx)
            .multi_line(1, 8)
            .placeholder("Ask anything, / for commands, @ for context");
        input.set_text("Compare these two lifts in dark mode", cx);
        input
    })
}

/// A rough count for the demo; a host counts with its model's tokenizer.
fn tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

fn attachments() -> Vec<Attachment> {
    vec![
        Attachment {
            key: "olive".into(),
            name: "atrium-olive.jpg".into(),
            bytes: 184_000,
            preview: Some(asset!("atrium-olive.jpg").into()),
            progress: None,
        },
        Attachment {
            key: "notes".into(),
            name: "lift-notes.pdf".into(),
            bytes: 482_000,
            preview: None,
            progress: Some(0.6),
        },
    ]
}

pub fn composer(
    draft: &Entity<TextInput>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let files = keep("chat-attachments", attachments, window, cx);
    let context = keep(
        "chat-context",
        || {
            vec![
                (SharedString::from("Color page"), IconName::Globe),
                (SharedString::from("Selection in lift.rs"), IconName::Code),
            ]
        },
        window,
        cx,
    );
    let busy = keep("chat-busy", || false, window, cx);
    let recording = keep("chat-recording", || false, window, cx);
    let (now_files, now_context, now_busy, now_recording) = (
        files.read(cx).clone(),
        context.read(cx).clone(),
        *busy.read(cx),
        *recording.read(cx),
    );
    let (removed, dropped, picked) = (files.clone(), files.clone(), files.clone());
    let chips = div()
        .flex()
        .flex_wrap()
        .gap_2()
        .children(now_files.iter().enumerate().map(|(ix, item)| {
            let removed = removed.clone();
            AttachmentChip::new(("chat-attachment", ix), item.clone()).on_remove(move |_, cx| {
                let mut next = removed.read(cx).clone();
                next.remove(ix);
                set(&removed, next, cx)
            })
        }));
    let context_chips = {
        let context = context.clone();
        ContextChips::new("chat-context", now_context.clone()).on_remove(move |ix, _, cx| {
            let mut next = context.read(cx).clone();
            next.remove(ix);
            set(&context, next, cx)
        })
    };
    let add = |paths: Vec<std::path::PathBuf>, files: &Entity<Vec<Attachment>>, cx: &mut App| {
        let mut next = files.read(cx).clone();
        next.extend(paths.iter().map(|path| {
            let name = path
                .file_name()
                .expect("a chosen file has a name")
                .to_string_lossy()
                .to_string();
            Attachment {
                key: name.clone().into(),
                name: name.into(),
                bytes: 0,
                preview: None,
                progress: Some(0.0),
            }
        }));
        set(files, next, cx)
    };
    let (sent, stopped, mentioned) = (busy.clone(), busy.clone(), context.clone());
    let mut input = PromptInput::new("chat-composer", draft, move |_, cx| set(&sent, true, cx))
        .above(chips)
        .above(context_chips)
        .tool(AttachmentButton::new("chat-attach", move |paths, _, cx| {
            add(paths, &picked, cx)
        }))
        .tool(VoiceInputButton::new(
            "chat-voice",
            now_recording,
            move |on, _, cx| set(&recording, on, cx),
        ))
        .commands(
            [
                Choice::new("summarize", "/summarize — shorten the thread"),
                Choice::new("translate", "/translate — into another language"),
                Choice::new("explain", "/explain — step by step"),
            ],
            |command, _, _| log::info!("gallery: command {command}"),
        )
        .context(
            [
                Choice::new("lift.rs", "lift.rs"),
                Choice::new("theme.rs", "theme.rs"),
                Choice::new("notes", "lift-notes.pdf"),
            ],
            move |item, _, cx| {
                let mut next = mentioned.read(cx).clone();
                next.push((item.clone(), IconName::FileText));
                set(&mentioned, next, cx)
            },
        )
        .on_drop(move |paths, _, cx| add(paths, &dropped, cx));
    if now_busy {
        input = input.busy(move |_, cx| set(&stopped, false, cx));
    }
    section(
        "PromptInput / Composer / AttachmentButton / AttachmentChip / AttachmentPreview / PasteImagePreview / DragDropOverlay / ContextChips / SlashCommandMenu / ContextMentionMenu / SendButton / InputHint",
        "Where a message is written: attachments and context above the field, tools below, send at the end; Enter sends and Shift-Enter breaks the line. A slash at the start offers commands, an at sign offers context, and files dropped on it attach.",
        cx,
    )
    .child(probe(
        "chat-composer",
        div()
            .w(px(640.))
            .flex()
            .flex_col()
            .gap_2()
            .child(input)
            .child(div().flex().justify_end().child(InputHint)),
    ))
    .child(if now_recording {
        div().w(px(640.)).child(VoiceWaveform::new("chat-waveform", *keep("chat-since", Instant::now, window, cx).read(cx), levels())).into_any_element()
    } else {
        div().into_any_element()
    })
}

fn levels() -> Vec<f32> {
    let mut wave = noise(11);
    (0..40)
        .map(|_| (0.15 + 0.85 * wave() as f32).min(1.0))
        .collect()
}

pub fn pickers(
    draft: &Entity<TextInput>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let model = keep("chat-model", || SharedString::from("large"), window, cx);
    let mode = keep("chat-mode-pick", || SharedString::from("chat"), window, cx);
    let tools = keep("chat-tools", || (true, false, false), window, cx);
    let (now_model, now_mode, (web, code, images)) = (
        model.read(cx).clone(),
        mode.read(cx).clone(),
        *tools.read(cx),
    );
    let flip = |tools: &Entity<(bool, bool, bool)>, which: usize| {
        let tools = tools.clone();
        move |_: &mut Window, cx: &mut App| {
            let (mut web, mut code, mut images) = *tools.read(cx);
            match which {
                0 => web = !web,
                1 => code = !code,
                _ => images = !images,
            }
            set(&tools, (web, code, images), cx)
        }
    };
    let fill = |draft: &Entity<TextInput>, prompt: &'static str| {
        let draft = draft.clone();
        move |_: &mut Window, cx: &mut App| {
            log::info!("gallery: prompt template");
            draft.update(cx, |input, cx| input.set_text(prompt, cx))
        }
    };
    section(
        "ModelSelector / ModeSelector / ToolToggleMenu / PromptTemplateMenu",
        "The model is a forms::Select with a note on each; the mode a buttons::SegmentedControl; tools and saved prompts are menus::DropdownMenu, one with checks and one that inserts a prompt.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .child(
                div().w(px(220.)).child(
                    Select::new(
                        "chat-model",
                        [
                            Choice { note: Some("Best for hard problems".into()), ..Choice::new("large", "Ely Large") },
                            Choice { note: Some("Fast for everyday work".into()), ..Choice::new("small", "Ely Small") },
                        ],
                    )
                    .selected(now_model)
                    .on_change(move |next, _, cx| set(&model, next.clone(), cx)),
                ),
            )
            .child(
                SegmentedControl::new("chat-mode-pick", now_mode)
                    .segment("chat", "Chat", None)
                    .segment("research", "Research", Some(IconName::Globe))
                    .segment("code", "Code", Some(IconName::Code))
                    .on_change(move |next, _, cx| set(&mode, next.clone(), cx)),
            )
            .child(DropdownMenu::new(
                "chat-tools",
                "Tools",
                Menu::new()
                    .item(MenuItem::check("Web search", web).on_click(flip(&tools, 0)))
                    .item(MenuItem::check("Run code", code).on_click(flip(&tools, 1)))
                    .item(MenuItem::check("Make images", images).on_click(flip(&tools, 2))),
            ))
            .child(probe(
                "chat-templates",
                DropdownMenu::new(
                    "chat-templates",
                    "Prompts",
                    Menu::new()
                        .item(MenuItem::new("Review this code").on_click(fill(draft, "Review the code below. List bugs first, then anything unclear, each with its line and a fix.")))
                        .item(MenuItem::new("Explain like a textbook").on_click(fill(draft, "Explain the idea below to a newcomer: define terms before using them, one idea per paragraph."))),
                ),
            )),
    )
}

pub fn tuning(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let values = keep("chat-params", || (0.7, 0.95, 1024.0), window, cx);
    let (temperature, top_p, longest) = *values.read(cx);
    let system = window.use_keyed_state("chat-system", cx, |window, cx| {
        let mut input = TextInput::new(window, cx).multi_line(3, 8);
        input.set_text(SYSTEM, cx);
        input
    });
    let reset = system.clone();
    let system_tokens = tokens(system.read(cx).text());
    let changed = values.clone();
    section(
        "TemperatureSlider / ParameterPanel / SystemPromptEditor / TokenCounter / ContextWindowMeter / CostEstimator",
        "How answers are made: each knob a slider with its value; the instructions every answer follows, counted against their limit; how full the context is; and what a message may cost. The context meter is data_display::UsageBar.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .flex()
            .flex_col()
            .gap_5()
            .child(ParameterPanel::new(
                "chat-parameters",
                [
                    Parameter { key: "temperature".into(), label: "Temperature".into(), value: temperature, min: 0.0, max: 2.0, step: 0.1, hint: Some("Higher reads looser; lower, steadier.".into()) },
                    Parameter { key: "top_p".into(), label: "Top p".into(), value: top_p, min: 0.0, max: 1.0, step: 0.05, hint: None },
                    Parameter { key: "longest".into(), label: "Longest answer, in tokens".into(), value: longest, min: 256.0, max: 8192.0, step: 256.0, hint: None },
                ],
                move |key, value, _, cx| {
                    let (mut t, mut p, mut l) = *changed.read(cx);
                    match key.as_ref() {
                        "temperature" => t = value,
                        "top_p" => p = value,
                        _ => l = value,
                    }
                    set(&changed, (t, p, l), cx)
                },
            ))
            .child(probe(
                "chat-system",
                div().w(px(560.)).child(
                    SystemPromptEditor::new("chat-system", &system, system_tokens, 400)
                        .on_reset(move |_, cx| reset.update(cx, |input, cx| input.set_text(SYSTEM, cx))),
                ),
            ))
            .child(
                UsageBar::new(200_000.0)
                    .part("System", 1_200.0)
                    .part("Messages", 38_400.0)
                    .part("Files", 64_000.0),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(TokenCounter::new(183_600).limit(200_000))
                    .child(CostEstimator::new("chat-cost", 2_400, 1_024, (3.0, 15.0))),
            ),
    )
}
