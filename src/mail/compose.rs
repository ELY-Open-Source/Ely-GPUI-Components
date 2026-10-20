use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, ElementId, Entity, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Subscription, Window, div, prelude::*,
};
use jiff::{Timestamp, tz::TimeZone};

use super::{reader::Contact, send::ScheduleSend, signature::signature_block};
use crate::{
    buttons::{Button, ButtonVariant, IconButton},
    chat::{Attachment, AttachmentChip},
    forms::{Choice, Input, InputEvent, OnValue, Run, TagInput, TextInput, is_email},
    i18n::text,
    primitives::IconName,
    theme::{ActiveTheme, ControlSize, TextSize},
};

/// The addresses and subject; the owner keeps the words.
#[derive(Clone, Debug, PartialEq)]
pub struct Draft {
    pub to: Vec<SharedString>,
    pub cc: Vec<SharedString>,
    pub bcc: Vec<SharedString>,
    pub subject: SharedString,
}

type OnDraft = Rc<dyn Fn(&Draft, &mut Window, &mut App)>;
type OnDraftAt = Rc<dyn Fn(&Draft, Timestamp, &mut Window, &mut App)>;

/// Someone to send to, and every address passes `check`.
fn ready(draft: &Draft, check: fn(&str) -> bool) -> bool {
    let mut all = draft
        .to
        .iter()
        .chain(&draft.cc)
        .chain(&draft.bcc)
        .peekable();
    all.peek().is_some() && all.all(|address| check(address))
}

/// The tags, then the typed words if new.
fn with_typed(tags: &[SharedString], typed: &SharedString) -> Vec<SharedString> {
    let typed = typed.trim().trim_end_matches(',').trim();
    let mut all = tags.to_vec();
    if !typed.is_empty() && !tags.iter().any(|tag| tag.as_ref() == typed) {
        all.push(typed.to_owned().into());
    }
    all
}

/// Which field of addresses.
#[derive(Clone, Copy)]
enum Field {
    To,
    Cc,
    Bcc,
}

/// The owner's addresses and subject a draft starts from.
type Seed = (
    Vec<SharedString>,
    Vec<SharedString>,
    Vec<SharedString>,
    SharedString,
);

/// The addresses, typed words, and subject being written.
struct Composing {
    seed: Seed,
    to: Vec<SharedString>,
    cc: Vec<SharedString>,
    bcc: Vec<SharedString>,
    /// Typed in To, Cc and Bcc, not yet added.
    typed: [SharedString; 3],
    copies: bool,
    subject: Entity<TextInput>,
    on_change: Option<OnDraft>,
    _subject: Subscription,
}

impl Composing {
    /// Starts over from the owner's addresses and subject.
    fn fill(&mut self, (to, cc, bcc, subject): Seed, cx: &mut Context<Self>) {
        self.to = to;
        self.copies = !cc.is_empty() || !bcc.is_empty();
        self.cc = cc;
        self.bcc = bcc;
        self.typed = Default::default();
        self.subject
            .update(cx, |input, cx| input.set_text(subject.to_string(), cx));
    }

    fn draft(&self, cx: &App) -> Draft {
        Draft {
            to: with_typed(&self.to, &self.typed[0]),
            cc: with_typed(&self.cc, &self.typed[1]),
            bcc: with_typed(&self.bcc, &self.typed[2]),
            subject: self.subject.read(cx).text().trim().to_string().into(),
        }
    }

    /// Tells the owner the draft as it now stands.
    fn changed(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.draft(cx), window, cx);
        }
        cx.notify();
    }
}

/// Addresses, subject, the owner's editor, files, signature, Send.
#[derive(IntoElement)]
pub struct MailComposer {
    id: ElementId,
    body: AnyElement,
    contacts: Vec<Contact>,
    to: Vec<SharedString>,
    cc: Vec<SharedString>,
    bcc: Vec<SharedString>,
    subject: SharedString,
    busy: bool,
    check: fn(&str) -> bool,
    signature: Option<SharedString>,
    files: Vec<Attachment>,
    zone: Option<TimeZone>,
    on_change: Option<OnDraft>,
    on_send: Option<OnDraft>,
    on_schedule: Option<OnDraftAt>,
    on_discard: Option<Run>,
    on_attach: Option<Run>,
    on_remove: Option<OnValue>,
}

