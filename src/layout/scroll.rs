use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, Axis, Bounds, Canvas, Div, ElementId, EmptyView, Entity, EntityId,
    FocusHandle, HoverListenerMode, InteractiveElement, IntoElement, ListOffset, ListState,
    ParentElement, Pixels, Point, RenderOnce, ScrollHandle, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, canvas, div, point, prelude::*, px,
};
use smallvec::SmallVec;
use web_time::Instant;

use super::ScrollShadow;
use crate::{motion, theme::ActiveTheme};

const IDLE: Duration = Duration::from_millis(900);

/// Sets `scroll`'s offset along `axis` whole so `item`, as painted, sits inside the box, its start first when it is longer; true when it moved.
pub(crate) fn bring_into_view(scroll: &ScrollHandle, item: Bounds<Pixels>, axis: Axis) -> bool {
    let (frame, offset) = (scroll.bounds(), scroll.offset());
    let ((start, end), (first, last)) = match axis {
        Axis::Horizontal => ((item.left(), item.right()), (frame.left(), frame.right())),
        Axis::Vertical => ((item.top(), item.bottom()), (frame.top(), frame.bottom())),
    };
    let shift = if start < first {
        first - start
    } else if end > last {
        (last - end).max(first - start)
    } else {
        Pixels::ZERO
    };
    if shift != Pixels::ZERO {
        scroll.set_offset(match axis {
            Axis::Horizontal => point(offset.x + shift, offset.y),
            Axis::Vertical => point(offset.x, offset.y + shift),
        });
    }
    shift != Pixels::ZERO
}

