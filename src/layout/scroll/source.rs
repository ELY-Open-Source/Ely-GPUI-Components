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

    /// Scrolls so `ratio` of what is hidden lies above.
    pub(super) fn set(&self, axis: Axis, ratio: f32) {
        match self {
            Self::Handle(handle) => {
                let (max, current) = (handle.max_offset(), handle.offset());
                handle.set_offset(match axis {
                    Axis::Vertical => point(current.x, -(max.y * ratio)),
                    Axis::Horizontal => point(-(max.x * ratio), current.y),
                });
            }
            // Past the last item, gpui keeps to the end.
            Self::List(list) => {
                let at = list.item_count() as f32 * ratio;
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

/// Lists measure lazily, so their place counts items.
fn list_place(list: &ListState) -> Option<Place> {
    let (count, view) = (list.item_count(), list.viewport_bounds());
    let last = count.checked_sub(1)?;
    let top = list.logical_scroll_top();
    let at_end = list
        .bounds_for_item(last)
        .is_some_and(|bounds| bounds.bottom() <= view.bottom());
    if top.item_ix == 0 && top.offset_in_item <= Pixels::ZERO && at_end {
        return None;
    }
    let into = match list.bounds_for_item(top.item_ix) {
        Some(b) if b.size.height > Pixels::ZERO => {
            (top.offset_in_item / b.size.height).clamp(0.0, 1.0)
        }
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
    let passed = match at_end {
        true => 1.0,
        false => ((top.item_ix as f32 + into) / count).clamp(0.0, 1.0),
    };
    Some(Place {
        span: view.size.height,
        shown: (shown / count).clamp(0.0, 1.0),
        passed,
    })
}
