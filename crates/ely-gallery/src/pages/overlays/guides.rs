use ely_gpui_component::{
    buttons::{Button, ButtonVariant, IconButton},
    forms::{Input, TextInput},
    overlays::{FloatingToolbar, Lightbox, Peek, Slide, Spotlight, Tour, TourStep},
    primitives::{IconName, Image, Measure},
    theme::ActiveTheme,
};
use gpui::{
    App, Bounds, Entity, InteractiveElement, IntoElement, ObjectFit, ParentElement, Pixels,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use crate::{
    probe::probe,
    ui::{keep, picture, row, section, set, specimen},
};

const PHOTOS: [(&str, &str); 2] = [
    (asset!("dunes.jpg"), "Dunes at first light"),
    (asset!("atrium.jpg"), "An atrium, noon"),
];

pub fn lightbox(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let shown = keep("lightbox-at", || None::<usize>, window, cx);
    let thumbs = PHOTOS.iter().enumerate().map(|(ix, (path, _))| {
        let open = shown.clone();
        probe(
            ["lightbox-0", "lightbox-1"][ix],
            div()
                .id(("thumb", ix))
                .w_48()
                .h_32()
                .cursor_pointer()
                .child(
                    Image::new(("thumb-image", ix), picture(*path))
                        .fit(ObjectFit::Cover)
                        .size_full(),
                )
                .on_click(move |_, _, cx| set(&open, Some(ix), cx)),
        )
    });
    let overlay = (*shown.read(cx)).map(|at| {
        let (step, close) = (shown.clone(), shown.clone());
        Lightbox::new(
            "lightbox",
            PHOTOS.map(|(path, caption)| Slide::new(picture(path), caption)),
            at,
            move |_, cx| set(&close, None, cx),
        )
        .on_step(move |to, _, cx| set(&step, Some(to), cx))
    });
    section(
        "Lightbox",
        "A picture over a dark layer, one at a time. Arrows step, Escape or a press on the dark closes it, and each picture fades in.",
        cx,
    )
    .child(row().children(thumbs))
    .children(overlay)
}

/// Where each tour target sits, measured as it renders.
type Boxes = Entity<[Bounds<Pixels>; 3]>;

fn target(boxes: &Boxes, ix: usize, child: impl IntoElement) -> impl IntoElement {
    let boxes = boxes.clone();
    Measure::new(("tour-target", ix), move |bounds, _, cx| {
        let mut next = *boxes.read(cx);
        next[ix] = bounds;
        set(&boxes, next, cx);
    })
    .child(child)
}

pub fn tour(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let boxes: Boxes = keep("tour-boxes", || [Bounds::default(); 3], window, cx);
    let at = keep("tour-at", || None::<usize>, window, cx);
    let coach = keep("coach-open", || false, window, cx);
    let spots = *boxes.read(cx);
    let steps = [
        ("Search", "Find any page, person or file from here."),
        ("Invite", "Bring your team in. They see what you share."),
        ("Settings", "Theme, density and keys live here."),
    ];
    let overlay = (*at.read(cx)).map(|step| {
        let (next, close) = (at.clone(), at.clone());
        Tour::new(
            "tour",
            steps
                .iter()
                .zip(spots)
                .map(|((title, body), spot)| TourStep::new(spot, *title, *body)),
            step,
            move |_, cx| set(&close, None, cx),
        )
        .on_step(move |to, _, cx| set(&next, Some(to), cx))
        .into_any_element()
    });
    let coachmark = (*coach.read(cx)).then(|| {
        let close = coach.clone();
        Spotlight::new(
            "coach",
            TourStep::new(
                spots[1],
                "New: invites",
                "Invite by name or email; links expire in a week.",
            ),
            move |_, cx| set(&close, false, cx),
        )
        .into_any_element()
    });
    let (start, show) = (at.clone(), coach.clone());
    section(
        "Spotlight / Coachmark / Tour",
        "The window dims around one control and a card explains it. A tour walks the stops; the lit box glides from one to the next.",
        cx,
    )
    .child(
        row()
            .child(target(&boxes, 0, Button::new("tour-search", "Search").icon(IconName::Search)))
            .child(target(&boxes, 1, Button::new("tour-invite", "Invite").icon(IconName::Users)))
            .child(target(&boxes, 2, IconButton::new("tour-settings", IconName::Settings)))
            .child(div().w_8())
            .child(probe(
                "tour",
                Button::new("tour-start", "Take the tour")
                    .primary()
                    .on_click(move |_, _, cx| set(&start, Some(0), cx)),
            ))
            .child(probe(
                "coach",
                Button::new("coach-start", "Show a coachmark")
                    .variant(ButtonVariant::Ghost)
                    .on_click(move |_, _, cx| set(&show, true, cx)),
            )),
    )
    .children(overlay)
    .children(coachmark)
}

pub fn toolbar(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let field: Entity<TextInput> = window.use_keyed_state("toolbar-field", cx, |window, cx| {
        let mut input = TextInput::new(window, cx);
        input.set_text("Select some of these words to see the tools.", cx);
        input
    });
    let used = keep("toolbar-used", || None::<SharedString>, window, cx);
    let caption = used
        .read(cx)
        .clone()
        .map_or("select text in the field".to_string(), |tool| {
            format!("applied {tool}")
        });
    let tool = |name: &'static str, icon: IconName| {
        let used = used.clone();
        IconButton::new(SharedString::from(format!("tool-{name}")), icon)
            .tooltip(name)
            .on_click(move |_, _, cx| set(&used, Some(name.into()), cx))
    };
    section(
        "FloatingToolbar",
        "Tools float over a selection while the field has focus, and leave focus where it is.",
        cx,
    )
    .child(specimen(
        caption,
        probe(
            "toolbar",
            div().w_96().child(Input::new(&field)).child(
                FloatingToolbar::new("toolbar", &field)
                    .child(tool("bold", IconName::Bold))
                    .child(tool("italic", IconName::Italic))
                    .child(tool("link", IconName::Link)),
            ),
        ),
        cx,
    ))
}

pub fn peek(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = keep("peek-open", || false, window, cx);
    let shown = *open.read(cx);
    let (toggle, close) = (open.clone(), open.clone());
    let theme = cx.theme();
    section(
        "Callout / Peek",
        "A panel opens in the flow under its line, the way an editor peeks at a definition, and pushes what follows down.",
        cx,
    )
    .child(
        div()
            .w_128()
            .font_family(theme.mono_family.clone())
            .text_color(theme.colors.fg_muted)
            .child(
                div().flex().child("let colors = cx.").child(probe(
                    "peek",
                    Button::new("peek-theme", "theme()")
                        .variant(ButtonVariant::Link)
                        .on_click(move |_, _, cx| set(&toggle, !shown, cx)),
                )),
            )
            .child(
                Peek::new("peek-theme-def", "src/theme/mod.rs", shown, move |_, cx| set(&close, false, cx))
                    .child("pub trait ActiveTheme { fn theme(&self) -> &Theme; }"),
            )
            .child(div().child("let accent = colors.accent;")),
    )
}
