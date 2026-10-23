use ely_gpui_component::{
    account::{ForgotPassword, LoginForm, MagicLinkForm, OAuthButtons, SignupForm, TwoFactorInput},
    feedback::InlineMessage,
    layout::{Card, CardHeader},
    primitives::{Divider, Severity},
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{change, keep, section},
};

/// The sign-in demo: the provider on its way, and why the last try failed.
#[derive(Default)]
struct SignIn {
    provider: Option<SharedString>,
    error: Option<SharedString>,
}

pub fn sign_in(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("account-sign-in", SignIn::default, window, cx);
    let now = state.read(cx);
    let (provider, error) = (now.provider.clone(), now.error.clone());
    let [picked, tried] = [(); 2].map(|_| state.clone());
    let mut providers = OAuthButtons::new(
        "account-oauth",
        [
            ("github", "GitHub"),
            ("google", "Google"),
            ("apple", "Apple"),
        ],
        move |key, _, cx| change(&picked, cx, |state| state.provider = Some(key.clone())),
    );
    if let Some(provider) = provider {
        providers = providers.busy(provider);
    }
    let mut form = LoginForm::new("account-login", move |_, _, cx| {
        change(&tried, cx, |state| {
            state.error = Some("No account uses that email and password.".into())
        })
    })
    .on_forgot(|_, _| log::info!("gallery: forgot password"))
    .on_sign_up(|_, _| log::info!("gallery: sign up"));
    if let Some(error) = error {
        form = form.error(error);
    }
    section(
        "LoginForm · OAuthButtons",
        "Sign in with a provider or with an email and a password. Sign in rests until the address reads and a password is typed; Enter in a field signs in, and a failed try says why above the button.",
        cx,
    )
    .child(probe(
        "account-login",
        div().w(px(380.)).child(
            Card::new()
                .header(CardHeader::new("Sign in to Ely").description("Welcome back."))
                .child(div().flex().flex_col().gap_5().child(providers).child(Divider::horizontal().label("or")).child(form)),
        ),
    ))
}

/// The sign-up demo: why the last try failed.
#[derive(Default)]
struct SignUp {
    error: Option<SharedString>,
}

pub fn sign_up(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("account-sign-up", SignUp::default, window, cx);
    let error = state.read(cx).error.clone();
    let tried = state.clone();
    let mut form = SignupForm::new("account-signup", move |signup, _, cx| {
        let taken = format!("An account already uses {}.", signup.email);
        change(&tried, cx, |state| state.error = Some(taken.into()))
    })
    .terms("I agree to the terms of service")
    .on_sign_in(|_, _| log::info!("gallery: sign in"));
    if let Some(error) = error {
        form = form.error(error);
    }
    section(
        "SignupForm",
        "A name, an email and a new password, whose rules tick as they hold. Create account waits for every rule and the terms.",
        cx,
    )
    .child(div().w(px(380.)).child(Card::new().header(CardHeader::new("Create an account")).child(form)))
}

/// The email demos: whether each link went.
#[derive(Default)]
struct ByEmail {
    magic: bool,
    reset: bool,
}

pub fn by_email(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("account-by-email", ByEmail::default, window, cx);
    let now = state.read(cx);
    let (magic, reset) = (now.magic, now.reset);
    let [sent, other, asked, back] = [(); 4].map(|_| state.clone());
    section(
        "MagicLinkForm · ForgotPassword",
        "A link by email: one that signs in, one that sets a new password. Once sent, each says where it went and offers it again.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_wrap()
            .gap_6()
            .child(probe(
                "account-magic",
                div().w(px(360.)).child(
                    Card::new().header(CardHeader::new("Sign in by email")).child(
                        MagicLinkForm::new("account-magic", move |_, _, cx| change(&sent, cx, |state| state.magic = true))
                            .sent(magic)
                            .on_other(move |_, cx| change(&other, cx, |state| state.magic = false)),
                    ),
                ),
            ))
            .child(
                div().w(px(360.)).child(
                    Card::new().header(CardHeader::new("Reset your password")).child(
                        ForgotPassword::new("account-forgot", move |_, _, cx| change(&asked, cx, |state| state.reset = true))
                            .sent(reset)
                            .on_back(move |_, cx| change(&back, cx, |state| state.reset = false)),
                    ),
                ),
            ),
    )
}

/// The code the demo takes.
const CODE: &str = "246810";

/// The second-factor demo: codes that failed, and whether one passed.
#[derive(Default)]
struct Factor {
    failed: usize,
    passed: bool,
}

pub fn second_factor(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("account-factor", Factor::default, window, cx);
    let now = state.read(cx);
    let (failed, passed) = (now.failed, now.passed);
    let checked = state.clone();
    section(
        "TwoFactorInput",
        format!("Six boxes for an authenticator's code; a full code goes at once. A code that fails clears the boxes and says so. The demo takes {CODE}."),
        cx,
    )
    .child(probe(
        "account-factor",
        div().w(px(380.)).child(
            Card::new()
                .header(CardHeader::new("Two-step sign-in"))
                .child(
                    TwoFactorInput::new("account-factor", move |code, _, cx| {
                            change(&checked, cx, |state| match code.as_ref() == CODE {
                                true => state.passed = true,
                                false => state.failed += 1,
                            })
                        })
                        .failed(failed)
                        .on_recovery(|_, _| log::info!("gallery: recovery code")),
                )
                .children(passed.then(|| InlineMessage::new(Severity::Success, "Signed in."))),
        ),
    ))
}
