use std::{rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div,
};
use web_time::Instant;

use super::{BASE, Entrance, duration, transition::arrive};

type Build = Rc<dyn Fn(&Page, &mut Window, &mut App) -> AnyElement>;

/// The page a [`PageTransition`] asks its builder to draw.
pub struct Page {
    key: SharedString,
}

impl Page {
    /// The key of the page to draw: the shown one, or the one leaving.
    pub fn key(&self) -> &SharedString {
        &self.key
    }
}

/// Which page shows, which one leaves, and how far the switch had come at `since`.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Turn {
    pub(super) shown: SharedString,
    pub(super) leaving: Option<SharedString>,
    pub(super) since: Instant,
    pub(super) start: f32,
}

impl Turn {
    /// A settled turn on `key`.
    pub(super) fn settled(key: SharedString, at: Instant) -> Self {
        Self {
            shown: key,
            leaving: None,
            since: at,
            start: 1.0,
        }
    }

    /// How far the shown page has arrived at `at`: 0 is gone, 1 is in place.
    pub(super) fn share(&self, at: Instant, length: Duration) -> f32 {
        let run = at.saturating_duration_since(self.since).as_secs_f32();
        (self.start + run / length.as_secs_f32().max(f32::EPSILON)).min(1.0)
    }

    /// The turn once `key` is asked for at `at`; the page still leaving comes back from where it stands.
    pub(super) fn switch(&self, key: SharedString, at: Instant, length: Duration) -> Self {
        if key == self.shown {
            return self.clone();
        }
        let share = self.share(at, length);
        let back = self.leaving.as_ref() == Some(&key) && share < 1.0;
        Self {
            shown: key,
            leaving: Some(self.shown.clone()),
            since: at,
            start: if back { 1.0 - share } else { 0.0 },
        }
    }
}

/// One page at a time under a key. When the key changes, the old page leaves while the new one arrives, by fade, rise or slide; a key asked back mid-switch turns around. The builder draws any key it is given, the leaving one too. State kept under a page ends when it stops drawing.
///
/// ```
/// use ely_core::motion::{Entrance, PageTransition};
/// use gpui::{ParentElement, div};
///
/// let pages = PageTransition::new("pages", "inbox", |page, _, _| div().child(page.key().clone()))
///     .entrance(Entrance::Slide);
/// ```
#[derive(IntoElement)]
pub struct PageTransition {
    id: ElementId,
    key: SharedString,
    entrance: Entrance,
    build: Build,
}

impl PageTransition {
    /// Draws `build`'s page for `key` under `id`, with a fade when the key changes.
    pub fn new<E: IntoElement>(
        id: impl Into<ElementId>,
        key: impl Into<SharedString>,
        build: impl Fn(&Page, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            entrance: Entrance::default(),
            build: Rc::new(move |page, window, cx| build(page, window, cx).into_any_element()),
        }
    }

    /// How pages come and go.
    pub fn entrance(mut self, entrance: Entrance) -> Self {
        self.entrance = entrance;
        self
    }
}

impl RenderOnce for PageTransition {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (at, length) = (Instant::now(), duration(BASE, cx));
        let first = self.key.clone();
        let state = window.use_keyed_state((self.id.clone(), "turn"), cx, |_, _| {
            Turn::settled(first, at)
        });
        let was = state.read(cx).clone();
        if was.shown != self.key {
            let next = was.switch(self.key.clone(), at, length);
            log::info!(
                "page transition {:?}: {} to {}, from {:.2}",
                self.id,
                was.shown,
                next.shown,
                next.start
            );
            state.update(cx, |turn, _| *turn = next);
        }
        let turn = state.read(cx).clone();
        let share = turn.share(at, length);
        let (id, build) = (self.id, self.build);
        let page = |key: SharedString, window: &mut Window, cx: &mut App| {
            let body = build(&Page { key: key.clone() }, window, cx);
            div().id((id.clone(), key)).child(body)
        };
        let arriving = page(turn.shown.clone(), window, cx);
        let Some(leaving) = turn.leaving.filter(|_| share < 1.0) else {
            return div().relative().child(arriving);
        };
        window.request_animation_frame();
        let leaving = page(leaving, window, cx)
            .absolute()
            .top_0()
            .left_0()
            .right_0();
        div()
            .relative()
            .overflow_hidden()
            .child(arrive(leaving, self.entrance, 1.0 - share))
            .child(div().absolute().top_0().left_0().size_full().occlude())
            .child(arrive(arriving, self.entrance, share))
    }
}
