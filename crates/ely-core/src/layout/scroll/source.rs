//! What a scrollbar reads and moves.

use std::rc::Rc;

use gpui::{
    App, Axis, IntoElement, ListOffset, ListState, Pixels, Point, ScrollHandle, canvas, point,
};

/// What a scrollbar moves: a box, list or kept scroll.
#[derive(Clone)]
pub enum ScrollSource {
    /// A box's handle.
    Handle(ScrollHandle),
    /// A gpui list.
    List(ListState),
    /// A scroll its owner keeps.
    Model(Rc<dyn ScrollModel>),
}

/// A scroll its owner keeps, such as a canvas.
pub trait ScrollModel {
    /// None when everything shows.
    fn place(&self, axis: Axis, cx: &App) -> Option<ScrollPlace>;
    /// Scrolls so `ratio` of what is hidden lies before.
    fn set(&self, axis: Axis, ratio: f32, cx: &mut App);
    /// Changes whenever the scroll moves.
    fn offset(&self, cx: &App) -> Point<Pixels>;
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

/// The track, and the shares shown and passed.
pub struct ScrollPlace {
    /// The track's length.
    pub span: Pixels,
    /// The share of the content in view.
    pub shown: f32,
    /// The hidden share that lies before the view.
    pub passed: f32,
}

impl ScrollSource {
    /// A list: its top item and the offset into it.
    pub(super) fn offset(&self, cx: &App) -> Point<Pixels> {
        match self {
            Self::Handle(handle) => handle.offset(),
            Self::List(list) => {
                let top = list.logical_scroll_top();
                point(Pixels::from(top.item_ix), top.offset_in_item)
            }
            Self::Model(model) => model.offset(cx),
        }
    }

    /// None when everything shows.
    pub(super) fn place(&self, axis: Axis, cx: &App) -> Option<ScrollPlace> {
        match self {
            Self::Handle(handle) => {
                let (view, max, offset) =
                    (handle.bounds().size, handle.max_offset(), handle.offset());
                let (span, reach, scrolled) = match axis {
                    Axis::Vertical => (view.height, max.y, -offset.y),
                    Axis::Horizontal => (view.width, max.x, -offset.x),
                };
                (reach > Pixels::ZERO).then(|| ScrollPlace {
                    span,
                    shown: span / (span + reach),
                    passed: (scrolled / reach).clamp(0.0, 1.0),
                })
            }
            Self::List(list) => list_place(list),
            Self::Model(model) => model.place(axis, cx),
        }
    }

    /// A list's end in items, to hold through one drag.
    pub(super) fn end(&self) -> Option<f32> {
        match self {
            Self::Handle(_) | Self::Model(_) => None,
            Self::List(list) => ends(list).map(|ends| ends.end),
        }
    }

    /// Scrolls so `ratio` of what is hidden lies above.
    pub(super) fn set(&self, axis: Axis, ratio: f32, end: Option<f32>, cx: &mut App) {
        match self {
            Self::Handle(handle) => {
                let (max, current) = (handle.max_offset(), handle.offset());
                handle.set_offset(match axis {
                    Axis::Vertical => point(current.x, -(max.y * ratio)),
                    Axis::Horizontal => point(-(max.x * ratio), current.y),
                });
            }
            Self::Model(model) => model.set(axis, ratio, cx),
            Self::List(list) if ratio >= 1.0 => list.scroll_to_end(),
            Self::List(list) => {
                // A measured end wins over the one held at press.
                let measured = measured_end(list).or(end);
                let Some(end) = measured.or_else(|| ends(list).map(|ends| ends.end)) else {
                    return;
                };
                let at = end * ratio;
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

/// Top, items in view and attainable top, in items.
struct Ends {
    first: f32,
    shown: f32,
    end: f32,
    at_end: bool,
}

/// None when everything shows; lists measure lazily.
fn ends(list: &ListState) -> Option<Ends> {
    let (count, view) = (list.item_count(), list.viewport_bounds());
    let last = count.checked_sub(1)?;
    let top = list.logical_scroll_top();
    let height = |ix| list.bounds_for_item(ix).map(|b| b.size.height);
    let into = match height(top.item_ix) {
        Some(height) if height > Pixels::ZERO => (top.offset_in_item / height).clamp(0.0, 1.0),
        _ => 0.0,
    };
    let first = top.item_ix as f32 + into;
    let (mut shown, mut at_end) = (0.0, false);
    // Measured without a gap from the top, so bounds hold.
    for ix in top.item_ix..count {
        let Some(bounds) = list.bounds_for_item(ix) else {
            break;
        };
        at_end = ix == last && bounds.bottom() <= view.bottom();
        if bounds.top() >= view.bottom() {
            break;
        }
        let seen = bounds.bottom().min(view.bottom()) - bounds.top().max(view.top());
        if bounds.size.height > Pixels::ZERO {
            shown += (seen / bounds.size.height).max(0.0);
        }
    }
    if top.item_ix == 0 && top.offset_in_item <= Pixels::ZERO && at_end {
        return None;
    }
    let end = measured_end(list).unwrap_or((count as f32 - shown).max(first));
    Some(Ends {
        first,
        shown,
        end,
        at_end,
    })
}

/// The attainable top in items, read from a measured tail.
fn measured_end(list: &ListState) -> Option<f32> {
    let mut left = list.viewport_bounds().size.height;
    for ix in (0..list.item_count()).rev() {
        let height = list.bounds_for_item(ix)?.size.height;
        if height > Pixels::ZERO && height >= left {
            return Some(ix as f32 + (height - left) / height);
        }
        left -= height;
    }
    Some(0.0)
}

fn list_place(list: &ListState) -> Option<ScrollPlace> {
    let Ends {
        first,
        shown,
        end,
        at_end,
    } = ends(list)?;
    let passed = match at_end {
        true => 1.0,
        false => (first / end.max(f32::EPSILON)).clamp(0.0, 1.0),
    };
    Some(ScrollPlace {
        span: list.viewport_bounds().size.height,
        shown: (shown / (end + shown).max(f32::EPSILON)).clamp(0.0, 1.0),
        passed,
    })
}

/// A bar's offset and thumb place.
pub(super) type Seen = (Point<Pixels>, Option<(Pixels, f32, f32)>);

pub(super) fn seen(source: &ScrollSource, axis: Axis, cx: &App) -> Seen {
    let place = source
        .place(axis, cx)
        .map(|place| (place.span, place.shown, place.passed));
    (source.offset(cx), place)
}

/// Asks one more frame if the source moved after render.
pub(super) fn follow(source: ScrollSource, axis: Axis, drawn: Seen) -> impl IntoElement {
    canvas(
        move |_, window, cx| {
            if seen(&source, axis, cx) != drawn {
                window.request_animation_frame();
            }
        },
        |_, _, _, _| {},
    )
}
