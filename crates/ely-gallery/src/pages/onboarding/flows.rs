use ely_gpui_component::{
    buttons::{Button, ButtonVariant, IconButton},
    data_display::{Badge, Tone},
    forms::{FormField, Input, TextInput},
    lists::{List, ListItem},
    onboarding::{
        FeatureHighlight, Hotspot, OnboardingStep, OnboardingWizard, SetupChecklist, SetupTask,
    },
    primitives::{Icon, IconName},
    theme::{ActiveTheme, ControlSize, IconSize},
};
use gpui::{App, Entity, IntoElement, ParentElement, Styled, Window, div, prelude::*, px};

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

/// The wizard demo: its step, and whether it was finished.
#[derive(Default)]
struct Welcome {
    step: usize,
    finished: bool,
}

fn field(
    key: &'static str,
    hint: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TextInput> {
    window.use_keyed_state(key, cx, move |window, cx| {
        TextInput::new(window, cx).placeholder(hint)
    })
}

pub fn wizard(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("onboarding-welcome", Welcome::default, window, cx);
    let name = field("onboarding-name", "Acme", window, cx);
    let invite = field("onboarding-invite", "name@example.com", window, cx);
    let (step, finished) = (state.read(cx).step, state.read(cx).finished);
    let step_of = |key: &str, title: &str, body: &str| OnboardingStep {
        key: key.to_string().into(),
        title: title.to_string().into(),
        body: body.to_string().into(),
    };
    let steps = [
        step_of(
            "name",
            "Name your workspace",
            "It shows in the sidebar and in every invitation.",
        ),
        step_of(
            "invite",
            "Bring your team",
            "Add an address now, or later from Settings.",
        ),
        step_of(
            "ready",
            "You're set",
            "Your workspace is ready. Press Finish to open it.",
        ),
    ];
    let ready = step != 0 || !name.read(cx).text().trim().is_empty();
    let content = match step {
        0 => FormField::new("onboarding-name-field", "Workspace name")
            .child(Input::new(&name))
            .into_any_element(),
        1 => FormField::new("onboarding-invite-field", "Email")
            .child(Input::new(&invite))
            .into_any_element(),
        _ => div().into_any_element(),
    };
    let [walk, finish, skip, again] = [(); 4].map(|_| state.clone());
    let body = match finished {
        true => div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .child(format!("{} is ready.", name.read(cx).text()))
            .child(
                Button::new("onboarding-again", "Start over")
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Sm)
                    .on_click(move |_, _, cx| {
                        change(&again, cx, |welcome| *welcome = Welcome::default())
                    }),
            )
            .into_any_element(),
        false => OnboardingWizard::new(
            "onboarding-wizard",
            steps,
            step,
            move |to, _, cx| change(&walk, cx, |welcome| welcome.step = to),
            move |_, cx| change(&finish, cx, |welcome| welcome.finished = true),
        )
        .content(content)
        .ready(ready)
        .on_skip(move |_, cx| change(&skip, cx, |welcome| welcome.finished = true))
        .into_any_element(),
    };
    section(
        "OnboardingWizard · WelcomeTour",
        "A first run in a few steps: where it stands over a bar in as many parts, each step's title and line, and Next once its fields are ready. The welcome tour that follows is overlays::Tour.",
        cx,
    )
    .child(probe("onboarding-wizard", div().w(px(440.)).child(body)))
}

/// The highlights demo: what was dismissed.
#[derive(Default)]
struct Seen {
    hotspot: bool,
    highlight: bool,
}

