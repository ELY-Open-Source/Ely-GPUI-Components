use std::path::PathBuf;

use ely_gpui_component::{
    editor::{FindOptions, FindWidget},
    forms::TextInput,
    layout::{PaneGroup, PaneId},
    navigation::{EditorTab, EditorTabs},
    primitives::IconName,
    terminal::{Launch, ShellChoice, ShellSelector, Terminal, TerminalToolbar},
    theme::{ActiveTheme, Radius},
};
use gpui::{
    App, AppContext as _, Axis, Entity, IntoElement, ParentElement, SharedString, Styled, Window,
    div, prelude::*, px,
};

use crate::{
    probe::probe,
    ui::{keep, set},
};

/// A build's output as a terminal printed it, colors and all.
const BUILD: &str = "\x1b[1;32m   Compiling\x1b[0m ely-gpui-component v0.1.0\r\n\x1b[1;33mwarning\x1b[0m\x1b[1m: unused variable: `spare`\x1b[0m\r\n  \x1b[1;34m-->\x1b[0m src/main.rs:47:9\r\n\x1b[1;33mwarning\x1b[0m\x1b[1m: field `hint` is never read\x1b[0m\r\n  \x1b[1;34m-->\x1b[0m src/palette.rs:12:5\r\n\x1b[1;32m    Finished\x1b[0m `dev` profile [unoptimized + debuginfo] target(s) in 4.21s\r\n\x1b[1;32m     Running\x1b[0m `target/debug/palette`\r\n\x1b[38;5;110m◆\x1b[0m 5 tints of accent, the last \x1b[38;2;232;243;252m██\x1b[0m #e8f3fc\r\n\x1b[2mdocs: https://github.com/ELY-Open-Source/Ely-GPUI-Components\x1b[0m\r\n";

const SHELLS: [(&str, &str); 3] = [
    ("bash", "/bin/bash"),
    ("zsh", "/bin/zsh"),
    ("sh", "/bin/sh"),
];

/// A shell with no startup files, its prompt the folder's name, in this repository.
fn launch(shell: usize) -> Launch {
    let (name, program) = SHELLS[shell];
    let args = match name {
        "bash" => vec!["--noprofile".into(), "--norc".into(), "-i".into()],
        "zsh" => vec!["-f".into(), "-i".into()],
        _ => vec!["-i".into()],
    };
    Launch {
        program: Some((program.into(), args)),
        cwd: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
        env: vec![
            ("PS1".into(), "\\W › ".into()),
            ("PROMPT".into(), "%1~ › ".into()),
            ("BASH_SILENCE_DEPRECATION_WARNING".into(), "1".into()),
        ],
    }
}

/// A pane group of shells, and the shell each pane holds.
type Panes = (Entity<PaneGroup>, Entity<Vec<(PaneId, Entity<Terminal>)>>);

/// Panes of `shell`, fresh for each `start`: a pick starts new shells even of the same kind.
fn panes(shell: usize, start: usize, window: &mut Window, cx: &mut App) -> Panes {
    let sessions = window.use_keyed_state(("terminal-sessions", start), cx, |_, _| Vec::new());
    let held = sessions.clone();
    let group = window.use_keyed_state(("terminal-panes", start), cx, move |_, _| {
        PaneGroup::new(move |pane, _, cx| session(&held, pane, shell, cx).into_any_element())
    });
    (group, sessions)
}

/// The shell pane `pane` holds, started the first time the pane draws.
fn session(
    sessions: &Entity<Vec<(PaneId, Entity<Terminal>)>>,
    pane: PaneId,
    shell: usize,
    cx: &mut App,
) -> Entity<Terminal> {
    if let Some((_, terminal)) = sessions.read(cx).iter().find(|(id, _)| *id == pane) {
        return terminal.clone();
    }
    let terminal = cx.new(|cx| Terminal::spawn(launch(shell), cx).expect("the shell starts"));
    sessions.update(cx, |all, _| all.push((pane, terminal.clone())));
    terminal
}

fn field(key: &'static str, text: &str, window: &mut Window, cx: &mut App) -> Entity<TextInput> {
    let seed = text.to_string();
    window.use_keyed_state(key, cx, move |window, cx| {
        let mut field = TextInput::new(window, cx);
        field.set_text(seed, cx);
        field
    })
}

