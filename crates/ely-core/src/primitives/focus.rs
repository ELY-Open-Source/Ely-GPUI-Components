use gpui::{
    AnyElement, AnyWindowHandle, App, Div, ElementId, Entity, FocusHandle, Global,
    InteractiveElement, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, StyleRefinement, Styled, Subscription, WeakEntity, Window, actions,
    div, prelude::FluentBuilder,
};

use crate::theme::ActiveTheme;

actions!(
    ely,
    [
        /// Moves focus to the next Tab stop.
        FocusNext,
        /// Moves focus to the previous Tab stop.
        FocusPrev
    ]
);

/// The key context of every `FocusScope`; Tab and Shift-Tab bind under it.
pub(crate) const CONTEXT: &str = "ElyFocus";

/// Focus-colored border while focused. Give the element a 1px border.
pub trait FocusRing: InteractiveElement + Sized {
    /// Turns the border focus-colored while focused.
    fn focus_ring(self, cx: &App) -> Self {
        let ring = cx.theme().colors.focus;
        self.focus(move |style| style.border_color(ring))
    }
}

impl<E: InteractiveElement> FocusRing for E {}

/// Owns Tab and Shift-Tab for its subtree. `trap` keeps focus inside.
///
/// ```
/// use ely_core::primitives::FocusScope;
/// use gpui::{App, ParentElement, div};
///
/// fn root(cx: &mut App) -> FocusScope {
///     let handle = cx.focus_handle();
///     FocusScope::new(&handle).root().child(div())
/// }
/// ```
#[derive(IntoElement)]
pub struct FocusScope {
    base: Div,
    handle: FocusHandle,
    trap: bool,
    root: bool,
    node: Option<Node>,
}

/// What assistive technology reads when the scope itself holds focus.
struct Node {
    id: ElementId,
    role: Role,
    label: SharedString,
    description: Option<SharedString>,
}

impl FocusScope {
    /// A scope that tracks `handle` and owns Tab inside it.
    pub fn new(handle: &FocusHandle) -> Self {
        Self {
            base: div().key_context(CONTEXT).track_focus(handle),
            handle: handle.clone(),
            trap: false,
            root: false,
            node: None,
        }
    }

    /// Reads as `role` named `label` while it holds focus, as a dialog announces itself on opening.
    pub fn node(
        mut self,
        id: impl Into<ElementId>,
        role: Role,
        label: impl Into<SharedString>,
        description: Option<SharedString>,
    ) -> Self {
        self.node = Some(Node {
            id: id.into(),
            role,
            label: label.into(),
            description,
        });
        self
    }

    /// Keeps Tab and Shift-Tab inside, as a dialog does.
    pub fn trap(mut self) -> Self {
        self.trap = true;
        self
    }

    /// The window's outermost scope: it takes focus when the focused element leaves the tree.
    pub fn root(mut self) -> Self {
        self.root = true;
        self
    }
}

/// Keeps a root scope's claim on lost focus while it renders.
struct Root {
    _lost: Subscription,
}

