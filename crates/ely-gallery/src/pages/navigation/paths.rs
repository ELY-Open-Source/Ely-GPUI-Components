use ely_gpui_component::{
    forms::Choice,
    navigation::{Breadcrumb, Crumb, Pagination, Steps, Wizard},
    primitives::IconName,
    typography::Paragraph,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set, specimen, specimens},
};

pub fn breadcrumb(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let project = keep("crumb-project", || SharedString::from("ely"), window, cx);
    let went = keep("crumb-went", || None::<String>, window, cx);
    let now = project.read(cx).clone();
    let caption = went
        .read(cx)
        .clone()
        .unwrap_or_else(|| "click a level; one with a chevron lists its siblings".into());
    section(
        "Breadcrumb",
        "Each level is a place. A level with siblings opens them, the way an editor's path bar does.",
        cx,
    )
    .child(specimen(
        caption,
        probe(
            "breadcrumb",
            Breadcrumb::new(
                "breadcrumb",
                [
                    Crumb::new("home", "Home").icon(IconName::House),
                    Crumb::new("projects", "Projects"),
                    Crumb::new(now.clone(), now.clone()).siblings([
                        Choice::new("ely", "ely"),
                        Choice::new("zed", "zed"),
                        Choice::new("helix", "helix"),
                    ]),
                    Crumb::new("src", "src"),
                    Crumb::new("main", "main.rs"),
                ],
            )
            .on_select(move |level, value, _, cx| {
                if level == 2 {
                    set(&project, value.clone(), cx);
                }
                set(&went, Some(format!("went to {value} at level {level}")), cx);
            }),
        ),
        cx,
    ))
}

pub fn pagination(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let page = keep("pagination-page", || 6usize, window, cx);
    let now = *page.read(cx);
    section(
        "Pagination",
        "The first, the last, and the pages around this one. A gap stands for two or more pages. Type a page to go there.",
        cx,
    )
    .child(specimen(
        format!("page {now} of 20"),
        probe(
            "pagination",
            Pagination::new("pagination", now, 20)
                .jump()
                .on_change(move |to, _, cx| set(&page, to, cx)),
        ),
        cx,
    ))
}

fn setup_steps() -> [Choice; 3] {
    [
        Choice::new("account", "Account").note("Name and email"),
        Choice::new("plan", "Plan").note("Seats and billing"),
        Choice::new("confirm", "Confirm").note("Check and start"),
    ]
}

pub fn steps(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let step = keep("wizard-step", || 0usize, window, cx);
    let finished = keep("wizard-finished", || false, window, cx);
    let (now, done) = (*step.read(cx), *finished.read(cx));
    let body = match now {
        0 => "Who runs this workspace? One name, one address; both can change later.",
        1 => "Ten seats to start. Billing runs monthly and stops when you do.",
        _ => "Ready. The workspace opens as soon as you finish.",
    };
    let (walk, finish) = (step.clone(), finished);
    section(
        "Steps / Wizard",
        "Finished steps carry a check and take you back. The line into the current step fills as you advance.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "vertical",
                div().w(px(240.0)).child(Steps::new("steps-vertical", setup_steps(), 1).vertical()),
                cx,
            ))
            .child(specimen(
                if done { "finished" } else { "wizard" },
                probe(
                    "wizard",
                    div().w(px(560.0)).child(
                        Wizard::new(
                            "wizard",
                            setup_steps(),
                            now,
                            move |to, _, cx| set(&walk, to, cx),
                            move |_, cx| set(&finish, true, cx),
                        )
                        .content(div().h(px(72.0)).child(Paragraph::new(body))),
                    ),
                ),
                cx,
            )),
    )
}
