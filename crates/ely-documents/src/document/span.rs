//! A selection across blocks: they open as one field.

use std::ops::Range;

use gpui::{
    Context, DispatchPhase, Entity, Focusable as _, IntoElement, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, Styled, Window, canvas,
};

use super::{MarkdownDocument, Open};
use crate::{forms::TextInput, modes::at_edge};

/// A left press that may drag past its blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Drag {
    /// The pressed field follows its own drag.
    Pressed,
    /// The field grew; the document moves the head.
    Across,
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
    /// The first and last blocks `range` covers.
    pub(super) fn covered(&self, range: &Range<usize>) -> (usize, usize) {
        let last = range.end.saturating_sub(1).max(range.start);
        (self.block_at(range.start), self.block_at(last))
    }

    /// A press left a field open; a drag may follow.
    pub(super) fn pressed_open(&mut self) {
        self.drag = (self.open.as_ref())
            .filter(|open| open.cell.is_none() && !open.append)
            .map(|_| Drag::Pressed);
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

    /// Follows a drag to `position`, growing the field.
    fn drag_to(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(drag), Some(open)) = (self.drag, self.open.as_ref()) else {
            return;
        };
        // Past the view's top or bottom, the list scrolls.
        let view = self.list.viewport_bounds();
        let below = (position.y - view.bottom()).max(Pixels::ZERO);
        let above = (view.top() - position.y).max(Pixels::ZERO);
        if below > Pixels::ZERO || above > Pixels::ZERO {
            self.list.scroll_by(below - above);
            cx.notify();
        }
        let ix = self
            .item_at(position)
            .min(self.parsed.blocks.len().saturating_sub(1));
        let (first, last) = self.covered(&open.range);
        if (first..=last).contains(&ix) {
            if drag == Drag::Across {
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
        self.drag = Some(Drag::Across);
    }

    /// Shift+Up or Down past the field's edge selects on.
    pub(super) fn extend(
        &mut self,
        down: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(open) = (self.open.as_ref()).filter(|open| open.cell.is_none() && !open.append)
        else {
            return false;
        };
        let input = open.input.read(cx);
        if !at_edge(input, down) {
            return false;
        }
        let (first, last) = self.covered(&open.range);
        let next = match down {
            true => Some(last + 1).filter(|next| *next < self.parsed.blocks.len()),
            false => first.checked_sub(1),
        };
        let (text, head) = (input.text(), input.cursor());
        let line = text[..head].rfind('\n').map_or(0, |at| at + 1);
        let column = text[line..head].chars().count();
        let Some((input, place)) = next.and_then(|next| self.widen(next, window, cx)) else {
            return false;
        };
        input.update(cx, |input, cx| {
            let block = &input.text()[place.clone()];
            // Its first line going down, its last going up.
            let line = match down {
                true => place.start..place.start + block.find('\n').unwrap_or(block.len()),
                false => place.start + block.rfind('\n').map_or(0, |at| at + 1)..place.end,
            };
            let shown = &input.text()[line.clone()];
            let shown = shown.strip_suffix('\r').unwrap_or(shown);
            let at = shown
                .char_indices()
                .nth(column)
                .map_or(shown.len(), |(at, _)| at);
            let anchor = ends(input).0;
            input.select_from(anchor, line.start + at, cx);
        });
        true
    }

    /// Opens block `ix` into a new field; where its text lies.
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
        let range = open.range.start.min(target.start)..open.range.end.max(target.end);
        let before = &text[range.start..open.range.start];
        let source = [before, typed, &text[open.range.end..range.end]].concat();
        let (anchor, head) = ends(input);
        let place = match target.start < open.range.start {
            true => target.start - range.start,
            false => before.len() + typed.len() + target.start - open.range.end,
        };
        let shift = before.len();
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

    /// Follows a drag that left a field until the button lifts.
    pub(super) fn drag_listener(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let document = cx.entity();
        canvas(
            |_, _, _| {},
            move |_, _, window, _| {
                let moved = document.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if phase == DispatchPhase::Bubble && event.dragging() {
                        moved.update(cx, |document, cx| {
                            document.drag_to(event.position, window, cx)
                        });
                    }
                });
                let lifted = document.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase == DispatchPhase::Bubble {
                        lifted.update(cx, |document, cx| {
                            document.drag_to(event.position, window, cx);
                            document.drag = None;
                        });
                    }
                });
            },
        )
        .absolute()
    }
}