/// A canvas over a Tab stop in a scroll box: when the stop takes focus, it brings its painted box into view along `axis`, once per focus. A new `id` reveals again.
pub(crate) fn reveal_when_focused(
    id: impl Into<ElementId>,
    scroll: &ScrollHandle,
    focus: &FocusHandle,
    axis: Axis,
    window: &mut Window,
    cx: &mut App,
) -> Canvas<()> {
    let shown = window.use_keyed_state(id, cx, |_, _| false);
    let (scroll, focus) = (scroll.clone(), focus.clone());
    canvas(
        move |bounds, window, cx| {
            let (focused, held) = (focus.is_focused(window), *shown.read(cx));
            if focused && !held {
                shown.update(cx, |shown, _| *shown = true);
                if bring_into_view(&scroll, bounds, axis) {
                    log::info!("scroll: a focused stop came into view");
                    window.request_animation_frame();
                }
            } else if !focused && held {
                shown.update(cx, |shown, _| *shown = false);
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// Keeps a scroll box's wheel to its own axes; gpui turns a wheel along the other axis onto a box that scrolls one way, while the page scrolls too.
pub fn on_axis<E: Styled>(mut element: E) -> E {
    element.style().restrict_scroll_to_axis = Some(true);
    element
}

struct Activity {
    handle: ScrollHandle,
    last: Point<Pixels>,
    active_at: Option<Instant>,
    hovered: bool,
}

/// Visibility 0..=1: full while active, fading after `IDLE`.
fn presence(activity: &Activity, fade: Duration) -> f32 {
    if activity.hovered {
        return 1.0;
    }
    let Some(at) = activity.active_at else {
        return 0.0;
    };
    let quiet = at.elapsed().saturating_sub(IDLE);
    1.0 - (quiet.as_secs_f32() / fade.as_secs_f32()).min(1.0)
}

/// Thumb start at press, and the pointer's first position.
struct ThumbDrag {
    owner: EntityId,
    start: Pixels,
    anchor: Rc<Cell<Option<Pixels>>>,
}

/// What a scrollbar moves: a scroll box or a list.
#[derive(Clone)]
pub enum ScrollSource {
    Handle(ScrollHandle),
    List(ListState),
}

impl From<&ScrollHandle> for ScrollSource {
    fn from(handle: &ScrollHandle) -> Self {
        Self::Handle(handle.clone())
    }
}

impl From<&ListState> for ScrollSource {
    fn from(list: &ListState) -> Self {
        Self::List(list.clone())
    }
}

/// Where the view sits: the track, and shares shown and passed.
struct Place {
    span: Pixels,
    shown: f32,
    passed: f32,
}

impl ScrollSource {
    /// A list: its top item and the offset into it.
    fn offset(&self) -> Point<Pixels> {
        match self {
            Self::Handle(handle) => handle.offset(),
            Self::List(list) => {
                let top = list.logical_scroll_top();
                point(px(top.item_ix as f32), top.offset_in_item)
            }
        }
    }

    /// None when everything shows.
    fn place(&self, axis: Axis) -> Option<Place> {
        match self {
            Self::Handle(handle) => {
                let (view, max, offset) =
                    (handle.bounds().size, handle.max_offset(), handle.offset());
                let (span, reach, scrolled) = match axis {
                    Axis::Vertical => (view.height, max.y, -offset.y),
                    Axis::Horizontal => (view.width, max.x, -offset.x),
                };
                (reach > Pixels::ZERO).then(|| Place {
                    span,
                    shown: span / (span + reach),
                    passed: (scrolled / reach).clamp(0.0, 1.0),
                })
            }
            // Lists measure lazily: their place counts items.
            Self::List(list) => {
                let (first, shown, count) = list_items(list)?;
                let rest = count - shown;
                Some(Place {
                    span: list.viewport_bounds().size.height,
                    shown: shown / count,
                    passed: (first / rest).clamp(0.0, 1.0),
                })
            }
        }
    }

    /// Scrolls so `ratio` of what is hidden lies above.
    fn set(&self, axis: Axis, ratio: f32) {
        match self {
            Self::Handle(handle) => {
                let (max, current) = (handle.max_offset(), handle.offset());
                handle.set_offset(match axis {
                    Axis::Vertical => point(current.x, -(max.y * ratio)),
                    Axis::Horizontal => point(-(max.x * ratio), current.y),
                });
            }
            Self::List(list) => {
                let Some((_, shown, count)) = list_items(list) else {
                    return;
                };
                let at = (count - shown) * ratio;
                let item_ix = at.floor() as usize;
                let height = list
                    .bounds_for_item(item_ix)
                    .map_or(Pixels::ZERO, |b| b.size.height);
                list.scroll_to(ListOffset {
                    item_ix,
                    offset_in_item: height * at.fract(),
                });
            }
        }
    }
}

/// Top item as a fraction, items shown, item count.
fn list_items(list: &ListState) -> Option<(f32, f32, f32)> {
    let (count, view) = (list.item_count(), list.viewport_bounds());
    let top = list.logical_scroll_top();
    let height = |ix| list.bounds_for_item(ix).map(|bounds| bounds.size.height);
    let into = match height(top.item_ix) {
        Some(height) if height > Pixels::ZERO => (top.offset_in_item / height).clamp(0.0, 1.0),
        _ => 0.0,
    };
    let mut shown = 0.0;
    for ix in top.item_ix..count {
        let Some(bounds) = list.bounds_for_item(ix) else {
            break;
        };
        if bounds.top() >= view.bottom() {
            break;
        }
        let seen = bounds.bottom().min(view.bottom()) - bounds.top().max(view.top());
        if bounds.size.height > Pixels::ZERO {
            shown += (seen / bounds.size.height).max(0.0);
        }
    }
    let count = count as f32;
    (shown > 0.0 && shown < count - 0.01).then_some((top.item_ix as f32 + into, shown, count))
}

/// How visible a scrollbar is.
#[derive(Clone, Copy)]
enum Shown {
    Given(f32),
    /// While scrolling or hovered, then fading.
    Auto,
}

/// Overlay thumb for a scroll box or list; drag it to scroll.
#[derive(IntoElement)]
pub struct Scrollbar {
    id: ElementId,
    handle: ScrollSource,
    axis: Axis,
    shown: Shown,
}

impl Scrollbar {
    pub fn new(id: impl Into<ElementId>, source: impl Into<ScrollSource>, axis: Axis) -> Self {
        Self {
            id: id.into(),
            handle: source.into(),
            axis,
            shown: Shown::Given(1.0),
        }
    }

    /// Fades the thumb; 0 hides it.
    pub fn presence(mut self, presence: f32) -> Self {
        self.shown = Shown::Given(presence.clamp(0.0, 1.0));
        self
    }

    /// Shows while scrolling or under the pointer, then fades.
    pub fn auto(mut self) -> Self {
        self.shown = Shown::Auto;
        self
    }
}

impl RenderOnce for Scrollbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let owner = window
            .use_keyed_state(self.id.clone(), cx, |_, _| ())
            .entity_id();
        let theme = cx.theme();
        let rem = window.rem_size();
        let thickness = theme.scrollbar_thickness().to_pixels(rem);
        let min_thumb = theme.scrollbar_min_thumb().to_pixels(rem);
        let activity = matches!(self.shown, Shown::Auto).then(|| {
            window.use_keyed_state((self.id.clone(), "activity"), cx, |_, _| Activity {
                handle: ScrollHandle::new(),
                last: Point::default(),
                active_at: None,
                hovered: false,
            })
        });
        let given = match self.shown {
            Shown::Given(presence) => presence,
            Shown::Auto => 0.0,
        };
        let presence = activity.as_ref().map_or(given, |state| {
            track_activity(state, self.handle.offset(), window, cx)
        });
        let hidden = activity.is_none() && presence <= 0.0;
        let Some(place) = self.handle.place(self.axis).filter(|_| !hidden) else {
            return div().into_any_element();
        };
        let span = place.span;
        let length = (span * place.shown).max(min_thumb).min(span);
        let travel = span - length;
        if travel <= Pixels::ZERO {
            return div().into_any_element();
        }
        let theme = cx.theme();
        let start = travel * place.passed;
        let axis = self.axis;
        let handle = self.handle.clone();
        let (idle, busy) = (theme.colors.fg.opacity(0.22), theme.colors.fg.opacity(0.42));
        let thumb = div()
            .id("thumb")
            .debug_selector(|| "scrollbar-thumb".into())
            .absolute()
            .rounded_full()
            .bg(idle)
            .hover(|style| style.bg(busy))
            .on_drag(
                ThumbDrag {
                    owner,
                    start,
                    anchor: Rc::new(Cell::new(None)),
                },
                |_, _, _, cx| cx.new(|_| EmptyView),
            )
            .map(|thumb| match axis {
                Axis::Vertical => thumb.top(start).left_0().w(thickness).h(length),
                Axis::Horizontal => thumb.left(start).top_0().h(thickness).w(length),
            });
        let track = div()
            .id(self.id)
            .absolute()
            .opacity(presence)
            .on_drag_move(move |event: &gpui::DragMoveEvent<ThumbDrag>, window, cx| {
                let drag = event.drag(cx);
                if drag.owner != owner {
                    return;
                }
                let local = event.event.position - event.bounds.origin;
                let along = match axis {
                    Axis::Vertical => local.y,
                    Axis::Horizontal => local.x,
                };
                let anchor = match drag.anchor.get() {
                    Some(anchor) => anchor,
                    None => {
                        drag.anchor.set(Some(along));
                        along
                    }
                };
                let ratio = ((drag.start + along - anchor) / travel).clamp(0.0, 1.0);
                handle.set(axis, ratio);
                window.refresh();
            })
            .when_some(activity, |track, state| {
                track.on_hover(move |hovered, _, cx| {
                    state.update(cx, |activity, cx| {
                        activity.hovered = *hovered;
                        cx.notify();
                    })
                })
            });
        match axis {
            Axis::Vertical => track
                .top_0()
                .bottom_0()
                .right_0()
                .w(thickness * 2.0)
                .pl(thickness * 0.5),
            Axis::Horizontal => track
                .left_0()
                .right_0()
                .bottom_0()
                .h(thickness * 2.0)
                .pt(thickness * 0.5),
        }
        .child(thumb)
        .into_any_element()
    }
}

/// Scroll region with overlay scrollbars that show while in use.
#[derive(IntoElement)]
pub struct ScrollArea {
    id: ElementId,
    base: Div,
    shadows: bool,
    vertical: bool,
    handle: Option<ScrollHandle>,
    body: SmallVec<[AnyElement; 2]>,
}

impl ScrollArea {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            base: div(),
            shadows: false,
            vertical: false,
            handle: None,
            body: SmallVec::new(),
        }
    }

    /// Scrolls through the owner's handle, so the owner can move it.
    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.handle = Some(handle.clone());
        self
    }

    /// Soft edges where content continues.
    pub fn shadows(mut self) -> Self {
        self.shadows = true;
        self
    }

    /// Scrolls down only; content keeps the area's width.
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
}

