//! Read-only text the pointer selects across blocks.

use std::{cell::RefCell, ops::Range, rc::Rc};

use gpui::{
    App, ClipboardItem, CursorStyle, Div, Entity, FocusHandle, HighlightStyle, InteractiveElement,
    MouseButton, Pixels, Point, Styled, TextLayout, Window, point,
};

use crate::theme::ActiveTheme;

/// What a right press on selectable text found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RightPress {
    /// The selected text, if any.
    pub selected: Option<String>,
    /// The address of the link pressed.
    pub link: Option<String>,
    /// The address of the picture pressed.
    pub image: Option<String>,
    /// Where the press fell, in window pixels.
    pub position: Point<Pixels>,
}

/// Takes a right press on selectable text.
pub(crate) type OnMenu = Rc<dyn Fn(RightPress, &mut Window, &mut App)>;

/// A byte of one drawn text, texts counted in drawing order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Spot {
    text: usize,
    at: usize,
}

/// One drawn text, as the last frame laid it out.
#[derive(Clone)]
pub(crate) struct Drawn {
    /// A list item's marker, copied before its first text.
    lead: String,
    text: String,
    layout: TextLayout,
    links: Vec<(Range<usize>, String)>,
}

/// A body's selection, kept across frames.
pub(crate) struct Selection {
    focus: FocusHandle,
    anchor: Spot,
    head: Spot,
    dragging: bool,
    texts: Vec<Drawn>,
}

impl Selection {
    pub(crate) fn new(cx: &mut App) -> Self {
        Self {
            focus: cx.focus_handle(),
            anchor: Spot::default(),
            head: Spot::default(),
            dragging: false,
            texts: Vec::new(),
        }
    }

    fn span(&self) -> (Spot, Spot) {
        (self.anchor.min(self.head), self.anchor.max(self.head))
    }

    /// The selected text, blocks joined by line ends.
    fn selected(&self) -> Option<String> {
        copied(&self.texts, self.span())
    }

    /// The spot under `position`: a text beside it, or one past it.
    fn spot(&self, position: Point<Pixels>) -> Option<Spot> {
        let last = self.texts.len().checked_sub(1)?;
        let mut beside: Option<(Pixels, usize)> = None;
        for (k, drawn) in self.texts.iter().enumerate() {
            let bounds = drawn.layout.bounds();
            if position.y < bounds.top() {
                if beside.is_none() {
                    return Some(Spot { text: k, at: 0 });
                }
                break;
            }
            if position.y <= bounds.bottom() {
                let gap = (bounds.left() - position.x)
                    .max(position.x - bounds.right())
                    .max(Pixels::ZERO);
                if beside.is_none_or(|(best, _)| gap < best) {
                    beside = Some((gap, k));
                }
            }
        }
        let Some((_, k)) = beside else {
            let at = self.texts[last].text.len();
            return Some(Spot { text: last, at });
        };
        let layout = &self.texts[k].layout;
        let bounds = layout.bounds();
        let across = point(position.x.clamp(bounds.left(), bounds.right()), position.y);
        let (Ok(at) | Err(at)) = layout.index_for_position(across);
        Some(Spot { text: k, at })
    }

    fn link(&self, spot: Spot) -> Option<String> {
        let links = &self.texts.get(spot.text)?.links;
        let (_, url) = links.iter().find(|(range, _)| range.contains(&spot.at))?;
        Some(url.clone())
    }
}

/// `texts` inside the span; a marker leads text taken from its start.
fn copied(texts: &[Drawn], (from, to): (Spot, Spot)) -> Option<String> {
    if from == to {
        return None;
    }
    let mut parts = Vec::new();
    for (k, drawn) in texts.iter().enumerate().take(to.text + 1).skip(from.text) {
        let range = part(k, drawn.text.len(), (from, to));
        let led = range.start == 0 && !range.is_empty();
        let Some(text) = drawn.text.get(range) else {
            log::error!("markdown: a selection fell inside a character");
            return None;
        };
        parts.push(match led {
            true => format!("{}{text}", drawn.lead),
            false => text.to_owned(),
        });
    }
    Some(parts.join("\n"))
}

/// Text `k`'s bytes inside the span.
fn part(k: usize, len: usize, (from, to): (Spot, Spot)) -> Range<usize> {
    let start = if k == from.text { from.at } else { 0 };
    let end = if k == to.text { to.at } else { len };
    start.min(end)..end
}

/// What one frame's drawing reads and gathers.
pub(crate) struct Gather {
    pub(crate) state: Entity<Selection>,
    pub(crate) on_menu: OnMenu,
    span: (Spot, Spot),
    texts: RefCell<Vec<Drawn>>,
    lead: RefCell<String>,
}

