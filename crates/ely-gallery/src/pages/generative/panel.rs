use std::{rc::Rc, time::Duration};

use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    forms::{Form, FormField, TagInput, TextInput},
    generative::{AspectRatioPicker, PromptEnhancer, SeedInput, StylePreset, StylePresetPicker},
    typography::Caption,
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

type Run = Rc<dyn Fn(&mut Window, &mut App)>;

/// What an enhancer might give back; the demo adds light, material and lens.
fn fuller(prompt: &str) -> String {
    format!(
        "{prompt}, pale limestone, soft morning light through tall windows, long shadows, 35mm, quiet and airy"
    )
}

fn presets() -> Vec<StylePreset> {
    let preset = |key: &str, name: &str, picture: Option<&str>| StylePreset {
        key: key.to_string().into(),
        name: name.to_string().into(),
        picture: picture.map(|path| path.to_string().into()),
    };
    vec![
        preset("none", "None", None),
        preset(
            "photo",
            "Photographic",
            Some(asset!("styles/photographic.jpg")),
        ),
        preset("mono", "Monochrome", Some(asset!("styles/monochrome.jpg"))),
        preset("line", "Line art", Some(asset!("styles/line-art.jpg"))),
        preset("wash", "Watercolor", Some(asset!("styles/watercolor.jpg"))),
        preset("film", "Cinematic", Some(asset!("styles/cinematic.jpg"))),
    ]
}

/// The demo's settings: a suggestion on its way or offered, the left-out words, style, shape, and the last run.
#[derive(Clone)]
struct Settings {
    enhancing: bool,
    suggestion: Option<SharedString>,
    left_out: Vec<SharedString>,
    style: SharedString,
    ratio: (u32, u32),
    ran: Option<SharedString>,
}

fn edit(settings: &Entity<Settings>, cx: &mut App, change: impl FnOnce(&mut Settings)) {
    let mut next = settings.read(cx).clone();
    change(&mut next);
    set(settings, next, cx)
}

pub fn panel(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let prompt = window.use_keyed_state("gen-prompt-field", cx, |window, cx| {
        let mut input = TextInput::new(window, cx)
            .multi_line(3, 6)
            .placeholder("Describe a picture");
        input.set_text("a spiral staircase in a white atrium", cx);
        input
    });
    let seed = window.use_keyed_state("gen-seed-field", cx, |window, cx| {
        let mut input = TextInput::new(window, cx);
        input.set_text("2710", cx);
        input
    });
    let settings = keep(
        "gen-settings",
        || Settings {
            enhancing: false,
            suggestion: None,
            left_out: vec!["blurry".into(), "watermark".into(), "people".into()],
            style: "photo".into(),
            ratio: (3, 2),
            ran: None,
        },
        window,
        cx,
    );
    let now = settings.read(cx).clone();
    let run: Run = {
        let (settings, seed, prompt) = (settings.clone(), seed.clone(), prompt.clone());
        Rc::new(move |_, cx| {
            let seed = SeedInput::read(&seed, cx)
                .map_or("a new seed".to_string(), |seed| format!("seed {seed}"));
            let words = prompt.read(cx).text().split_whitespace().count();
            let line = format!("Queued: {words} words, {seed}");
            edit(&settings, cx, |settings| settings.ran = Some(line.into()));
        })
    };
    let (asked, resolved, tagged, styled, shaped) = (
        settings.clone(),
        settings.clone(),
        settings.clone(),
        settings.clone(),
        settings,
    );
    let enhancer = PromptEnhancer::new("gen-prompt", &prompt)
        .enhancing(now.enhancing)
        .on_enhance(move |text, _, cx| {
            edit(&asked, cx, |settings| settings.enhancing = true);
            let (asked, text) = (asked.clone(), fuller(text));
            cx.spawn(async move |cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(900))
                    .await;
                cx.update(|cx| {
                    edit(&asked, cx, |settings| {
                        settings.enhancing = false;
                        settings.suggestion = Some(text.into());
                    })
                });
            })
            .detach();
        })
        .on_resolve(move |_, _, cx| edit(&resolved, cx, |settings| settings.suggestion = None));
    let enhancer = match now.suggestion {
        Some(text) => enhancer.suggestion(text),
        None => enhancer,
    };
    let submit = run.clone();
    section(
        "ImageGenerationPanel / PromptEnhancer / NegativePromptInput / StylePresetPicker / AspectRatioPicker / SeedInput",
        "A picture's settings in one form: the prompt, which Enhance offers to fill out, the words it adds washed; what to leave out; a style; a shape; and a seed, empty for a new one each run. Cmd-Enter generates.",
        cx,
    )
    .child(
        div().w(px(400.)).child(
            Form::new("gen-form")
                .on_submit(move |window, cx| submit(window, cx))
                .child(probe(
                    "gen-enhancer",
                    div().w(px(400.)).child(FormField::new("gen-prompt-label", "Prompt").child(enhancer)),
                ))
                .child(
                    FormField::new("gen-left-out", "Leave out").child(
                        TagInput::new("gen-left-out-tags", now.left_out)
                            .placeholder("A word, then Enter")
                            .on_change(move |tags, _, cx| edit(&tagged, cx, |settings| settings.left_out = tags)),
                    ),
                )
                .child(
                    FormField::new("gen-style", "Style").child(
                        StylePresetPicker::new("gen-styles", presets(), now.style)
                            .on_select(move |key, _, cx| {
                                let key = key.clone();
                                edit(&styled, cx, |settings| settings.style = key)
                            }),
                    ),
                )
                .child(
                    FormField::new("gen-shape", "Shape").child(
                        AspectRatioPicker::new("gen-ratios", [(1, 1), (4, 5), (3, 2), (16, 9), (9, 16)], now.ratio)
                            .on_select(move |ratio, _, cx| edit(&shaped, cx, |settings| settings.ratio = ratio)),
                    ),
                )
                .child(FormField::new("gen-seed", "Seed").child(SeedInput::new("gen-seed-input", &seed)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            Button::new("gen-generate", "Generate")
                                .variant(ButtonVariant::Primary)
                                .full_width()
                                .shortcut("secondary-enter")
                                .on_click(move |_, window, cx| run(window, cx)),
                        )
                        .children(now.ran.map(Caption::new)),
                ),
        ),
    )
}
