use ely_gpui_component::forms::{NumberInput, ScrubInput, UnitInput};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{code, section, specimen, specimens},
};

fn number(key: &'static str, initial: f64, window: &mut Window, cx: &mut App) -> Entity<f64> {
    window.use_keyed_state(key, cx, move |_, _| initial)
}

fn store(state: Entity<f64>) -> impl Fn(f64, &mut Window, &mut App) + 'static {
    move |value, _, cx| {
        state.update(cx, |state, cx| {
            *state = value;
            cx.notify();
        })
    }
}

pub fn number_input(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let seats = number("number-seats", 4.0, window, cx);
    let now = *seats.read(cx);
    section(
        "NumberInput",
        "Arrow keys or the stepper move it; drag the grip sideways to scrub.",
        cx,
    )
    .child(probe(
        "number",
        div().w(px(220.0)).child(
            NumberInput::new("seats", now)
                .range(1.0, 99.0)
                .on_change(store(seats)),
        ),
    ))
}

pub fn scrub(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let x = number("scrub-x", 120.0, window, cx);
    let opacity = number("scrub-opacity", 0.8, window, cx);
    let (x_now, opacity_now) = (*x.read(cx), *opacity.read(cx));
    section(
        "ScrubInput",
        "Drag the label sideways, as in a design tool. The field still types.",
        cx,
    )
    .child(probe(
        "scrub",
        div()
            .flex()
            .flex_col()
            .gap_2()
            .w(px(260.0))
            .child(
                ScrubInput::new("scrub-x", "X", x_now)
                    .range(-2000.0, 2000.0)
                    .on_change(store(x)),
            )
            .child(
                ScrubInput::new("scrub-opacity", "Opacity", opacity_now)
                    .range(0.0, 1.0)
                    .step(0.01)
                    .precision(2)
                    .on_change(store(opacity)),
            ),
    ))
}

pub fn money(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let price = number("number-price", 1249.5, window, cx);
    let tax = number("number-tax", 8.0, window, cx);
    let gap = number("number-gap", 16.0, window, cx);
    let unit = window.use_keyed_state("number-unit", cx, |_, _| SharedString::from("px"));
    let (price_now, tax_now, gap_now, unit_now) = (
        *price.read(cx),
        *tax.read(cx),
        *gap.read(cx),
        unit.read(cx).clone(),
    );
    section(
        "CurrencyInput / PercentInput / UnitInput",
        "The same number field with a symbol, a percent, or a unit you click through.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "currency",
                div().w(px(200.0)).child(
                    NumberInput::new("price", price_now)
                        .currency("USD")
                        .range(0.0, 1_000_000.0)
                        .on_change(store(price)),
                ),
                cx,
            ))
            .child(specimen(
                "percent",
                div().w(px(160.0)).child(
                    NumberInput::new("tax", tax_now)
                        .percent()
                        .range(0.0, 100.0)
                        .on_change(store(tax)),
                ),
                cx,
            ))
            .child(specimen(
                "unit",
                probe(
                    "unit",
                    div().w(px(180.0)).child(
                        UnitInput::new("gap", gap_now, ["px", "%", "em"], unit_now)
                            .range(0.0, 400.0)
                            .on_change(store(gap))
                            .on_unit(move |next, _, cx| {
                                let next = next.clone();
                                unit.update(cx, |unit, cx| {
                                    *unit = next;
                                    cx.notify();
                                })
                            }),
                    ),
                ),
                cx,
            )),
    )
    .child(code(
        "NumberInput::new(id, value).currency(\"USD\") and .percent()",
        cx,
    ))
}
