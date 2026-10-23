use std::cell::RefCell;

use ely_gpui_component::theme::{ActiveTheme, TextSize};
use gpui::{
    AnyElement, App, Div, Entity, FontWeight, Global, ImageSource, IntoElement, ParentElement,
    Resource, SharedString, Styled, Window, div, prelude::*,
};

/// The one section a story draws while its page renders.
pub struct Story {
    wanted: SharedString,
    met: RefCell<Vec<SharedString>>,
    live: RefCell<Vec<SharedString>>,
}

impl Global for Story {}

impl Story {
    pub fn new(wanted: SharedString) -> Self {
        Self {
            wanted,
            met: RefCell::default(),
            live: RefCell::default(),
        }
    }

    fn answers(&self, title: &str) -> bool {
        *self.wanted == *title || *self.wanted == slug(title)
    }

    /// Why no single section answered this story, if none did.
    pub fn missing(&self, page: &str) -> Option<String> {
        let met = self.met.borrow();
        if let Some(stray) = self.live.borrow().iter().find(|title| !met.contains(title)) {
            return Some(format!("ui::live names {stray}, no section on {page}."));
        }
        match met.iter().filter(|title| self.answers(title)).count() {
            1 => None,
            0 => Some(format!(
                "The page {page} has no story {}. Its stories: {}.",
                self.wanted,
                met.join(", ")
            )),
            n => Some(format!("{n} stories on {page} answer to {}.", self.wanted)),
        }
    }
}

/// Whether a demo's clock runs: on its page or story.
pub fn live(title: &str, cx: &App) -> bool {
    cx.try_global::<Story>().is_none_or(|story| {
        story.live.borrow_mut().push(title.to_string().into());
        story.answers(title)
    })
}

/// A title as a story's address: lowercase, dashes between.
pub fn slug(title: &str) -> String {
    let mut slug = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_string()
}

/// Titled block of demos; a story draws one, untitled.
pub fn section(title: impl Into<SharedString>, note: impl Into<SharedString>, cx: &App) -> Section {
    let title = title.into();
    let theme = cx.theme();
    let story = cx.try_global::<Story>();
    let shown = story.is_none_or(|story| {
        story.met.borrow_mut().push(title.clone());
        story.answers(&title)
    });
    Section {
        div: div()
            .flex()
            .flex_col()
            .gap_4()
            .when(story.is_none(), |section| section.pt_10())
            .when(!shown, |section| section.hidden()),
        shown,
    }
    .child(
        div()
            .flex()
            .flex_col()
            .gap_1()
            .when(story.is_none(), |header| {
                header.child(
                    div()
                        .text_size(theme.text_size(TextSize::Md))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.colors.fg)
                        .child(title),
                )
            })
            .child(
                div()
                    .text_size(theme.text_size(TextSize::Sm))
                    .text_color(theme.colors.fg_muted)
                    .child(note.into()),
            ),
    )
}

/// A section's box; one a story passes drops its demos.
pub struct Section {
    div: Div,
    shown: bool,
}

impl ParentElement for Section {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        if self.shown {
            self.div.extend(elements);
        }
    }
}

impl IntoElement for Section {
    type Element = Div;

    fn into_element(self) -> Div {
        self.div
    }
}

/// A gallery picture: a file, or a web address.
pub fn resource(path: impl Into<SharedString>) -> Resource {
    let path = path.into();
    if cfg!(target_family = "wasm") {
        Resource::Uri(path.into())
    } else {
        Resource::Path(std::path::Path::new(path.as_ref()).into())
    }
}

pub fn picture(path: impl Into<SharedString>) -> ImageSource {
    ImageSource::Resource(resource(path))
}

pub fn row() -> Div {
    div().flex().flex_wrap().items_center().gap_3()
}

/// Row whose captions share one line.
pub fn specimens() -> Div {
    div().flex().flex_wrap().items_end().gap_6()
}

pub fn label(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    div()
        .text_size(theme.text_size(TextSize::Xs))
        .text_color(theme.colors.fg_subtle)
        .child(text.into())
}

/// Demo with a caption beneath.
pub fn specimen(caption: impl Into<SharedString>, body: impl IntoElement, cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(body)
        .child(label(caption, cx))
}

/// Monospace hint naming the call.
pub fn code(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    div()
        .font_family(theme.mono_family.clone())
        .text_size(theme.text_size(TextSize::Xs))
        .text_color(theme.colors.fg_muted)
        .child(text.into())
}

/// Honest note for an entry gpui cannot support.
pub fn blocked(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    div()
        .flex()
        .items_start()
        .gap_2()
        .text_size(theme.text_size(TextSize::Sm))
        .text_color(theme.colors.warning)
        .child(
            div()
                .flex_none()
                .mt_1p5()
                .size_1p5()
                .rounded_full()
                .bg(theme.colors.warning),
        )
        .child(div().flex_1().min_w_0().child(text.into()))
}

/// Why no screen reader reads a demo here.
pub const UNREAD: &str = if cfg!(target_family = "wasm") {
    "gpui builds no AccessKit tree on the web, so no screen reader reads it here"
} else {
    "gpui builds an AccessKit tree from elements with an id and a role, and no screen reader here reads it"
};

/// A demo browsers cannot run draws natively alone; the site shows its capture.
pub fn native_only(element: impl IntoElement) -> AnyElement {
    if cfg!(target_family = "wasm") {
        div().into_any_element()
    } else {
        element.into_any_element()
    }
}

/// A demo's state, kept across frames under `key`.
pub fn keep<T: 'static>(
    key: &'static str,
    init: impl FnOnce() -> T,
    window: &mut Window,
    cx: &mut App,
) -> Entity<T> {
    window.use_keyed_state(key, cx, move |_, _| init())
}

/// Edits a demo's state in place and redraws.
pub fn change<T: 'static>(state: &Entity<T>, cx: &mut App, edit: impl FnOnce(&mut T)) {
    state.update(cx, |state, cx| {
        edit(state);
        cx.notify();
    });
}

/// Replaces a demo's state and redraws.
pub fn set<T: 'static>(state: &Entity<T>, value: T, cx: &mut App) {
    state.update(cx, |state, cx| {
        *state = value;
        cx.notify();
    });
}

/// A steady pseudo-random series, so captures repeat.
pub fn noise(seed: u64) -> impl FnMut() -> f64 {
    let mut state = seed;
    move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 33) as f64 / (1u64 << 31) as f64
    }
}
