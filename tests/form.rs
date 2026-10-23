//! A sign-up form an app builds from Ely's public API alone.

use ely_gpui_component::{
    buttons::Button,
    forms::{Form, FormField, Input, TextInput},
    init_for_tests,
    primitives::FocusScope,
};
use gpui::{
    AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    TestAppContext, VisualTestContext, Window,
};

struct Signup {
    root: FocusHandle,
    name: Entity<TextInput>,
    email: Entity<TextInput>,
    sent: Vec<(String, String)>,
}

impl Render for Signup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let (name, email) = (self.name.clone(), self.email.clone());
        FocusScope::new(&self.root).root().child(
            Form::new("signup")
                .on_submit(move |_, cx| {
                    let filled = (
                        name.read(cx).text().to_string(),
                        email.read(cx).text().to_string(),
                    );
                    view.update(cx, |view, _| view.sent.push(filled));
                })
                .child(
                    FormField::new("name", "Name")
                        .required()
                        .child(Input::new(&self.name)),
                )
                .child(FormField::new("email", "Email").child(Input::new(&self.email)))
                .child(Button::new("save", "Create account")),
        )
    }
}

fn signup(cx: &mut TestAppContext) -> (Entity<Signup>, &mut VisualTestContext) {
    cx.update(init_for_tests);
    let (view, cx) = cx.add_window_view(|window, cx| Signup {
        root: cx.focus_handle(),
        name: cx.new(|cx| TextInput::new(window, cx)),
        email: cx.new(|cx| TextInput::new(window, cx)),
        sent: Vec::new(),
    });
    let root = view.read_with(cx, |view, _| view.root.clone());
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&root, cx);
    });
    cx.run_until_parked();
    (view, cx)
}

fn focused(field: &Entity<TextInput>, cx: &mut VisualTestContext) -> bool {
    cx.update(|window, cx| field.read(cx).focus_handle(cx).is_focused(window))
}

#[gpui::test]
fn tab_walks_the_fields_in_order(cx: &mut TestAppContext) {
    let (view, cx) = signup(cx);
    let (name, email) = view.read_with(cx, |view, _| (view.name.clone(), view.email.clone()));
    cx.simulate_keystrokes("tab");
    assert!(focused(&name, cx), "the first Tab reaches the name");
    cx.simulate_keystrokes("tab");
    assert!(focused(&email, cx), "the second reaches the email");
}

#[gpui::test]
fn typing_then_submitting_hands_the_owner_what_was_typed(cx: &mut TestAppContext) {
    let (view, cx) = signup(cx);
    cx.simulate_keystrokes("tab");
    cx.simulate_input("Ada");
    cx.simulate_keystrokes("tab");
    cx.simulate_input("ada@example.com");
    cx.simulate_keystrokes("secondary-enter");
    let sent = view.read_with(cx, |view, _| view.sent.clone());
    assert_eq!(sent, [("Ada".to_string(), "ada@example.com".to_string())]);
}