impl MailComposer {
    /// `body` is the owner's editor for the words.
    pub fn new(id: impl Into<ElementId>, body: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            body: body.into_any_element(),
            contacts: Vec::new(),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: SharedString::default(),
            busy: false,
            check: is_email,
            signature: None,
            files: Vec::new(),
            zone: None,
            on_change: None,
            on_send: None,
            on_schedule: None,
            on_discard: None,
            on_attach: None,
            on_remove: None,
        }
    }

    /// People to suggest as addresses are typed.
    pub fn contacts(mut self, contacts: impl IntoIterator<Item = Contact>) -> Self {
        self.contacts = contacts.into_iter().collect();
        self
    }

    /// The addresses it starts with, as a reply has them.
    pub fn to(mut self, addresses: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.to = addresses.into_iter().map(Into::into).collect();
        self
    }

    /// The copies it starts with; Cc and Bcc then show.
    pub fn cc(mut self, addresses: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.cc = addresses.into_iter().map(Into::into).collect();
        self
    }

    /// The blind copies it starts with; both show.
    pub fn bcc(mut self, addresses: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.bcc = addresses.into_iter().map(Into::into).collect();
        self
    }

    /// The subject it starts with.
    pub fn subject(mut self, subject: impl Into<SharedString>) -> Self {
        self.subject = subject.into();
        self
    }

    /// Something is under way; Send waits.
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    /// What counts as an address; `is_email` otherwise.
    pub fn check(mut self, check: fn(&str) -> bool) -> Self {
        self.check = check;
        self
    }

    /// Gets the draft after each edit.
    pub fn on_change(mut self, handler: impl Fn(&Draft, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn signature(mut self, text: impl Into<SharedString>) -> Self {
        self.signature = Some(text.into());
        self
    }

    /// Files the owner attached.
    pub fn files(mut self, files: impl IntoIterator<Item = Attachment>) -> Self {
        self.files = files.into_iter().collect();
        self
    }

    /// Send times read in this zone; else the system's.
    pub fn zone(mut self, zone: TimeZone) -> Self {
        self.zone = Some(zone);
        self
    }

    pub fn on_send(mut self, handler: impl Fn(&Draft, &mut Window, &mut App) + 'static) -> Self {
        self.on_send = Some(Rc::new(handler));
        self
    }

    /// Gets the draft and the moment to send it.
    pub fn on_schedule(
        mut self,
        handler: impl Fn(&Draft, Timestamp, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_schedule = Some(Rc::new(handler));
        self
    }

    pub fn on_discard(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_discard = Some(Rc::new(handler));
        self
    }

    /// Asks the owner for files to attach.
    pub fn on_attach(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_attach = Some(Rc::new(handler));
        self
    }

    /// Gets the key of the file to take off.
    pub fn on_remove(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_remove = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for MailComposer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let seed = (
            self.to.clone(),
            self.cc.clone(),
            self.bcc.clone(),
            self.subject.clone(),
        );
        let state = window.use_keyed_state(
            (self.id.clone(), "draft"),
            cx,
            |window, cx: &mut Context<Composing>| {
                let placeholder = text(cx, "mail.compose.subject", &[]);
                let subject = cx.new(|cx| TextInput::new(window, cx).placeholder(placeholder));
                let typing =
                    cx.subscribe_in(&subject, window, |composing, _, event, window, cx| {
                        if *event == InputEvent::Changed {
                            composing.changed(window, cx);
                        }
                    });
                let mut composing = Composing {
                    seed: seed.clone(),
                    to: Vec::new(),
                    cc: Vec::new(),
                    bcc: Vec::new(),
                    typed: Default::default(),
                    copies: false,
                    subject,
                    on_change: None,
                    _subject: typing,
                };
                composing.fill(seed.clone(), cx);
                composing
            },
        );
        let on_change = self.on_change.clone();
        state.update(cx, |composing, _| composing.on_change = on_change);
        if state.read(cx).seed != seed {
            log::info!("mail composer {}: started over", self.id);
            state.update(cx, |composing, cx| {
                composing.seed = seed.clone();
                composing.fill(seed, cx);
            });
        }
        let now = state.read(cx);
        let (to, cc, bcc, copies) = (now.to.clone(), now.cc.clone(), now.bcc.clone(), now.copies);
        let subject = now.subject.clone();
        let check = self.check;
        let can_send = ready(&now.draft(cx), check) && !self.busy;
        let people = text(cx, "mail.compose.people", &[]);
        let choices: Vec<Choice> = self
            .contacts
            .iter()
            .map(|contact| Choice::new(contact.address.clone(), contact.name.clone()))
            .collect();
        let theme = cx.theme();
        let colors = &theme.colors;
        let row = |label: &'static str, field: Field, tags: Vec<SharedString>| {
            let (owner, typist) = (state.clone(), state.clone());
            let shown = match field {
                Field::To => text(cx, "mail.compose.to", &[]),
                Field::Cc => text(cx, "mail.compose.cc", &[]),
                Field::Bcc => text(cx, "mail.compose.bcc", &[]),
            };
            div()
                .debug_selector(move || format!("composer-{label}"))
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex_none()
                        .w_10()
                        .text_size(theme.text_size(TextSize::Sm))
                        .text_color(colors.fg_muted)
                        .child(shown),
                )
                .child(
                    div().flex_1().min_w_0().child(
                        TagInput::new((self.id.clone(), label), tags)
                            .placeholder(people.clone())
                            .check(check)
                            .suggestions(choices.clone())
                            .on_typing(move |typed, window, cx| {
                                typist.update(cx, |composing, cx| {
                                    composing.typed[field as usize] = typed;
                                    composing.changed(window, cx);
                                })
                            })
                            .on_change(move |next, window, cx| {
                                log::info!("mail composer: {label} holds {}", next.len());
                                owner.update(cx, |composing, cx| {
                                    match field {
                                        Field::To => composing.to = next,
                                        Field::Cc => composing.cc = next,
                                        Field::Bcc => composing.bcc = next,
                                    }
                                    composing.changed(window, cx);
                                })
                            }),
                    ),
                )
        };
        let showing = state.clone();
        let copies_button = (!copies).then(|| {
            Button::new(
                (self.id.clone(), "copies"),
                text(cx, "mail.compose.copies", &[]),
            )
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Sm)
            .on_click(move |_, _, cx| {
                log::info!("mail composer: shows Cc and Bcc");
                showing.update(cx, |composing, cx| {
                    composing.copies = true;
                    cx.notify();
                })
            })
        });
        let files = self.files.iter().map(|file| {
            let chip = AttachmentChip::new(
                (self.id.clone(), format!("file-{}", file.key)),
                file.clone(),
            );
            match self.on_remove.clone() {
                Some(remove) => {
                    let key = file.key.clone();
                    chip.on_remove(move |window, cx| {
                        log::info!("mail composer: takes off {key}");
                        remove(&key, window, cx)
                    })
                    .into_any_element()
                }
                None => chip.into_any_element(),
            }
        });
        let sending = state.clone();
        let send = ScheduleSend::new((self.id.clone(), "send"))
            .disabled(!can_send)
            .when_some(self.zone.clone(), |send, zone| send.zone(zone))
            .when_some(self.on_send.clone(), |send, on_send| {
                send.on_send(move |window, cx| {
                    let draft = sending.read(cx).draft(cx);
                    log::info!("mail composer: sends to {}", draft.to.len());
                    on_send(&draft, window, cx)
                })
            });
        let scheduling = state.clone();
        let send = match self.on_schedule.clone() {
            Some(on_schedule) => send.on_schedule(move |at, window, cx| {
                let draft = scheduling.read(cx).draft(cx);
                log::info!("mail composer: sends at {at}");
                on_schedule(&draft, at, window, cx)
            }),
            None => send,
        };
        let attach = self.on_attach.map(|attach| {
            IconButton::new((self.id.clone(), "attach"), IconName::Paperclip)
                .variant(ButtonVariant::Ghost)
                .tooltip(text(cx, "mail.compose.attach", &[]))
                .on_click(move |_, window, cx| {
                    log::info!("mail composer: attach");
                    attach(window, cx)
                })
        });
        let discard = self.on_discard.map(|discard| {
            IconButton::new((self.id.clone(), "discard"), IconName::Trash2)
                .variant(ButtonVariant::Ghost)
                .tooltip(text(cx, "mail.compose.discard", &[]))
                .on_click(move |_, window, cx| {
                    log::info!("mail composer: discard");
                    discard(window, cx)
                })
        });
        div()
            .debug_selector(|| "mail-composer".into())
            .flex()
            .flex_col()
            .gap_3()
            .child(
                row("to", Field::To, to)
                    .children(copies_button.map(|button| div().flex_none().child(button))),
            )
            .when(copies, |form| {
                form.child(row("cc", Field::Cc, cc))
                    .child(row("bcc", Field::Bcc, bcc))
            })
            .child(Input::new(&subject))
            .child(self.body)
            .when(!self.files.is_empty(), |form| {
                form.child(div().flex().flex_wrap().gap_2().children(files))
            })
            .children(self.signature.map(|text| signature_block(&text, cx)))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(send)
                            .children(attach),
                    )
                    .children(discard),
            )
    }
}

#[cfg(test)]
mod tests;
