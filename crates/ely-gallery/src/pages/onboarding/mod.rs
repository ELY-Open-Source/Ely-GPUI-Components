use gpui::{AnyElement, App, Window, div, prelude::*};

mod flows;
mod help;

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 37,
    slug: "onboarding",
    title: "Onboarding & Help",
    summary: "A first run in steps, what is new called out, first things to do, and help where it is needed.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("onboarding-wizard", 200.0, 167.0),
    Step::UpAt("onboarding-wizard", 200.0, 167.0),
    Step::Type("Acme"),
    Step::Key("tab"),
    Step::Key("enter"),
    Step::Wait(200),
    Step::Shot("wizard-step-2"),
    Step::DownAt("onboarding-hotspot", 10.0, 10.0),
    Step::UpAt("onboarding-hotspot", 10.0, 10.0),
    Step::Wait(200),
    Step::Shot("hotspot-tip"),
    Step::Key("escape"),
    Step::DownAt("onboarding-help-panel", 120.0, 124.0),
    Step::UpAt("onboarding-help-panel", 120.0, 124.0),
    Step::Key("enter"),
    Step::Wait(200),
    Step::Shot("help-article"),
    Step::DownAt("onboarding-shortcuts", 20.0, 16.0),
    Step::UpAt("onboarding-shortcuts", 20.0, 16.0),
    Step::Wait(300),
    Step::Shot("cheatsheet"),
    Step::Key("escape"),
    Step::DownAt("onboarding-whats-new", 20.0, 16.0),
    Step::UpAt("onboarding-whats-new", 20.0, 16.0),
    Step::Wait(300),
    Step::Shot("whats-new"),
    Step::Key("escape"),
    Step::DownAt("onboarding-feedback", 20.0, 16.0),
    Step::UpAt("onboarding-feedback", 20.0, 16.0),
    Step::Wait(200),
    Step::Shot("feedback"),
    Step::Key("escape"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(flows::wizard(window, cx))
        .child(flows::highlights(window, cx))
        .child(flows::checklist(window, cx))
        .child(help::tips(window, cx))
        .child(help::panel(window, cx))
        .child(help::dialogs(window, cx))
        .child(help::feedback(window, cx))
        .into_any_element()
}
