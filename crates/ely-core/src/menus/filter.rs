//! Filtered menus: arrows focus the rows, typing refocuses the filter.

use gpui::{
    App, Div, ElementInputHandler, Entity, EntityInputHandler, FocusHandle, InteractiveElement,
    ParentElement, Stateful, Styled, Window, canvas,
};

use super::{
    menu::{Open, keys, on_buttons},
    model::Menu,
};
use crate::forms::{Backspace, Down, Enter, Left, Right, Run, TextInput, Up};

/// The panel `root` takes the filter's arrows and its typing.
pub(super) fn arrows(
    root: Stateful<Div>,
    (menu, state, close): (&Menu, &Entity<Open>, &Run),
    panel: &FocusHandle,
    field: &Entity<TextInput>,
) -> Stateful<Div> {
    let walk = |key: &'static str, buttons_only: bool| {
        let (menu, state, close) = (menu.clone(), state.clone(), close.clone());
        let (panel, field) = (panel.clone(), field.clone());
        move |window: &mut Window, cx: &mut App| {
            // A button row takes the side arrows from the filter.
            if buttons_only && !on_buttons(&menu, &state, cx) {
                return false;
            }
            keys(&menu, &state, key, &close, window, cx);
            to_rows(&state, &panel, &field, window, cx);
            true
        }
    };
    let (up, down) = (walk("up", false), walk("down", false));
    let (left, right) = (walk("left", true), walk("right", true));
    root.child(typing(panel.clone(), field.clone()))
        .capture_action(move |_: &Up, window, cx| {
            up(window, cx);
            cx.stop_propagation();
        })
        .capture_action(move |_: &Down, window, cx| {
            down(window, cx);
            cx.stop_propagation();
        })
        .capture_action(|_: &Enter, _, cx| cx.stop_propagation())
        .capture_action(move |_: &Left, window, cx| {
            if left(window, cx) {
                cx.stop_propagation();
            }
        })
        .capture_action(move |_: &Right, window, cx| {
            if right(window, cx) {
                cx.stop_propagation();
            }
        })
}

/// Lends the panel focus; gpui heeds marks only under focus.
fn to_rows(
    state: &Entity<Open>,
    panel: &FocusHandle,
    field: &Entity<TextInput>,
    window: &mut Window,
    cx: &mut App,
) {
    let text = field.read(cx).text().to_string();
    state.update(cx, |open, _| open.listing = Some(text));
    window.focus(panel, cx);
}

/// Focuses the field on opening and after typing changes it.
pub(super) fn settle(
    state: &Entity<Open>,
    panel: &FocusHandle,
    field: &Entity<TextInput>,
    window: &mut Window,
    cx: &mut App,
) {
    if !panel.is_focused(window) {
        if state.read(cx).listing.is_some() {
            state.update(cx, |open, _| open.listing = None);
        }
        return;
    }
    let text = field.read(cx).text().to_string();
    let typed = match &state.read(cx).listing {
        None => true,
        Some(was) => *was != text && !composing(field, window, cx),
    };
    if typed {
        log::info!("filtered menu: focus back to the filter");
        back(state, field, window, cx);
    }
}

/// Backspace on the rows edits the filter from there.
pub(super) fn backspace(
    state: &Entity<Open>,
    field: &Entity<TextInput>,
    window: &mut Window,
    cx: &mut App,
) {
    back(state, field, window, cx);
    let focus = field.read(cx).focus().clone();
    focus.dispatch_action(&Backspace, window, cx);
}

/// The field's text handler while the panel holds focus.
fn typing(panel: FocusHandle, field: Entity<TextInput>) -> impl gpui::IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            window.handle_input(&panel, ElementInputHandler::new(bounds, field), cx)
        },
    )
    .absolute()
    .size_0()
}

fn back(state: &Entity<Open>, field: &Entity<TextInput>, window: &mut Window, cx: &mut App) {
    state.update(cx, |open, _| open.listing = None);
    let focus = field.read(cx).focus().clone();
    window.focus(&focus, cx);
}

/// Whether an input method is still composing in `field`.
fn composing(field: &Entity<TextInput>, window: &mut Window, cx: &mut App) -> bool {
    field.update(cx, |input, cx| {
        input.marked_text_range(window, cx).is_some()
    })
}