impl Gather {
    pub(crate) fn new(state: Entity<Selection>, on_menu: OnMenu, cx: &App) -> Self {
        let span = state.read(cx).span();
        Self {
            state,
            on_menu,
            span,
            texts: RefCell::default(),
            lead: RefCell::default(),
        }
    }

    /// The selection's highlight on `text`, the next drawn.
    pub(crate) fn over(&self, text: &str, cx: &App) -> Vec<(Range<usize>, HighlightStyle)> {
        let k = self.texts.borrow().len();
        let (from, to) = self.span;
        let range = part(k, text.len(), self.span);
        if from == to || k < from.text || k > to.text || range.is_empty() {
            return Vec::new();
        }
        // Last frame's span may not fit new text.
        let fits = range.end <= text.len()
            && text.is_char_boundary(range.start)
            && text.is_char_boundary(range.end);
        if !fits {
            return Vec::new();
        }
        let style = HighlightStyle {
            background_color: Some(cx.theme().colors.selection),
            ..Default::default()
        };
        vec![(range, style)]
    }

    /// Notes a drawn text, in order.
    pub(crate) fn push(&self, text: &str, layout: TextLayout, links: &[(Range<usize>, String)]) {
        self.texts.borrow_mut().push(Drawn {
            lead: self.lead.take(),
            text: text.to_owned(),
            layout,
            links: links.to_vec(),
        });
    }

    /// Sets the marker the next text copies with; empty drops it.
    pub(crate) fn lead(&self, lead: String) {
        *self.lead.borrow_mut() = lead;
    }

    /// Keeps this frame's texts; other texts drop the selection.
    pub(crate) fn finish(self, cx: &mut App) -> Entity<Selection> {
        let texts = self.texts.into_inner();
        self.state.update(cx, |selection, cx| {
            let same = selection.texts.len() == texts.len()
                && (selection.texts.iter().zip(&texts)).all(|(old, new)| old.text == new.text);
            if !same {
                selection.anchor = Spot::default();
                selection.head = Spot::default();
                selection.dragging = false;
                cx.notify();
            }
            selection.texts = texts;
        });
        self.state
    }
}

/// A right press on a picture, its selection kept.
pub(crate) fn picture_press(
    state: &Entity<Selection>,
    on_menu: &OnMenu,
    (image, link): (String, Option<String>),
    position: Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let selected = state.read(cx).selected();
    let press = RightPress {
        selected,
        link,
        image: Some(image),
        position,
    };
    on_menu(press, window, cx);
}

/// `body` selects by drag, copies on the secondary C and selects all on A.
pub(crate) fn selectable(body: Div, state: Entity<Selection>, on_menu: OnMenu, cx: &App) -> Div {
    let focus = state.read(cx).focus.clone();
    let (down, moved, up, out, right) = (
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
    );
    body.track_focus(&focus)
        .cursor(CursorStyle::IBeam)
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            down.update(cx, |selection, cx| {
                window.focus(&selection.focus, cx);
                let Some(spot) = selection.spot(event.position) else {
                    return;
                };
                if !event.modifiers.shift {
                    selection.anchor = spot;
                }
                selection.head = spot;
                selection.dragging = true;
                cx.notify();
            })
        })
        .on_mouse_move(move |event, _, cx| {
            moved.update(cx, |selection, cx| {
                if !selection.dragging || !event.dragging() {
                    return;
                }
                if let Some(spot) = selection.spot(event.position)
                    && spot != selection.head
                {
                    selection.head = spot;
                    cx.notify();
                }
            })
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            up.update(cx, |selection, _| selection.dragging = false)
        })
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
            out.update(cx, |selection, _| selection.dragging = false)
        })
        .on_mouse_down(MouseButton::Right, move |event, window, cx| {
            let selection = right.read(cx);
            let link = selection
                .spot(event.position)
                .and_then(|spot| selection.link(spot));
            let press = RightPress {
                selected: selection.selected(),
                link,
                image: None,
                position: event.position,
            };
            on_menu(press, window, cx);
        })
        .on_key_down(move |event, _, cx| {
            if !event.keystroke.modifiers.secondary(cx) {
                return;
            }
            match event.keystroke.key.as_str() {
                "c" => {
                    let Some(text) = state.read(cx).selected() else {
                        return;
                    };
                    log::info!("markdown: a selection copied");
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                    cx.stop_propagation();
                }
                "a" => {
                    state.update(cx, |selection, cx| {
                        let Some(last) = selection.texts.last() else {
                            return;
                        };
                        selection.anchor = Spot::default();
                        selection.head = Spot {
                            text: selection.texts.len() - 1,
                            at: last.text.len(),
                        };
                        cx.notify();
                    });
                    cx.stop_propagation();
                }
                _ => {}
            }
        })
}

#[cfg(test)]
mod tests;