impl Styled for FocusScope {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for FocusScope {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl RenderOnce for FocusScope {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.root {
            let handle = self.handle.clone();
            window.use_keyed_state("focus-root", cx, |window, cx| Root {
                _lost: cx.on_focus_lost(window, move |_: &mut Root, window, cx| {
                    match dropped_overlay(window, cx) {
                        Some(previous) => {
                            log::info!("focus: an overlay left unclosed; focus goes back");
                            window.focus(&previous, cx);
                        }
                        None => {
                            log::info!("focus: its element left the tree; the root takes it");
                            window.focus(&handle, cx);
                        }
                    }
                }),
            });
        }
        let (next, prev) = (self.handle.clone(), self.handle);
        let trap = self.trap;
        let base = self
            .base
            .on_action(move |_: &FocusNext, window, cx| step(&next, trap, true, window, cx))
            .on_action(move |_: &FocusPrev, window, cx| step(&prev, trap, false, window, cx));
        match self.node {
            Some(node) => base
                .id(node.id)
                .role(node.role)
                .aria_label(node.label)
                .when_some(node.description, |scope, text| scope.aria_description(text))
                .into_any_element(),
            None => base.into_any_element(),
        }
    }
}

/// Moves focus from a trapping scope to its first stop, as Tab would.
pub(crate) fn enter_scope(scope: &FocusHandle, window: &mut Window, cx: &mut App) {
    step(scope, true, true, window, cx);
}

fn step(scope: &FocusHandle, trap: bool, forward: bool, window: &mut Window, cx: &mut App) {
    let advance = |window: &mut Window, cx: &mut App| {
        if forward {
            window.focus_next(cx)
        } else {
            window.focus_prev(cx)
        }
    };
    if !trap {
        advance(window, cx);
        return;
    }
    let origin = window.focused(cx);
    let mut first = None;
    loop {
        advance(window, cx);
        let focused = window.focused(cx);
        if scope.contains_focused(window, cx) && focused.as_ref() != Some(scope) {
            return;
        }
        if focused.is_none() || focused == origin || (first.is_some() && focused == first) {
            break;
        }
        if first.is_none() {
            first = focused;
        }
    }
    log::warn!("focus scope: trap holds no tab stop; focus stays");
    if let Some(origin) = origin {
        window.focus(&origin, cx);
    }
}

/// Overlays holding focus: their focus and the prior one.
#[derive(Default)]
struct Holders(Vec<Holder>);

struct Holder {
    takeover: WeakEntity<Takeover>,
    window: AnyWindowHandle,
    focus: FocusHandle,
    previous: Option<FocusHandle>,
}

impl Global for Holders {}

/// Forgets overlays gone unclosed; the focus before them all.
fn dropped_overlay(window: &Window, cx: &mut App) -> Option<FocusHandle> {
    let here = window.window_handle();
    let holders = &mut cx.default_global::<Holders>().0;
    let (gone, open): (Vec<Holder>, Vec<Holder>) = holders
        .drain(..)
        .partition(|holder| holder.takeover.upgrade().is_none());
    *holders = open;
    let gone: Vec<Holder> = gone
        .into_iter()
        .filter(|holder| holder.window == here)
        .collect();
    // Walk back past every gone overlay.
    let mut back = gone.last()?.previous.clone();
    while let Some(holder) = gone
        .iter()
        .find(|holder| Some(&holder.focus) == back.as_ref())
    {
        back = holder.previous.clone();
    }
    back
}

/// Focus an overlay holds while open, and what it held before.
#[doc(hidden)]
pub struct Takeover {
    pub focus: FocusHandle,
    previous: Option<FocusHandle>,
    taken: bool,
    returned: bool,
    _lost: Subscription,
}

/// Keyed focus for an overlay; focused on its first render, and again whenever the focused element leaves the tree.
#[doc(hidden)]
pub fn take_focus(
    key: impl Into<gpui::ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Takeover> {
    let state = window.use_keyed_state(key, cx, |window, cx| Takeover {
        focus: cx.focus_handle(),
        previous: None,
        taken: false,
        returned: false,
        _lost: cx.on_focus_lost(window, |takeover: &mut Takeover, window, cx| {
            if !takeover.returned {
                log::info!("focus: its element left the tree; the overlay takes it back");
                window.focus(&takeover.focus, cx);
            }
        }),
    });
    if !state.read(cx).taken {
        let previous = window.focused(cx);
        window.focus(&state.read(cx).focus.clone(), cx);
        let holder = Holder {
            takeover: state.downgrade(),
            window: window.window_handle(),
            focus: state.read(cx).focus.clone(),
            previous: previous.clone(),
        };
        cx.default_global::<Holders>().0.push(holder);
        state.update(cx, |takeover, _| {
            takeover.previous = previous;
            takeover.taken = true;
        });
    }
    state
}

/// Whether an open overlay holds the window's focus.
#[doc(hidden)]
pub fn lent(window: &Window, cx: &App) -> bool {
    cx.try_global::<Holders>().is_some_and(|holders| {
        (holders.0.iter())
            .any(|holder| holder.takeover.upgrade().is_some() && holder.focus.is_focused(window))
    })
}

/// Returns focus to whatever held it before the overlay opened.
#[doc(hidden)]
pub fn give_back(state: &Entity<Takeover>, window: &mut Window, cx: &mut App) {
    let closed = state.downgrade();
    cx.default_global::<Holders>()
        .0
        .retain(|holder| holder.takeover != closed);
    let previous = state.update(cx, |takeover, _| {
        takeover.returned = true;
        takeover.previous.clone()
    });
    match previous {
        Some(previous) => {
            window.focus(&previous, cx);
            log::info!("focus: handed back after an overlay");
        }
        None => log::info!("focus: overlay closed, nothing was focused before"),
    }
}

/// A mode's own focus, where focus was before the mode took it, and whether the mode was on last frame.
#[doc(hidden)]
pub struct Held {
    pub focus: FocusHandle,
    previous: Option<FocusHandle>,
    was_on: bool,
}

/// The focus a mode keyed `key` holds: turned on while focus is elsewhere, it takes focus so Escape reaches it.
#[doc(hidden)]
pub fn hold_focus(
    key: impl Into<ElementId>,
    on: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Held> {
    let held = window.use_keyed_state(key, cx, |_, cx| Held {
        focus: cx.focus_handle(),
        previous: None,
        was_on: false,
    });
    let (focus, was_on) = (held.read(cx).focus.clone(), held.read(cx).was_on);
    if on && !was_on && !focus.contains_focused(window, cx) {
        log::info!("focus: a mode takes it, so Escape reaches it");
        let previous = window.focused(cx);
        window.focus(&focus, cx);
        held.update(cx, |held, _| held.previous = previous);
    }
    if on != was_on {
        held.update(cx, |held, _| held.was_on = on);
    }
    held
}

/// Hands focus back to where it was before the mode took it.
#[doc(hidden)]
pub fn hand_back(held: &Entity<Held>, window: &mut Window, cx: &mut App) {
    if let Some(previous) = held.update(cx, |held, _| held.previous.take()) {
        log::info!("focus: handed back after a mode");
        window.focus(&previous, cx);
    }
}

/// A focus handle kept for `id`, a Tab stop while enabled.
#[doc(hidden)]
pub fn tab_stop(id: ElementId, enabled: bool, window: &mut Window, cx: &mut App) -> FocusHandle {
    window
        .use_keyed_state(id, cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
        .tab_stop(enabled)
}
