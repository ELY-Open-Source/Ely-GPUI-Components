//! A selection across blocks: they open as one field.

use std::{ops::Range, time::Duration};

use gpui::{
    Context, DispatchPhase, Entity, Focusable as _, IntoElement, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, Styled, Task, Window, canvas, point,
};

use super::{MarkdownDocument, Open};
use crate::{
    forms::TextInput,
    modes::{at_edge, joint},
};

/// How often a drag held past an edge scrolls.
const TICK: Duration = Duration::from_millis(16);

/// A left press that may drag past its blocks.
#[derive(Default)]
pub(super) struct Drag {
    /// The field grew; the document moves the head.
    across: bool,
    /// The pointer, last seen.
    at: Option<Point<Pixels>>,
    /// Scrolls past an edge; dropped with the drag.
    scrolling: Option<Task<()>>,
}

/// A head to place once the grown field is drawn.
pub(super) struct Aim {
    input: Entity<TextInput>,
    /// The caret's left edge before the field grew.
    x: Pixels,
    /// Where the added block lies in the field.
    place: Range<usize>,
    down: bool,
    /// The field's edits and selection when aimed.
    seen: (u64, Range<usize>),
    asked: bool,
}

/// The selection's fixed and moving ends.
fn ends(input: &TextInput) -> (usize, usize) {
    let (picked, head) = (input.selection(), input.cursor());
    match head == picked.start {
        true => (picked.end, head),
        false => (picked.start, head),
    }
}

impl MarkdownDocument {
    /// The blocks `open` covers; a new end lies past them.
    pub(super) fn covered(&self, open: &Open) -> (usize, usize) {
        if open.append {
            let end = self.parsed.blocks.len();
            return (end, end);
        }
        let range = &open.range;
        let last = range.end.saturating_sub(1).max(range.start);
        (self.block_at(range.start), self.block_at(last))
    }

    /// A press left a field open; a drag may follow.
    pub(super) fn pressed_open(&mut self) {
        self.aim = None;
        self.drag = (self.open.as_ref())
            .filter(|open| open.cell.is_none())
            .map(|_| Drag::default());
    }

    /// The list item at `position`'s height, else the nearest.
    fn item_at(&self, position: Point<Pixels>) -> usize {
        let top = self.list.logical_scroll_top().item_ix;
        let mut found = top;
        for ix in top..self.list.item_count() {
            match self.list.bounds_for_item(ix) {
                Some(bounds) if bounds.top() <= position.y => found = ix,
                _ => break,
            }
        }
        found
    }

    /// How far `position` lies past the view; above is negative.
    fn past(&self, position: Point<Pixels>) -> Pixels {
        let view = self.list.viewport_bounds();
        let below = (position.y - view.bottom()).max(Pixels::ZERO);
        below - (view.top() - position.y).max(Pixels::ZERO)
    }

    /// A drag at `position`; past an edge, scrolling starts.
    fn drag_to(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let outside = self.past(position) != Pixels::ZERO;
        let Some(drag) = self.drag.as_mut() else {
            return;
        };
        drag.at = Some(position);
        if outside && drag.scrolling.is_none() {
            log::debug!("markdown document: a drag past the view scrolls it");
            drag.scrolling = Some(cx.spawn_in(window, async move |document, cx| {
                loop {
                    cx.background_executor().timer(TICK).await;
                    let step =
                        |document: &mut Self, window: &mut Window, cx: &mut Context<Self>| {
                            document.edge_scroll(window, cx)
                        };
                    if !document.update_in(cx, step).unwrap_or(false) {
                        break;
                    }
                }
            }));
        }
        self.follow(position, window, cx);
    }

    /// One scroll step for a drag held past an edge.
    fn edge_scroll(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(at) = self.drag.as_ref().and_then(|drag| drag.at) else {
            return false;
        };
        let past = self.past(at);
        if past == Pixels::ZERO {
            if let Some(drag) = self.drag.as_mut() {
                drag.scrolling = None;
            }
            return false;
        }
        self.list.scroll_by(past / 2.0);
        cx.notify();
        self.follow(at, window, cx);
        true
    }

