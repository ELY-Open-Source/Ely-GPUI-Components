use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    forms::{Switch, TextInput},
    theme::{ActiveTheme, Radius},
    tooling::{EventLogger, FpsMeter, RenderCounter},
};
use gpui::{
    AnyElement, AnyView, App, Context, IntoElement, ParentElement, Render, StyleRefinement, Styled,
    Window, div, px,
};

use crate::probe::probe;
use crate::ui::section;

/// A view of its own, drawn cached, so only its own notify renders it again.
struct Apart;

impl Render for Apart {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_3()
            .child(RenderCounter::new("apart-renders", "This view"))
            .child(
                Button::new("apart-redraw", "Redraw this view")
                    .variant(ButtonVariant::Outline)
                    .on_click(cx.listener(|_, _, _, cx| cx.notify())),
            )
    }
}

#[cfg(debug_assertions)]
fn inspect() -> AnyElement {
    probe(
        "inspect-button",
        Button::new("inspect", "Inspect").on_click(|_, window, cx| window.toggle_inspector(cx)),
    )
    .into_any_element()
}

#[cfg(not(debug_assertions))]
fn inspect() -> AnyElement {
    div()
        .child("Release builds leave gpui's inspector out.")
        .into_any_element()
}

pub fn sections(window: &mut Window, cx: &mut App) -> Vec<AnyElement> {
    let measuring = window.use_keyed_state("fps-measuring", cx, |_, _| false);
    let on = *measuring.read(cx);
    let apart = window.use_keyed_state("renders-apart", cx, |_, _| Apart);
    let field = window.use_keyed_state("events-field", cx, TextInput::new);
    let theme = cx.theme();
    let colors = &theme.colors;
    let boxed = |element: gpui::Div| {
        element
            .border_2()
            .border_color(colors.border)
            .rounded(theme.radius(Radius::Md))
            .bg(colors.surface)
    };
    vec![
        section(
            "InspectorOverlay",
            "gpui's inspector, which debug builds have, drawn in Ely's look. Inspect opens it at the window's right: hovering picks a box and shows its margin, border, padding and size over the page, the wheel steps out to the boxes around it, and a press holds it.",
            cx,
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_4()
                .child(inspect())
                .child(probe(
                    "inspect-sample",
                    boxed(div().m_3().p_4()).child("A box to pick"),
                )),
        )
        .into_any_element(),
        section(
            "FPSMeter / PerfOverlay",
            "Frames per second over the last second, the worst frame, and a bar per frame against the 60 Hz budget: green on time, amber a frame late, red later. It keeps the window drawing while shown, so Measure shows it only while you look.",
            cx,
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_4()
                .child(probe(
                    "fps-switch",
                    Switch::new("fps-measure", on)
                        .label("Measure")
                        .on_change(move |on, _, cx| {
                            measuring.update(cx, |measuring, cx| {
                                *measuring = on;
                                cx.notify();
                            })
                        }),
                ))
                .children(on.then(|| FpsMeter::new("gallery-fps"))),
        )
        .into_any_element(),
        section(
            "RenderCounter",
            "How often the view that draws it renders. The page's count climbs with every frame the window draws; the view beside it is drawn cached, so it renders only when it redraws itself or the window refreshes.",
            cx,
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_4()
                .child(RenderCounter::new("page-renders", "Page"))
                .child(probe(
                    "renders-apart",
                    div().w(px(320.0)).h(px(36.0)).child(
                        AnyView::from(apart)
                            .cached(StyleRefinement::default().w(px(320.0)).h(px(36.0))),
                    ),
                )),
        )
        .into_any_element(),
        section(
            "EventLogger",
            "The input events that reach what it holds, newest first: presses, releases, moves run together while the pointer travels, wheels, keys and modifiers, each timed from the first.",
            cx,
        )
        .child(probe(
            "events",
            div().w(px(420.0)).child(
                EventLogger::new("gallery-events").child(
                    boxed(div().flex().flex_col().gap_2().p_3())
                        .child("Move, press or scroll here, or type in the field.")
                        .child(field),
                ),
            ),
        ))
        .into_any_element(),
    ]
}
