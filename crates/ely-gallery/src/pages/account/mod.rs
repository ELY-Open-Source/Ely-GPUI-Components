use gpui::{AnyElement, App, Window, div, prelude::*};

mod access;
mod billing;
mod sign_in;

use super::Page;
use crate::step::Step;

pub const PAGE: Page = Page {
    number: 36,
    slug: "account",
    title: "Account",
    summary: "Signing in and up, a link by email, a second factor, and the account once inside.",
    render,
    script: SCRIPT,
};

const SCRIPT: &[Step] = &[
    Step::DownAt("account-login", 150.0, 280.0),
    Step::UpAt("account-login", 150.0, 280.0),
    Step::Type("ada@example.com"),
    Step::Key("tab"),
    Step::Type("hunter22"),
    Step::Key("enter"),
    Step::Wait(200),
    Step::Shot("sign-in-failed"),
    Step::DownAt("account-magic", 150.0, 150.0),
    Step::UpAt("account-magic", 150.0, 150.0),
    Step::Type("ada@example.com"),
    Step::Key("enter"),
    Step::Wait(200),
    Step::Shot("link-sent"),
    Step::DownAt("account-factor", 120.0, 110.0),
    Step::UpAt("account-factor", 120.0, 110.0),
    Step::Type("123456"),
    Step::Wait(200),
    Step::Shot("code-failed"),
    Step::DownAt("account-user", 12.0, 12.0),
    Step::UpAt("account-user", 12.0, 12.0),
    Step::Wait(200),
    Step::Shot("user-menu"),
    Step::Key("escape"),
];

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(sign_in::sign_in(window, cx))
        .child(sign_in::sign_up(window, cx))
        .child(sign_in::by_email(window, cx))
        .child(sign_in::second_factor(window, cx))
        .child(access::switchers(window, cx))
        .child(access::profile(window, cx))
        .child(access::security_page(window, cx))
        .child(billing::plan(window, cx))
        .child(billing::history(window, cx))
        .child(billing::members(window, cx))
        .into_any_element()
}