    /// Moves the head to `position`, growing the field.
    fn follow(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(drag), Some(open)) = (self.drag.as_ref(), self.open.as_ref()) else {
            return;
        };
        let end = self.parsed.blocks.len();
        let (first, last) = self.covered(open);
        // Past the blocks: a new end's own, else the last.
        let ix = match self.item_at(position).min(end) {
            ix if ix == end && last < end => end.saturating_sub(1),
            ix => ix,
        };
        if (first..=last).contains(&ix) {
            if drag.across {
                open.input.clone().update(cx, |input, cx| {
                    let head = input.offset_for_point(position);
                    input.select_from(ends(input).0, head, cx)
                });
            }
            return;
        }
        let Some((input, place)) = self.widen(ix, window, cx) else {
            return;
        };
        let head = if ix > last { place.start } else { place.end };
        input.update(cx, |input, cx| input.select_from(ends(input).0, head, cx));
        if let Some(drag) = self.drag.as_mut() {
            drag.across = true;
        }
    }

    /// Shift+Up or Down past the field's edge selects on.
    pub(super) fn extend(
        &mut self,
        down: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.aim = None;
        let Some(open) = (self.open.as_ref()).filter(|open| open.cell.is_none()) else {
            return false;
        };
        let input = open.input.read(cx);
        if !at_edge(input, down) {
            return false;
        }
        let (first, last) = self.covered(open);
        let next = match down {
            true => Some(last + 1).filter(|next| *next < self.parsed.blocks.len()),
            false => first.checked_sub(1),
        };
        let x = input.bounds_for(input.cursor()).map(|caret| caret.left());
        let Some((input, place)) = next.and_then(|next| self.widen(next, window, cx)) else {
            return false;
        };
        // Its first line going down, its last going up.
        let edge = if down { place.start } else { place.end };
        let seen = input.update(cx, |input, cx| {
            input.select_from(ends(input).0, edge, cx);
            (input.edits().0, input.selection())
        });
        self.aim = x.map(|x| Aim {
            input,
            x,
            place,
            down,
            seen,
            asked: false,
        });
        true
    }

    /// Places an aimed head a frame after its field draws.
    pub(super) fn aim_waiting(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(aim) = self.aim.as_mut().filter(|aim| !aim.asked) else {
            return;
        };
        aim.asked = true;
        cx.on_next_frame(window, |document, _, cx| document.aimed(cx));
    }

    /// The head on the added block's line, at its x.
    fn aimed(&mut self, cx: &mut Context<Self>) {
        let Some(aim) = self.aim.take() else {
            return;
        };
        if self
            .open
            .as_ref()
            .is_none_or(|open| open.input != aim.input)
        {
            return log::info!("markdown document: the aimed field closed first");
        }
        aim.input.update(cx, |input, cx| {
            // Typing or a move since lets it lapse.
            if (input.edits().0, input.selection()) != aim.seen {
                return log::info!("markdown document: the aimed head lapsed");
            }
            let edge = if aim.down {
                aim.place.start
            } else {
                aim.place.end
            };
            let Some(line) = input.bounds_for(edge) else {
                return log::warn!("markdown document: the grown field is not laid out");
            };
            let head = input.offset_for_point(point(aim.x, line.center().y));
            let head = head.clamp(aim.place.start, aim.place.end);
            input.select_from(ends(input).0, head, cx);
        });
    }

    /// Opens block `ix` into a new field; where it lies.
    fn widen(
        &mut self,
        ix: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<(Entity<TextInput>, Range<usize>)> {
        if !self.read_now(cx) {
            log::info!("markdown document: a selection waits for the text's read");
            return None;
        }
        let open = self.open.as_ref()?;
        let target = self.parsed.blocks.get(ix)?.range.clone();
        let text = self.editor.read(cx).text();
        let input = open.input.read(cx);
        let typed = input.text();
        // A new end writes back after its joint.
        let lead = match open.append && !typed.is_empty() {
            true => joint(text),
            false => "",
        };
        let range = open.range.start.min(target.start)..open.range.end.max(target.end);
        let before = &text[range.start..open.range.start];
        let source = [before, lead, typed, &text[open.range.end..range.end]].concat();
        let (anchor, head) = ends(input);
        let shift = before.len() + lead.len();
        let place = match target.start < open.range.start {
            true => target.start - range.start,
            false => shift + typed.len() + target.start - open.range.end,
        };
        let (input, subscriptions) = Self::new_field(source, false, window, cx);
        input.update(cx, |input, cx| {
            input.select_from(anchor + shift, head + shift, cx)
        });
        log::info!("markdown document: the open field grows to {range:?}");
        self.open = Some(Open {
            range,
            input: input.clone(),
            append: false,
            cell: None,
            lent: None,
            _subscriptions: subscriptions,
        });
        window.focus(&input.focus_handle(cx), cx);
        cx.notify();
        Some((input, place..place + target.len()))
    }

    /// Follows a drag past its field until the button lifts.
    pub(super) fn drag_listener(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let document = cx.entity();
        canvas(
            |_, _, _| {},
            move |_, _, window, _| {
                let moved = document.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    moved.update(cx, |document, cx| match event.dragging() {
                        true => document.drag_to(event.position, window, cx),
                        // A release we never heard.
                        false => document.drag = None,
                    });
                });
                let lifted = document.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase == DispatchPhase::Bubble {
                        lifted.update(cx, |document, cx| {
                            document.follow(event.position, window, cx);
                            document.drag = None;
                        });
                    }
                });
            },
        )
        .absolute()
    }
}
