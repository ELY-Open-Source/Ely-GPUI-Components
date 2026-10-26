//! The name a `FormField` gives the control laid out inside it, read as a context is.

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, Global, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, SharedString, Window,
};

/// A field's label, and the line under it: its error, else its description.
#[derive(Clone, PartialEq, Default)]
pub(crate) struct Naming {
    pub(crate) name: SharedString,
    pub(crate) note: Option<SharedString>,
}

#[derive(Default)]
struct Scope(Vec<Naming>);

impl Global for Scope {}

/// The naming of the innermost field whose control lays out now.
pub(crate) fn current(cx: &App) -> Option<Naming> {
    cx.try_global::<Scope>()
        .and_then(|scope| scope.0.last().cloned())
}

fn scoped<R>(naming: &Naming, cx: &mut App, f: impl FnOnce(&mut App) -> R) -> R {
    cx.default_global::<Scope>().0.push(naming.clone());
    let out = f(cx);
    cx.global_mut::<Scope>().0.pop();
    out
}

/// Holds `naming` in scope while `child` lays out and prepaints: gpui renders a child as its parent lays out, a cached view in prepaint.
pub(crate) struct Named {
    naming: Naming,
    child: AnyElement,
}

impl Named {
    pub(crate) fn new(naming: Naming, child: AnyElement) -> Self {
        Self { naming, child }
    }
}

impl IntoElement for Named {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for Named {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let child = &mut self.child;
        let layout = scoped(&self.naming, cx, |cx| child.request_layout(window, cx));
        (layout, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let child = &mut self.child;
        scoped(&self.naming, cx, |cx| child.prepaint(window, cx));
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}
