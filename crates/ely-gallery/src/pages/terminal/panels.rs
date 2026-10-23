use std::rc::Rc;

use ely_gpui_component::{
    forms::{Input, TextInput},
    terminal::{LogLevel, LogLine, LogViewer, Process, ProcessList},
    theme::{ActiveTheme, Radius},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::ui::{keep, section, set};

/// Two hundred lines of a service's day, the same each time.
fn log() -> Rc<Vec<LogLine>> {
    let targets = [
        "ely::shell",
        "ely::pty",
        "gpui::window",
        "ely::store",
        "net::http",
    ];
    let messages = [
        (LogLevel::Debug, "frame drawn in 4.1 ms"),
        (LogLevel::Info, "window opened at 1280×820"),
        (LogLevel::Trace, "key down: cmd-k"),
        (LogLevel::Info, "\x1b[1mzsh\x1b[0m started, pid 4821"),
        (LogLevel::Warn, "slow frame: 38 ms, over budget"),
        (LogLevel::Debug, "layout cached: 212 nodes"),
        (LogLevel::Info, "GET /api/quotes \x1b[32m200\x1b[0m 18 ms"),
        (
            LogLevel::Error,
            "GET /api/orders \x1b[31m503\x1b[0m retry in 2 s",
        ),
        (LogLevel::Info, "saved workspace, 3 panes"),
    ];
    Rc::new(
        (0..200)
            .map(|ix: usize| {
                let (level, message) = messages[(ix * 7 + ix / 3) % messages.len()];
                LogLine {
                    at: format!(
                        "09:{:02}:{:02}.{:03}",
                        ix / 60 % 60,
                        ix % 60,
                        ix * 37 % 1000
                    )
                    .into(),
                    level,
                    target: targets[ix * 3 % targets.len()].into(),
                    message: message.into(),
                }
            })
            .collect(),
    )
}

fn field(
    key: &'static str,
    placeholder: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Entity<TextInput> {
    window.use_keyed_state(key, cx, move |window, cx| {
        TextInput::new(window, cx).placeholder(placeholder)
    })
}

pub fn logs(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let lines = keep("terminal-log", log, window, cx);
    let least = keep("terminal-least", || LogLevel::Debug, window, cx);
    let follow = keep("terminal-follow", || true, window, cx);
    let query = field("terminal-log-query", "Filter lines", window, cx);
    let (now_lines, now_least, now_follow) =
        (lines.read(cx).clone(), *least.read(cx), *follow.read(cx));
    let text = query.read(cx).text().to_string();
    let (pick, keep_up) = (least.clone(), follow.clone());
    let theme = cx.theme();
    section(
        "LogViewer",
        "A log from a chosen level up, filtered as you type, following new lines until a scroll up lets go. Long logs draw only the lines in view.",
        cx,
    )
    .child(div().w(px(320.)).child(Input::new(&query)))
    .child(
        div()
            .w(px(840.))
            .h(px(280.))
            .p_2()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .child(
                LogViewer::new("terminal-log-viewer", now_lines)
                    .least(now_least)
                    .query(text)
                    .follow(now_follow)
                    .on_least(move |level, _, cx| set(&pick, level, cx))
                    .on_follow(move |on, _, cx| set(&keep_up, on, cx)),
            ),
    )
}

fn processes() -> Vec<Process> {
    let process = |pid, name: &str, user: &str, cpu, memory_mb: u64| Process {
        pid,
        name: name.to_string().into(),
        user: user.to_string().into(),
        cpu,
        memory: memory_mb * 1024 * 1024,
    };
    vec![
        process(4821, "zsh", "ely", 0.2, 6),
        process(512, "WindowServer", "_windowserver", 14.6, 412),
        process(9001, "gallery", "ely", 23.4, 188),
        process(733, "rust-analyzer", "ely", 41.8, 1_240),
        process(88, "launchd", "root", 0.1, 12),
        process(2210, "cargo", "ely", 6.3, 96),
        process(3302, "Safari", "ely", 8.9, 734),
        process(3303, "Safari Networking", "ely", 1.4, 88),
        process(401, "mds_stores", "root", 3.2, 140),
        process(6120, "codex", "ely", 2.7, 310),
    ]
}

pub fn process_list(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let running = keep("terminal-processes", processes, window, cx);
    let chosen = keep("terminal-chosen", Vec::<u32>::new, window, cx);
    let said = keep("terminal-ended", || None::<SharedString>, window, cx);
    let query = field("terminal-process-query", "Find a process", window, cx);
    let (now, picked, told) = (
        running.read(cx).clone(),
        chosen.read(cx).clone(),
        said.read(cx).clone(),
    );
    let text = query.read(cx).text().to_string();
    let (select, end, clear, tell) = (
        chosen.clone(),
        running.clone(),
        chosen.clone(),
        said.clone(),
    );
    section(
        "ProcessList",
        "Running processes, busiest first: found by a query, sorted by any column, and the selected ones ended.",
        cx,
    )
    .child(div().w(px(320.)).child(Input::new(&query)))
    .child(
        div().w(px(840.)).child(
            ProcessList::new("terminal-process-list", now)
                .query(text)
                .selected(picked)
                .on_select(move |pids, _, cx| set(&select, pids, cx))
                .on_end(move |pids, _, cx| {
                    let left: Vec<Process> = end.read(cx).iter().filter(|process| !pids.contains(&process.pid)).cloned().collect();
                    set(&end, left, cx);
                    set(&clear, Vec::new(), cx);
                    set(&tell, Some(format!("Ended {} process{}.", pids.len(), if pids.len() == 1 { "" } else { "es" }).into()), cx);
                }),
        ),
    )
    .children(told.map(Caption::new))
}
