//! What a scrollbar reads and moves.

use gpui::{Axis, ListOffset, ListState, Pixels, Point, ScrollHandle, point};

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

/// The track, and the shares shown and passed.
pub(super) struct Place {
    pub span: Pixels,
    pub shown: f32,
    pub passed: f32,
}

impl ScrollSource {
    /// A list: its top item and the offset into it.
    pub(super) fn offset(&self) -> Point<Pixels> {
        match self {
            Self::Handle(handle) => handle.offset(),
            Self::List(list) => {
                let top = list.logical_scroll_top();
                point(Pixels::from(top.item_ix), top.offset_in_item)
            }
        }
    }

    /// None when everything shows.
    pub(super) fn place(&self, axis: Axis) -> Option<Place> {
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
            Self::List(list) => list_place(list),
        }
    }

    /// A list's end in items, to hold through one drag.
    pub(super) fn end(&self) -> Option<f32> {
        match self {
            Self::Handle(_) => None,
            Self::List(list) => ends(list).map(|ends| ends.end),
        }
    }

    /// Scrolls so `ratio` of what is hidden lies above.
    pub(super) fn set(&self, axis: Axis, ratio: f32, end: Option<f32>) {
        match self {
            Self::Handle(handle) => {
                let (max, current) = (handle.max_offset(), handle.offset());
                handle.set_offset(match axis {
                    Axis::Vertical => point(current.x, -(max.y * ratio)),
                    Axis::Horizontal => point(-(max.x * ratio), current.y),
                });
            }
            Self::List(list) if ratio >= 1.0 => list.scroll_to_end(),
            Self::List(list) => {
                let Some(end) = end.or_else(|| ends(list).map(|ends| ends.end)) else {
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
    let last = list.bounds_for_item(count.checked_sub(1)?);
    let top = list.logical_scroll_top();
    let at_end = last.is_some_and(|bounds| bounds.bottom() <= view.bottom());
    if top.item_ix == 0 && top.offset_in_item <= Pixels::ZERO && at_end {
        return None;
    }
    let height = |ix| list.bounds_for_item(ix).map(|b| b.size.height);
    let into = match height(top.item_ix) {
        Some(height) if height > Pixels::ZERO => (top.offset_in_item / height).clamp(0.0, 1.0),
        _ => 0.0,
    };
    let first = top.item_ix as f32 + into;
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
    let end = match last {
        // The last in view: the rest scrolls exactly.
        Some(last) => first + rest(list, first, last.bottom() - view.bottom()),
        None => (count as f32 - shown).max(first),
    };
    Some(Ends {
        first,
        shown,
        end,
        at_end,
    })
}

/// Items the top passes scrolling `left` more, from `first`.
fn rest(list: &ListState, first: f32, mut left: Pixels) -> f32 {
    let (mut ix, mut into, mut moved) = (first.floor() as usize, first.fract(), 0.0);
    while left > Pixels::ZERO {
        let Some(bounds) = list.bounds_for_item(ix) else {
            break;
        };
        let height = bounds.size.height;
        let room = height * (1.0 - into);
        if height > Pixels::ZERO && left <= room {
            return moved + left / height;
        }
        moved += 1.0 - into;
        left -= room;
        (ix, into) = (ix + 1, 0.0);
    }
    moved
}

fn list_place(list: &ListState) -> Option<Place> {
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
    Some(Place {
        span: list.viewport_bounds().size.height,
        shown: (shown / (end + shown).max(f32::EPSILON)).clamp(0.0, 1.0),
        passed,
    })
}
