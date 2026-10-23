use ely_gpui_component::{
    buttons::{ActionBar, Button, ButtonVariant},
    forms::{
        DirtyIndicator, FieldArray, Form, FormField, FormSection, InlineForm, Input, TextInput,
        is_email,
    },
};
use gpui::{
    App, AppContext, Entity, IntoElement, ParentElement, Styled, Window, div, prelude::*, px,
};

use super::text::field;
use crate::{
    probe::probe,
    ui::{keep, section, set, specimen},
};

type Saved = (String, String, String, Vec<String>);

fn ready(now: &Saved) -> bool {
    !now.0.trim().is_empty() && is_email(&now.1)
}

fn snapshot(fields: &[Entity<TextInput>; 3], links: &[Entity<TextInput>], cx: &App) -> Saved {
    let text = |field: &Entity<TextInput>| field.read(cx).text().to_string();
    let [a, b, c] = fields.each_ref().map(text);
    (a, b, c, links.iter().map(text).collect())
}

fn link(text: String, window: &mut Window, cx: &mut App) -> Entity<TextInput> {
    cx.new(|cx| {
        let mut input = TextInput::new(window, cx).placeholder("https://");
        input.set_text(text, cx);
        input
    })
}

pub fn form(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let name = field("form-name", window, cx, |input| {
        input.placeholder("Ada Lovelace")
    });
    let email = field("form-email", window, cx, |input| {
        input.placeholder("ada@example.com")
    });
    let bio = field("form-bio", window, cx, |input| {
        input.multi_line(3, 5).placeholder("A line about you")
    });
    let links = keep("form-links", Vec::<Entity<TextInput>>::new, window, cx);
    let saved = keep("form-saved", Saved::default, window, cx);
    let sent = keep("form-sent", || 0usize, window, cx);
    let fields = [name.clone(), email.clone(), bio.clone()];
    let now = snapshot(&fields, links.read(cx), cx);
    let dirty = now != *saved.read(cx);
    let problem = (!now.1.is_empty() && !is_email(&now.1))
        .then_some("Enter an address like name@example.com");
    let save = {
        let (fields, links, saved, sent) =
            (fields.clone(), links.clone(), saved.clone(), sent.clone());
        move |_: &mut Window, cx: &mut App| {
            let now = snapshot(&fields, links.read(cx), cx);
            if !ready(&now) {
                log::info!("gallery form: kept unsaved until name and email are filled");
                return;
            }
            set(&saved, now, cx);
            let count = *sent.read(cx) + 1;
            set(&sent, count, cx);
        }
    };
    let discard = {
        let (fields, links, saved) = (fields.clone(), links.clone(), saved.clone());
        move |window: &mut Window, cx: &mut App| {
            let (a, b, c, kept) = saved.read(cx).clone();
            for (field, text) in fields.iter().zip([a, b, c]) {
                field.update(cx, |input, cx| input.set_text(text, cx));
            }
            let rows = kept
                .into_iter()
                .map(|text| link(text, window, cx))
                .collect();
            set(&links, rows, cx);
        }
    };
    let rows = links.read(cx).clone();
    let (grow, shrink) = (links.clone(), links);
    let array = rows.iter().fold(
        FieldArray::new("form-links")
            .add_label("Add link")
            .on_add(move |window, cx| {
                let row = link(String::new(), window, cx);
                grow.update(cx, |links, cx| {
                    links.push(row);
                    cx.notify();
                });
            })
            .on_remove(move |ix, _, cx| {
                shrink.update(cx, |links, cx| {
                    links.remove(ix);
                    cx.notify();
                })
            }),
        |array, link| array.row(Input::new(link)),
    );
    let (save_click, count) = (save.clone(), *sent.read(cx));
    section(
        "Form / FormSection / FormField / FieldArray / DirtyIndicator",
        "Sections with room between them. Errors take the description's place; Cmd-Enter saves from any field.",
        cx,
    )
    .child(specimen(
        match count {
            0 => "not saved yet".to_string(),
            1 => "saved once".to_string(),
            count => format!("saved {count} times"),
        },
        div().w(px(480.0)).child(
            Form::new("profile-form")
                .on_submit(save)
                .child(
                    FormSection::new("Profile")
                        .description("How others see you.")
                        .child(
                            FormField::new("profile-name", "Name")
                                .required()
                                .child(probe("form-name", div().w(px(480.0)).child(Input::new(&name)))),
                        )
                        .child(
                            FormField::new("profile-email", "Email")
                                .required()
                                .help("Only for sign-in and receipts.")
                                .description("We never share it.")
                                .when_some(problem, |field, problem| field.error(problem))
                                .child(probe("form-email", div().w(px(480.0)).child(Input::new(&email)))),
                        )
                        .child(
                            FormField::new("profile-bio", "Bio")
                                .description("Plain text, a few lines.")
                                .child(Input::new(&bio)),
                        ),
                )
                .child(
                    FormSection::new("Links")
                        .description("Sites to show on your profile.")
                        .child(array),
                )
                .child(probe(
                    "form-actions",
                    div().w(px(480.0)).child(
                    ActionBar::new()
                        .child(DirtyIndicator::new("form-dirty", dirty))
                        .child(div().flex_1())
                        .child(
                            Button::new("form-discard", "Discard")
                                .variant(ButtonVariant::Ghost)
                                .disabled(!dirty)
                                .on_click(move |_, window, cx| discard(window, cx)),
                        )
                        .child(
                            Button::new("form-save", "Save")
                                .primary()
                                .disabled(!dirty || !ready(&now))
                                .on_click(move |_, window, cx| save_click(window, cx)),
                        ),
                    ),
                )
                ),
        ),
        cx,
    ))
}

pub fn inline(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let email = field("inline-email", window, cx, |input| {
        input.placeholder("you@example.com")
    });
    section(
        "InlineForm",
        "Fields and actions in one row, their bottoms aligned.",
        cx,
    )
    .child(
        div().w(px(480.0)).child(
            InlineForm::new()
                .child(div().flex_1().child(
                    FormField::new("inline-newsletter", "Newsletter").child(Input::new(&email)),
                ))
                .child(Button::new("inline-join", "Subscribe").primary()),
        ),
    )
}
