use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, Bounds, ElementId, Entity, InteractiveElement, IntoElement, ParentElement,
    Pixels, Point, RenderOnce, SharedString, Size, Styled, Window, div, point, size,
};
use web_time::Instant;

use super::{BASE, Entrance, duration, ease_in_out_cubic, lerp, transition::arrive};
use crate::primitives::Measure;

type Build = Rc<dyn Fn(&Page, &mut Window, &mut App) -> AnyElement>;

/// Window bounds of each shared child, by page key, then child key.
type Seen = HashMap<(SharedString, SharedString), Bounds<Pixels>>;

/// The page a [`PageTransition`] asks its builder to draw.
pub struct Page {
    key: SharedString,
    seen: Entity<Seen>,
    flying: Rc<HashMap<SharedString, Size<Pixels>>>,
    leaving: bool,
    flights: RefCell<Vec<(SharedString, AnyElement)>>,
}

impl Page {
    /// The key of the page to draw: the shown one, or the one leaving.
    pub fn key(&self) -> &SharedString {
        &self.key
    }

    /// Marks `child` as one element on every page that uses `key`. During a switch it flies from its box on the leaving page to its box on the arriving one, resized on the way, so it should fill the box it is given.
    pub fn shared(&self, key: impl Into<SharedString>, child: impl IntoElement) -> AnyElement {
        let key = key.into();
        let (seen, slot) = (self.seen.clone(), (self.key.clone(), key.clone()));
        let held = Measure::new(key.clone(), move |bounds, _, cx| {
            seen.update(cx, |seen, _| seen.insert(slot.clone(), bounds));
        });
        match self.flying.get(&key) {
            None => held.child(child).into_any_element(),
            Some(_) if self.leaving => held.opacity(0.0).child(child).into_any_element(),
            Some(room) => {
                self.flights
                    .borrow_mut()
                    .push((key, child.into_any_element()));
                held.w(room.width).h(room.height).into_any_element()
            }
        }
    }
}

/// The box a share `t` of the way from `from` to `to`.
pub(super) fn between(from: Bounds<Pixels>, to: Bounds<Pixels>, t: f32) -> Bounds<Pixels> {
    let at = |a: Pixels, b: Pixels| Pixels::from(lerp(f32::from(a), f32::from(b), t));
    Bounds {
        origin: point(
            at(from.origin.x, to.origin.x),
            at(from.origin.y, to.origin.y),
        ),
        size: size(
            at(from.size.width, to.size.width),
            at(from.size.height, to.size.height),
        ),
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

/// One page at a time under a key. When the key changes, the old page leaves while the new one arrives, by fade, rise or slide; a key asked back mid-switch turns around, and a child both pages mark with [`Page::shared`] flies between its two places. The builder draws any key it is given, the leaving one too. State kept under a page ends when it stops drawing.
///
/// ```
/// use ely_core::motion::{Entrance, PageTransition};
/// use gpui::{ParentElement, Styled, div};
///
/// let pages = PageTransition::new("pages", "inbox", |page, _, _| {
///     div().child(page.shared("title", div().size_full().child(page.key().clone())))
/// })
/// .entrance(Entrance::Slide);
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
        let seen = window.use_keyed_state((self.id.clone(), "seen"), cx, |_, _| Seen::new());
        let from = window.use_keyed_state((self.id.clone(), "from"), cx, |_, _| HashMap::new());
        let frame = window.use_keyed_state((self.id.clone(), "origin"), cx, |_, _| None);
        let was = state.read(cx).clone();
        if was.shown != self.key {
            let next = was.switch(self.key.clone(), at, length);
            let places = takeoff(&seen, &next, &was.shown, cx);
            log::info!(
                "page transition {:?}: {} to {}, from {:.2}, {} shared",
                self.id,
                was.shown,
                next.shown,
                next.start,
                places.len()
            );
            from.update(cx, |from, _| *from = places);
            state.update(cx, |turn, _| *turn = next);
        }
        let turn = state.read(cx).clone();
        let share = turn.share(at, length);
        let leaving = turn.leaving.clone().filter(|_| share < 1.0);
        let within = |bounds: &Bounds<Pixels>, origin: Point<Pixels>| Bounds {
            origin: bounds.origin - origin,
            size: bounds.size,
        };
        let landing: HashMap<SharedString, (Bounds<Pixels>, Bounds<Pixels>)> =
            match (&leaving, *frame.read(cx)) {
                (Some(_), Some(origin)) => from
                    .read(cx)
                    .iter()
                    .filter_map(|(key, start)| {
                        let end = seen.read(cx).get(&(turn.shown.clone(), key.clone()))?;
                        Some((key.clone(), (within(start, origin), within(end, origin))))
                    })
                    .collect(),
                _ => HashMap::new(),
            };
        let flying: Rc<HashMap<SharedString, Size<Pixels>>> = Rc::new(
            landing
                .iter()
                .map(|(key, (_, end))| (key.clone(), end.size))
                .collect(),
        );
        let (id, build) = (self.id.clone(), self.build);
        let page = |key: SharedString, leaving: bool, window: &mut Window, cx: &mut App| {
            let page = Page {
                key: key.clone(),
                seen: seen.clone(),
                flying: flying.clone(),
                leaving,
                flights: RefCell::new(Vec::new()),
            };
            let body = build(&page, window, cx);
            (div().id((id.clone(), key)).child(body), page.flights.take())
        };
        let (arriving, flights) = page(turn.shown.clone(), false, window, cx);
        let held = frame.clone();
        let container = Measure::new((self.id, "frame"), move |bounds, _, cx| {
            held.update(cx, |frame, _| *frame = Some(bounds.origin))
        });
        let Some(leaving) = leaving else {
            return container.child(arriving);
        };
        window.request_animation_frame();
        let (leaving, _) = page(leaving, true, window, cx);
        let leaving = leaving.absolute().top_0().left_0().right_0();
        let t = ease_in_out_cubic(share);
        let flights = flights.into_iter().filter_map(|(key, child)| {
            let (start, end) = landing.get(&key)?;
            let now = between(*start, *end, t);
            Some(
                div()
                    .absolute()
                    .left(now.origin.x)
                    .top(now.origin.y)
                    .w(now.size.width)
                    .h(now.size.height)
                    .child(child),
            )
        });
        container
            .overflow_hidden()
            .child(arrive(leaving, self.entrance, 1.0 - share))
            .child(div().absolute().top_0().left_0().size_full().occlude())
            .child(arrive(arriving, self.entrance, share))
            .children(flights)
    }
}

/// Where each shared child of the page now leaving sat; drops what neither page drew.
fn takeoff(
    seen: &Entity<Seen>,
    next: &Turn,
    leaving: &SharedString,
    cx: &mut App,
) -> HashMap<SharedString, Bounds<Pixels>> {
    seen.update(cx, |seen, _| {
        seen.retain(|(page, _), _| page == leaving || *page == next.shown);
        seen.iter()
            .filter(|((page, _), _)| page == leaving)
            .map(|((_, key), bounds)| (key.clone(), *bounds))
            .collect()
    })
}