/// The shell and the build's output, in tabs.
pub fn shell(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tab = keep("terminal-tab", || SharedString::from("shell"), window, cx);
    let shell = keep("terminal-shell-choice", || (0usize, 0usize), window, cx);
    let (now_tab, (now_shell, start)) = (tab.read(cx).clone(), *shell.read(cx));
    let (group, sessions) = panes(now_shell, start, window, cx);
    let build = window.use_keyed_state("terminal-build", cx, |_, cx| {
        Terminal::replay(BUILD.as_bytes(), 92, 12, cx)
    });
    let query = field("terminal-find", "warning", window, cx);
    let options = keep("terminal-options", FindOptions::default, window, cx);
    let searched = keep(
        "terminal-searched",
        || None::<(String, FindOptions)>,
        window,
        cx,
    );
    let wanted = (query.read(cx).text().to_string(), *options.read(cx));
    let error = keep("terminal-find-error", || None::<SharedString>, window, cx);
    if searched.read(cx).as_ref() != Some(&wanted) {
        let found = build.update(cx, |build, cx| build.find(&wanted.0, wanted.1, cx));
        set(&error, found.err().map(SharedString::from), cx);
        set(&searched, Some(wanted.clone()), cx);
    }
    let (total, current) = build.read(cx).matches();
    let focused = group.read(cx).focused();
    let active = session(&sessions, focused, now_shell, cx);
    let (pick, choose, splitter, clearer, stopper) = (
        tab.clone(),
        shell.clone(),
        group.clone(),
        active.clone(),
        active.clone(),
    );
    let shells = ShellSelector::new(
        "terminal-shells",
        SHELLS.map(|(name, program)| ShellChoice {
            name: name.into(),
            program: program.into(),
        }),
    )
    .default_shell(now_shell)
    .on_pick(move |ix, _, cx| set(&choose, (ix, start + 1), cx));
    let on_shell = now_tab.as_ref() == "shell";
    let exit = active
        .read(cx)
        .exit_status()
        .and_then(|status| status.code());
    let toolbar = TerminalToolbar::new(
        "terminal-toolbar",
        format!("{} — Ely-GPUI-Components", SHELLS[now_shell].0),
        exit,
    )
    .shells(shells)
    .on_split(move |_, cx| {
        splitter.update(cx, |group, cx| {
            let pane = group.focused();
            group.split(pane, Axis::Horizontal, cx);
        })
    })
    .on_clear(move |_, cx| clearer.update(cx, |terminal, cx| terminal.clear(cx)))
    .on_stop(move |_, cx| stopper.update(cx, |terminal, _| terminal.write(&b"\x03"[..])));
    let (step, flip) = (build.clone(), options.clone());
    let find = FindWidget::new("terminal-find-widget", &query, current, total)
        .options(*options.read(cx))
        .on_options(move |next, _, cx| set(&flip, next, cx))
        .on_step(move |forward, _, cx| step.update(cx, |build, cx| build.step(forward, cx)));
    let find = match error.read(cx).clone() {
        Some(message) => find.error(message),
        None => find,
    };
    let theme = cx.theme();
    div().child(
        div()
            .w(px(840.))
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .overflow_hidden()
            .child(probe(
                "terminal-tabs",
                div().w(px(838.)).child(
                    EditorTabs::new(
                        "terminal-tabs",
                        [
                            EditorTab::new("shell", SHELLS[now_shell].0).icon(IconName::Terminal),
                            EditorTab::new("build", "cargo run").icon(IconName::FileText),
                        ],
                    )
                    .selected(now_tab.clone())
                    .on_select(move |id, _, cx| set(&pick, id.clone(), cx)),
                ),
            ))
            .children(on_shell.then_some(toolbar))
            .child(probe(
                "terminal",
                div().w(px(838.)).h(px(300.)).relative().map(|body| {
                    if on_shell {
                        body.child(group)
                    } else {
                        body.child(build)
                            .child(div().absolute().top_1().right_4().child(find))
                    }
                }),
            )),
    )
}
