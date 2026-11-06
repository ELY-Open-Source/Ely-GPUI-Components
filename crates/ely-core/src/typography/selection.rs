use std::{collections::BTreeMap, ops::Range, rc::Rc};

use gpui::{
    App, Bounds, ClipboardItem, Context, DispatchPhase, Entity, FocusHandle, InteractiveElement,
    IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels,
    Point, ScrollStrategy, SharedString, StatefulInteractiveElement, Styled, StyledText,
    TextLayout, UniformListScrollHandle, Window, canvas, div,
};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Position {
    row: usize,
    byte: usize,
}

#[doc(hidden)]
pub struct TextSelection {
    lines: Rc<Vec<Option<SharedString>>>,
    focus: FocusHandle,
    _blur: gpui::Subscription,
    anchor: Position,
    head: Position,
    dragging: bool,
    moved: bool,
    extend_shift: bool,
    scroll: Option<UniformListScrollHandle>,
    viewport: Option<Bounds<Pixels>>,
    pointer: Option<Point<Pixels>>,
    scrolling: bool,
    layouts: BTreeMap<usize, (TextLayout, Bounds<Pixels>)>,
}

impl TextSelection {
    fn clear(&mut self, cx: &mut Context<Self>) {
        let changed = self.anchor != self.head || self.dragging;
        self.anchor = Position::default();
        self.head = self.anchor;
        self.dragging = false;
        self.pointer = None;
        if changed {
            cx.notify();
        }
    }

    fn ends(&self) -> (Position, Position) {
        (self.anchor.min(self.head), self.anchor.max(self.head))
    }

    pub(crate) fn range(&self, row: usize) -> Range<usize> {
        let (start, end) = self.ends();
        let len = self
            .lines
            .get(row)
            .and_then(Option::as_ref)
            .map_or(0, |s| s.len());
        if row < start.row || row > end.row {
            return 0..0;
        }
        let from = if row == start.row {
            start.byte.min(len)
        } else {
            0
        };
        let to = if row == end.row {
            end.byte.min(len)
        } else {
            len
        };
        from..to
    }