impl Styled for ScrollArea {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for ScrollArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

fn track_activity(
    state: &Entity<Activity>,
    offset: Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    if state.read(cx).last != offset {
        state.update(cx, |activity, _| {
            activity.last = offset;
            activity.active_at = Some(Instant::now());
        });
    }
    let fade = motion::duration(motion::SLOW, cx);
    let presence = presence(state.read(cx), fade);
    if presence > 0.0 && !state.read(cx).hovered {
        window.request_animation_frame();
    }
    presence
}

impl RenderOnce for ScrollArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| Activity {
            handle: ScrollHandle::new(),
            last: Point::default(),
            active_at: None,
            hovered: false,
        });
        let handle = match self.handle {
            Some(handle) => handle,
            None => state.read(cx).handle.clone(),
        };
        let presence = track_activity(&state, handle.offset(), window, cx);
        self.base
            .id(self.id)
            .relative()
            .overflow_hidden()
            .hover_listener_mode(HoverListenerMode::InputModalityIndependent)
            .on_hover(move |hovered, _, cx| {
                state.update(cx, |activity, cx| {
                    activity.hovered = *hovered;
                    cx.notify();
                })
            })
            .child(
                div()
                    .id("scroll-body")
                    .size_full()
                    .when(self.vertical, |body| {
                        body.overflow_y_scroll().overflow_x_hidden()
                    })
                    .when(!self.vertical, |body| body.overflow_scroll())
                    .track_scroll(&handle)
                    .on_scroll_wheel({
                        let handle = handle.clone();
                        move |event, window, cx| {
                            let delta = event.delta.pixel_delta(window.line_height());
                            let reach = handle.max_offset();
                            let vertical = delta.y != Pixels::ZERO && reach.y > Pixels::ZERO;
                            if vertical || (delta.x != Pixels::ZERO && reach.x > Pixels::ZERO) {
                                cx.stop_propagation();
                            }
                        }
                    })
                    .map(|body| match self.vertical {
                        // Children fill and never widen, so rows wrap.
                        true => body.flex().flex_col().children(
                            self.body
                                .into_iter()
                                .map(|child| div().w_full().min_w_0().child(child)),
                        ),
                        false => body.children(self.body),
                    }),
            )
            .when(self.shadows, |area| area.child(ScrollShadow::new(&handle)))
            .child(Scrollbar::new("scrollbar-y", &handle, Axis::Vertical).presence(presence))
            .when(!self.vertical, |area| {
                area.child(
                    Scrollbar::new("scrollbar-x", &handle, Axis::Horizontal).presence(presence),
                )
            })
    }
}
