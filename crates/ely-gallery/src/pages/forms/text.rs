use ely_gpui_component::{
    buttons::IconButton,
    forms::{Input, InputAddon, InputGroup, PasswordInput, SearchInput, TextInput},
    primitives::IconName,
    theme::ControlSize,
    typography::Caption,
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{code, section, specimen, specimens},
};

pub fn field(
    key: &'static str,
    window: &mut Window,
    cx: &mut App,
    build: impl FnOnce(TextInput) -> TextInput,
) -> Entity<TextInput> {
    window.use_keyed_state(key, cx, move |window, cx| build(TextInput::new(window, cx)))
}

pub fn input(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let name = field("input-name", window, cx, |input| {
        input.placeholder("Full name")
    });
    let small = field("input-small", window, cx, |input| {
        input.placeholder("Small")
    });
    let large = field("input-large", window, cx, |input| {
        input.placeholder("Large")
    });
    let clear = field("input-clear", window, cx, |input| {
        input.placeholder("Clearable")
    });
    section(
        "Input / TextField / ClearableInput",
        "One line. Undo, word jumps, IME and the clipboard come built in.",
        cx,
    )
    .child(probe(
        "input-name",
        div().w(px(320.0)).child(Input::new(&name)),
    ))
    .child(
        specimens()
            .child(specimen(
                "small",
                div()
                    .w(px(200.0))
                    .child(Input::new(&small).size(ControlSize::Sm)),
                cx,
            ))
            .child(specimen(
                "large",
                div()
                    .w(px(240.0))
                    .child(Input::new(&large).size(ControlSize::Lg)),
                cx,
            ))
            .child(specimen(
                "clearable",
                div().w(px(240.0)).child(Input::new(&clear).clearable()),
                cx,
            )),
    )
}

pub fn text_area(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let notes = field("textarea-notes", window, cx, |input| {
        input
            .multi_line(3, 6)
            .placeholder("Notes grow from three lines to six, then scroll.")
    });
    section(
        "TextArea / AutoResizeInput",
        "Several lines. It grows to its content, then scrolls.",
        cx,
    )
    .child(probe(
        "textarea",
        div().w(px(420.0)).child(Input::new(&notes)),
    ))
    .child(code("TextInput::new(window, cx).multi_line(3, 6)", cx))
}

pub fn password(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let secret = field("input-password", window, cx, |input| {
        input.masked().placeholder("Password")
    });
    section(
        "PasswordInput",
        "Bullets by default; the eye shows the text. Masked text never reaches the clipboard.",
        cx,
    )
    .child(probe(
        "password",
        div().w(px(320.0)).child(PasswordInput::new(&secret)),
    ))
}

pub fn search(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let query = field("input-search", window, cx, |input| {
        input.placeholder("Search docs")
    });
    let picked = window.use_keyed_state("search-picked", cx, |_, _| {
        SharedString::from("nothing yet")
    });
    let note = picked.read(cx).clone();
    section(
        "SearchInput",
        "A magnifier, a clear button, and recent searches while it is empty.",
        cx,
    )
    .child(probe(
        "search",
        div().w(px(320.0)).child(
            SearchInput::new("docs-search", &query)
                .history(["dock layout", "sticky header", "spring easing"])
                .on_pick(move |query, _, cx| {
                    let query = query.clone();
                    picked.update(cx, |picked, cx| {
                        *picked = query;
                        cx.notify();
                    })
                }),
        ),
    ))
    .child(Caption::new(format!("Last recent search: {note}")))
}

pub fn group(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let site = field("input-site", window, cx, |input| {
        input.placeholder("ely.dev")
    });
    let share = field("input-share", window, cx, |input| input.placeholder("0"));
    section(
        "InputGroup / InputAddon",
        "Addons join the field's frame: a scheme before, a unit or a control after.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "text addons",
                div().w(px(320.0)).child(
                    InputGroup::new(&site)
                        .before(InputAddon::text("https://"))
                        .after(InputAddon::text(".com")),
                ),
                cx,
            ))
            .child(specimen(
                "control addon",
                div().w(px(200.0)).child(
                    InputGroup::new(&share).after(InputAddon::new(
                        IconButton::new("group-copy", IconName::Copy)
                            .size(ControlSize::Sm)
                            .tooltip("Copy"),
                    )),
                ),
                cx,
            )),
    )
}