pub fn highlights(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("onboarding-seen", Seen::default, window, cx);
    let (spot_seen, card_seen) = (state.read(cx).hotspot, state.read(cx).highlight);
    let [spot, card, reset] = [(); 3].map(|_| state.clone());
    let theme = cx.theme();
    let tool =
        |id: &'static str, icon: IconName| IconButton::new(id, icon).variant(ButtonVariant::Ghost);
    let pin = div()
        .relative()
        .child(tool("onboarding-pin", IconName::Pin))
        .when(!spot_seen, |pin| {
            pin.child(div().absolute().top_neg_1p5().right_neg_1p5().child(probe(
                "onboarding-hotspot",
                Hotspot::new(
                    "onboarding-hotspot",
                    "Pin a view",
                    "Keep the views you use most one press away.",
                    move |_, cx| change(&spot, cx, |seen| seen.hotspot = true),
                ),
            )))
        });
    let toolbar = div()
        .flex()
        .items_center()
        .gap_1()
        .p_1()
        .rounded_lg()
        .border_1()
        .border_color(theme.colors.border)
        .child(tool("onboarding-search", IconName::Search))
        .child(pin)
        .child(tool("onboarding-share", IconName::Share2));
    let highlight = (!card_seen).then(|| {
        FeatureHighlight::new(
            "onboarding-split",
            IconName::Columns2,
            "Split view",
            "Work on two files side by side, each with its own history.",
            |_, _| log::info!("gallery: split view tried"),
            move |_, cx| change(&card, cx, |seen| seen.highlight = true),
        )
    });
    let menu = List::new()
        .child(
            ListItem::new("onboarding-inbox", "Inbox")
                .leading(Icon::new(IconName::Inbox).size(IconSize::Sm)),
        )
        .child(
            ListItem::new("onboarding-automations", "Automations")
                .leading(Icon::new(IconName::Zap).size(IconSize::Sm))
                .trailing(Badge::new("New").tone(Tone::Accent)),
        );
    section(
        "Coachmark / Hotspot · FeatureHighlight / NewBadge",
        "A hotspot's rings call out something new, and a press or Enter opens its tip. A highlight names a feature with a New badge over it; the badge alone marks a new row. Coachmarks are overlays::Spotlight.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(div().w(px(240.)).flex().flex_col().gap_6().child(toolbar).child(menu))
            .child(div().w(px(360.)).children(highlight))
            .when(spot_seen || card_seen, |row| {
                row.child(
                    Button::new("onboarding-show-again", "Show again")
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm)
                        .on_click(move |_, _, cx| change(&reset, cx, |seen| *seen = Seen::default())),
                )
            }),
    )
}

/// Which setup tasks are done, and whether the list was hidden.
struct Setup {
    done: [bool; 4],
    hidden: bool,
}

pub fn checklist(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep(
        "onboarding-setup",
        || Setup {
            done: [true, false, false, false],
            hidden: false,
        },
        window,
        cx,
    );
    let (done, hidden) = (state.read(cx).done, state.read(cx).hidden);
    let [start, hide, reset] = [(); 3].map(|_| state.clone());
    const TASKS: [(&str, &str, &str); 4] = [
        (
            "profile",
            "Add your photo",
            "So your team knows who is who.",
        ),
        ("invite", "Invite a teammate", "Work is better shared."),
        (
            "connect",
            "Connect your calendar",
            "See meetings beside your tasks.",
        ),
        (
            "shortcuts",
            "Learn three shortcuts",
            "Command K opens everything.",
        ),
    ];
    let tasks = TASKS
        .iter()
        .zip(done)
        .map(|((key, title, body), done)| SetupTask {
            key: (*key).into(),
            title: (*title).into(),
            body: (*body).into(),
            done,
        });
    let body = match hidden {
        true => Button::new("onboarding-setup-again", "Show the checklist")
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Sm)
            .on_click(move |_, _, cx| {
                change(&reset, cx, |setup| {
                    *setup = Setup {
                        done: [true, false, false, false],
                        hidden: false,
                    }
                })
            })
            .into_any_element(),
        false => SetupChecklist::new(
            "onboarding-setup",
            "Get started",
            tasks,
            move |key, _, cx| {
                let at = TASKS
                    .iter()
                    .position(|(each, _, _)| *each == key.as_ref())
                    .expect("a setup task");
                change(&start, cx, |setup| setup.done[at] = true)
            },
        )
        .on_dismiss(move |_, cx| change(&hide, cx, |setup| setup.hidden = true))
        .into_any_element(),
    };
    section(
        "Checklist (first-run tasks)",
        "First things to do, counted in words and a bar. Start opens each; here it marks it done, and the check eases in. Once all are done, Hide puts it away.",
        cx,
    )
    .child(probe("onboarding-setup", div().w(px(400.)).child(body)))
}
