use std::time::Duration;

use ely_gpui_component::{
    buttons::Button,
    editor::{
        Project, ProjectSwitcher, RecentProjects, Task, TaskRunner, TaskState, ThemePreview,
        WelcomePage,
    },
    primitives::IconName,
    theme::{ActiveTheme, Mode, Palette, Theme},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::ui::{keep, row, section, set};

fn projects() -> Vec<Project> {
    let project = |name: &str, path: &str, branch: Option<&str>, opened: &str, pinned| Project {
        name: name.to_string().into(),
        path: path.to_string().into(),
        branch: branch.map(|branch| branch.to_string().into()),
        opened: opened.to_string().into(),
        pinned,
    };
    vec![
        project(
            "ely-gpui-component",
            "~/Documents/GitHub/Ely-GPUI-Components",
            Some("main"),
            "now",
            true,
        ),
        project(
            "palette",
            "~/Code/palette",
            Some("feature/lift"),
            "2 h ago",
            false,
        ),
        project("notes", "~/Code/notes", None, "yesterday", false),
        project("zed", "~/Code/zed", Some("main"), "3 days ago", false),
    ]
}

pub fn start(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let switching = keep("panels-switcher", || false, window, cx);
    let opened = keep("panels-opened", || None::<SharedString>, window, cx);
    let show = keep("panels-welcome", || true, window, cx);
    let tasks = keep(
        "panels-tasks",
        || {
            vec![
                TaskState::Passed(Duration::from_secs(14)),
                TaskState::Running,
                TaskState::Failed(101),
                TaskState::Idle,
            ]
        },
        window,
        cx,
    );
    let open_task = keep("panels-task-open", || Some(2usize), window, cx);
    let (now_switching, said, now_show, now_tasks, now_open) = (
        *switching.read(cx),
        opened.read(cx).clone(),
        *show.read(cx),
        tasks.read(cx).clone(),
        *open_task.read(cx),
    );
    let mode = cx.theme().mode();
    let task = |name: &str, group: &str, command: &str, state| Task {
        name: name.to_string().into(),
        group: group.to_string().into(),
        command: command.to_string().into(),
        cwd: "~/Documents/GitHub/Ely-GPUI-Components".into(),
        env: vec![("RUST_LOG".into(), "info".into())],
        state,
    };
    let task_list = vec![
        task("check", "Build", "cargo check --all-targets", now_tasks[0]),
        task("gallery", "Run", "cargo run -p ely-gallery", now_tasks[1]),
        task(
            "test",
            "Test",
            "cargo test --lib --features test-support",
            now_tasks[2],
        ),
        task(
            "clippy",
            "Test",
            "cargo clippy --all-targets -- -D warnings",
            now_tasks[3],
        ),
    ];
    let (open, pin, close, pick, start_run, stop_run, fold) = (
        opened.clone(),
        opened.clone(),
        switching.clone(),
        opened.clone(),
        tasks.clone(),
        tasks.clone(),
        open_task.clone(),
    );
    let (opener, toggle) = (switching.clone(), show.clone());
    let recent = RecentProjects::new("panels-recent", projects())
        .on_open(move |ix, _, cx| {
            set(
                &open,
                Some(format!("Opened {}.", projects()[ix].name).into()),
                cx,
            )
        })
        .on_pin(move |ix, _, cx| {
            set(
                &pin,
                Some(format!("Pinned {}.", projects()[ix].name).into()),
                cx,
            )
        });
    section(
        "WelcomePage / RecentProjects / ProjectSwitcher / ThemePreview / TaskRunner / RunConfiguration",
        "Where an editor starts: ways in, recent projects and a switcher; each theme painted in its own colors; and tasks with how their last run went, one open to its configuration.",
        cx,
    )
    .child(
        div().w(px(840.)).child(
            WelcomePage::new("panels-welcome-page", "Ely Editor", "Quiet tools for careful code.")
                .action(IconName::FilePlus, "New file", Some("secondary-n"))
                .action(IconName::FolderOpen, "Open folder", Some("secondary-o"))
                .action(IconName::GitBranch, "Clone repository", None)
                .recent(recent)
                .show_at_start(now_show)
                .on_show(move |on, _, cx| set(&toggle, on, cx)),
        ),
    )
    .child(row().child(Button::new("panels-switch", "Switch project…").on_click(move |_, _, cx| set(&opener, true, cx))))
    .children(said.map(Caption::new))
    .child(
        row()
            .gap_6()
            .child(div().w(px(260.)).child(ThemePreview::new("panels-light", "Ely Light", Palette::light(false)).selected(mode == Mode::Light).on_pick(|_, cx| Theme::set_mode(Mode::Light, cx))))
            .child(div().w(px(260.)).child(ThemePreview::new("panels-dark", "Ely Dark", Palette::dark(false)).selected(mode == Mode::Dark).on_pick(|_, cx| Theme::set_mode(Mode::Dark, cx)))),
    )
    .child(
        div().w(px(840.)).child(
            TaskRunner::new("panels-tasks", task_list)
                .open(now_open)
                .on_open(move |ix, _, cx| set(&fold, ix, cx))
                .on_run(move |ix, _, cx| {
                    let mut next = start_run.read(cx).clone();
                    next[ix] = TaskState::Running;
                    set(&start_run, next, cx)
                })
                .on_stop(move |ix, _, cx| {
                    let mut next = stop_run.read(cx).clone();
                    next[ix] = TaskState::Idle;
                    set(&stop_run, next, cx)
                }),
        ),
    )
    .children(now_switching.then(|| {
        ProjectSwitcher::new("panels-switcher-palette", projects(), move |_, cx| set(&close, false, cx))
            .on_open(move |ix, _, cx| set(&pick, Some(format!("Switched to {}.", projects()[ix].name).into()), cx))
    }))
}
