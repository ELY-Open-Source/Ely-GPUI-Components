use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    motion::ProgressRing,
    theme::ControlSize,
    tooling::{Knob, Playground},
};
use gpui::{AnyElement, App, IntoElement, ParentElement, Styled, Window, div, px};

mod catalogs;
mod inspection;

use super::Page;
use crate::probe::probe;
use crate::step::Step;
use crate::ui::section;

pub const PAGE: Page = Page {
    number: 43,
    slug: "tooling",
    title: "Library Tooling",
    summary: "Tools for building with the library: this gallery, a playground that edits a component live, tools that inspect a running window, its boxes, frames, renders and input, and catalogs of its tokens, icons and contrast.",
    render,
    script: &[
        Step::DownAt("button-playground", 472.0, 207.0),
        Step::UpAt("button-playground", 472.0, 207.0),
        Step::Shot("playground-loading"),
        Step::DownAt("inspect-button", 12.0, 12.0),
        Step::UpAt("inspect-button", 12.0, 12.0),
        Step::HoverAt("inspect-sample", 60.0, 30.0),
        Step::Shot("inspector-picking"),
        Step::DownAt("inspect-sample", 60.0, 30.0),
        Step::UpAt("inspect-sample", 60.0, 30.0),
        Step::HoverAt("inspect-button", 12.0, 12.0),
        Step::Shot("inspector-held"),
        Step::DownAt("inspect-button", 12.0, 12.0),
        Step::UpAt("inspect-button", 12.0, 12.0),
        Step::DownAt("fps-switch", 14.0, 10.0),
        Step::UpAt("fps-switch", 14.0, 10.0),
        Step::Wait(1200),
        Step::Shot("fps-measuring"),
        Step::DownAt("fps-switch", 14.0, 10.0),
        Step::UpAt("fps-switch", 14.0, 10.0),
        Step::DownAt("renders-apart", 200.0, 18.0),
        Step::UpAt("renders-apart", 200.0, 18.0),
        Step::DownAt("renders-apart", 200.0, 18.0),
        Step::UpAt("renders-apart", 200.0, 18.0),
        Step::Shot("renders"),
        Step::HoverAt("events", 40.0, 20.0),
        Step::HoverAt("events", 120.0, 30.0),
        Step::DownAt("events", 120.0, 30.0),
        Step::UpAt("events", 120.0, 30.0),
        Step::DownAt("events", 60.0, 60.0),
        Step::UpAt("events", 60.0, 60.0),
        Step::Type("Hi"),
        Step::Key("backspace"),
        Step::Shot("events"),
        Step::DownAt("icon-browser", 100.0, 14.0),
        Step::UpAt("icon-browser", 100.0, 14.0),
        Step::Type("sun"),
        Step::DownAt("icon-browser", 16.0, 52.0),
        Step::UpAt("icon-browser", 16.0, 52.0),
        Step::Shot("icon-browser"),
    ],
};

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    let inspection = inspection::sections(window, cx);
    let catalogs = catalogs::sections(window, cx);
    div()
        .child(section(
            "ComponentGallery / Storybook",
            "This app is the gallery: a page per chapter of the library, light and dark, each with scripted states. The --page flag opens one page, and --capture photographs every page and state.",
            cx,
        ))
        .child(
            section(
                "PlaygroundPanel",
                "A component beside the knobs that shape it: a switch, a choice or a number edits one property, and the preview redraws.",
                cx,
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(probe(
                        "button-playground",
                        div().w(px(720.0)).child(Playground::new(
                        "button-playground",
                        [
                            Knob::Choice(
                                "variant".into(),
                                ["Primary", "Secondary", "Outline", "Ghost", "Danger"]
                                    .map(Into::into)
                                    .into(),
                                0,
                            ),
                            Knob::Choice("size".into(), ["Sm", "Md", "Lg"].map(Into::into).into(), 1),
                            Knob::Toggle("disabled".into(), false),
                            Knob::Toggle("loading".into(), false),
                        ],
                        |settings, _, _| {
                            let variant = match settings.picked("variant").as_ref() {
                                "Primary" => ButtonVariant::Primary,
                                "Secondary" => ButtonVariant::Secondary,
                                "Outline" => ButtonVariant::Outline,
                                "Ghost" => ButtonVariant::Ghost,
                                "Danger" => ButtonVariant::Danger,
                                other => panic!("gallery playground: no variant {other}"),
                            };
                            let size = match settings.picked("size").as_ref() {
                                "Sm" => ControlSize::Sm,
                                "Md" => ControlSize::Md,
                                "Lg" => ControlSize::Lg,
                                other => panic!("gallery playground: no size {other}"),
                            };
                            Button::new("played", "Save changes")
                                .variant(variant)
                                .size(size)
                                .disabled(settings.on("disabled"))
                                .loading(settings.on("loading"))
                                .into_any_element()
                        },
                    ))))
                    .child(div().w(px(720.0)).child(Playground::new(
                        "ring-playground",
                        [
                            Knob::Number("value".into(), 0.62, (0.0, 1.0), 0.01),
                            Knob::Toggle("percent".into(), true),
                        ],
                        |settings, _, _| {
                            let ring = ProgressRing::new("played-ring", settings.number("value") as f32);
                            match settings.on("percent") {
                                true => ring.percent(),
                                false => ring,
                            }
                            .into_any_element()
                        },
                    ))),
            ),
        )
        .children(inspection)
        .children(catalogs)
        .into_any_element()
}