    fn copied(&self) -> String {
        let (start, end) = self.ends();
        (start.row..=end.row)
            .filter_map(|row| {
                let text = self.lines.get(row)?.as_ref()?;
                text.get(self.range(row)).map(str::to_string)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn at(&self, point: Point<Pixels>) -> Option<Position> {
        let (&row, (layout, bounds)) = self.layouts.iter().min_by(|a, b| {
            let distance = |bounds: &Bounds<Pixels>| {
                (bounds.top() - point.y)
                    .max(point.y - bounds.bottom())
                    .max(Pixels::ZERO)
            };
            distance(&a.1.1)
                .partial_cmp(&distance(&b.1.1))
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
        let byte = if point.y < bounds.top() {
            0
        } else if point.y > bounds.bottom() {
            self.lines[row].as_ref()?.len()
        } else {
            match layout.index_for_position(point) {
                Ok(at) | Err(at) => at,
            }
        };
        Some(Position { row, byte })
    }
}

#[doc(hidden)]
pub fn state(
    id: impl Into<gpui::ElementId>,
    lines: Rc<Vec<Option<SharedString>>>,
    extend_shift: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TextSelection> {
    let state = window.use_keyed_state(id.into(), cx, |window, cx| {
        let focus = cx.focus_handle();
        let blur = cx.on_blur(&focus, window, |s: &mut TextSelection, _, cx| s.clear(cx));
        TextSelection {
            lines: lines.clone(),
            focus,
            _blur: blur,
            anchor: Position::default(),
            head: Position::default(),
            dragging: false,
            moved: false,
            extend_shift,
            scroll: None,
            viewport: None,
            pointer: None,
            scrolling: false,
            layouts: BTreeMap::new(),
        }
    });
    state.update(cx, |s, _| {
        if s.lines != lines {
            s.lines = lines;
            s.anchor = Position::default();
            s.head = Position::default();
            s.dragging = false;
            s.moved = false;
            s.pointer = None;
        }
        s.layouts.clear();
    });
    state
}

#[doc(hidden)]
pub fn scroll(state: &Entity<TextSelection>, handle: UniformListScrollHandle, cx: &mut App) {
    state.update(cx, |s, _| s.scroll = Some(handle));
}

impl TextSelection {
    fn scroll_step(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.dragging {
            return false;
        }
        let (Some(scroll), Some(bounds), Some(point)) = (&self.scroll, self.viewport, self.pointer)
        else {
            return false;
        };
        let (row, strategy, byte) = if point.y >= bounds.bottom() {
            let row = self
                .layouts
                .keys()
                .next_back()
                .copied()
                .unwrap_or(self.head.row)
                .max(self.head.row)
                .saturating_add(1)
                .min(self.lines.len().saturating_sub(1));
            let byte = self
                .lines
                .get(row)
                .and_then(Option::as_ref)
                .map_or(0, |line| line.len());
            (row, ScrollStrategy::Bottom, byte)
        } else if point.y <= bounds.top() {
            (
                self.layouts
                    .keys()
                    .next()
                    .copied()
                    .unwrap_or(self.head.row)
                    .min(self.head.row)
                    .saturating_sub(1),
                ScrollStrategy::Top,
                0,
            )
        } else {
            return false;
        };
        if self.head == (Position { row, byte }) {
            return false;
        }
        scroll.scroll_to_item(row, strategy);
        self.head = Position { row, byte };
        self.moved = true;
        cx.notify();
        true
    }

    fn start_scroll(&mut self, cx: &mut Context<Self>) {
        if self.scrolling || !self.scroll_step(cx) {
            return;
        }
        self.scrolling = true;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(40))
                    .await;
                let more = this
                    .update(cx, |s, cx| {
                        let more = s.scroll_step(cx);
                        if !more {
                            s.scrolling = false;
                        }
                        more
                    })
                    .unwrap_or(false);
                if !more {
                    break;
                }
            }
        })
        .detach();
    }
}

#[doc(hidden)]
pub fn range(state: &Entity<TextSelection>, row: usize, cx: &App) -> Range<usize> {
    state.read(cx).range(row)
}

#[doc(hidden)]
pub fn fragment(
    id: impl Into<gpui::ElementId>,
    state: Entity<TextSelection>,
    row: usize,
    styled: StyledText,
) -> gpui::Stateful<gpui::Div> {
    let layout = styled.layout().clone();
    let (down, click, paint) = (state.clone(), state.clone(), state);
    div()
        .id(id)
        .debug_selector(move || format!("selection-text-{row}"))
        .relative()
        .cursor(gpui::CursorStyle::IBeam)
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            down.update(cx, |s, cx| {
                let byte = match layout.index_for_position(event.position) {
                    Ok(at) | Err(at) => at,
                };
                let at = Position { row, byte };
                if !s.extend_shift || !event.modifiers.shift {
                    s.anchor = at;
                }
                s.head = at;
                s.pointer = Some(event.position);
                s.dragging = true;
                s.moved = false;
                window.focus(&s.focus, cx);
                window.prevent_default();
                cx.notify();
            });
        })
        .on_click(move |_, _, cx| {
            if click.read(cx).moved {
                cx.stop_propagation();
            }
        })
        .child({
            let layout = styled.layout().clone();
            canvas(
                |_, _, _| (),
                move |bounds, _, _, cx| {
                    paint.update(cx, |s, _| {
                        s.layouts.insert(row, (layout, bounds));
                    });
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full()
        })
        .child(styled)
}

#[doc(hidden)]
pub fn surface(
    id: impl Into<gpui::ElementId>,
    state: Entity<TextSelection>,
    child: impl IntoElement,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    let focus = state.read(cx).focus.clone();
    let (keys, pointer) = (state.clone(), state);
    div()
        .id(id)
        .relative()
        .track_focus(&focus)
        .on_key_down(move |event, _, cx| {
            let key = &event.keystroke;
            if !key.modifiers.secondary(cx) || key.modifiers.alt {
                return;
            }
            match key.key.as_str() {
                "c" => {
                    let text = keys.read(cx).copied();
                    if !text.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(text));
                    }
                    cx.stop_propagation();
                }
                "a" => {
                    keys.update(cx, |s, cx| {
                        if let Some((row, Some(text))) =
                            s.lines.iter().enumerate().rfind(|(_, t)| t.is_some())
                        {
                            s.anchor = Position::default();
                            s.head = Position {
                                row,
                                byte: text.len(),
                            };
                            cx.notify();
                        }
                    });
                    cx.stop_propagation();
                }
                _ => {}
            }
        })
        .child(child)
        .child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, cx| {
                    pointer.update(cx, |s, _| s.viewport = Some(bounds));
                    let outside = pointer.clone();
                    window.on_mouse_event(move |event: &MouseDownEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture && event.button == MouseButton::Left {
                            outside.update(cx, |s, cx| {
                                if !s
                                    .viewport
                                    .is_some_and(|viewport| viewport.contains(&event.position))
                                    || !s
                                        .layouts
                                        .values()
                                        .any(|(_, bounds)| bounds.contains(&event.position))
                                {
                                    s.clear(cx);
                                }
                            });
                        }
                    });
                    let moved = pointer.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                        if phase != DispatchPhase::Bubble
                            || !event.dragging()
                            || !moved.read(cx).dragging
                        {
                            return;
                        }
                        moved.update(cx, |s, cx| {
                            s.pointer = Some(event.position);
                            if let Some(at) = s.at(event.position) {
                                if s.head != at {
                                    s.head = at;
                                    s.moved = true;
                                    cx.notify();
                                }
                            }
                            s.start_scroll(cx);
                        });
                    });
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                        if phase == DispatchPhase::Bubble && event.button == MouseButton::Left {
                            let suppress = pointer.update(cx, |s, _| {
                                let suppress = s.dragging && s.moved;
                                s.dragging = false;
                                suppress
                            });
                            if suppress {
                                cx.stop_propagation();
                            }
                        }
                    });
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
        )
}
