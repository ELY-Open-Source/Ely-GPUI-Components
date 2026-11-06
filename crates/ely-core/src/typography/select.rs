use std::ops::Range;

use gpui::{
    App, ClipboardItem, Context, CursorStyle, DispatchPhase, ElementId, FocusHandle,
    HighlightStyle, InteractiveElement, IntoElement, MouseButton, MouseMoveEvent, MouseUpEvent,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, StyledText,
    TextLayout, Window, canvas, div, prelude::*,
};

use crate::theme::ActiveTheme;

struct Selection {
    text: SharedString,
    focus: FocusHandle,
    anchor: usize,
    head: usize,
    dragging: bool,
    moved: bool,
    _blur: gpui::Subscription,
}

impl Selection {
    fn range(&self) -> Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        if self.anchor != self.head || self.dragging {
            self.anchor = 0;
            self.head = 0;
            self.dragging = false;
            cx.notify();
        }
    }
}

fn index_at(layout: &TextLayout, position: gpui::Point<gpui::Pixels>) -> usize {
    match layout.index_for_position(position) {
        Ok(ix) | Err(ix) => ix,
    }
}

/// Read-only text the pointer can select and copy.
///
/// ```
/// use ely_core::typography::SelectableText;
///
/// let log = SelectableText::new("log", "error: the build failed");
/// ```
#[derive(IntoElement)]
pub struct SelectableText {
    id: ElementId,
    text: SharedString,
    ellipsis: bool,
}

impl SelectableText {
    /// Selectable `text` under `id`.
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            ellipsis: false,
        }
    }

    /// Shows an ellipsis until text is selected.
    pub fn ellipsis(mut self) -> Self {
        self.ellipsis = true;
        self
    }
}

impl RenderOnce for SelectableText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |window, cx| {
            let focus = cx.focus_handle();
            let blur = cx.on_blur(&focus, window, |s: &mut Selection, _, cx| s.clear(cx));
            Selection {
                text: self.text.clone(),
                focus,
                anchor: 0,
                head: 0,
                dragging: false,
                moved: false,
                _blur: blur,
            }
        });
        if state.read(cx).text != self.text {
            state.update(cx, |selection, _| {
                selection.text = self.text.clone();
                selection.anchor = 0;
                selection.head = 0;
                selection.dragging = false;
            });
        }
        let (focus, range) = {
            let selection = state.read(cx);
            (selection.focus.clone(), selection.range())
        };
        let elide = self.ellipsis && range.is_empty();
        let highlight = HighlightStyle {
            background_color: (!range.is_empty()).then_some(cx.theme().colors.selection),
            fade_out: elide.then_some(1.0),
            ..Default::default()
        };
        let styled = StyledText::new(self.text.clone()).with_highlights(
            (elide || !range.is_empty())
                .then_some((if elide { 0..self.text.len() } else { range }, highlight)),
        );
        let layout = styled.layout().clone();
        let (down, outside, drag, up, keys) = (
            state.clone(),
            state.clone(),
            state.clone(),
            state.clone(),
            state,
        );
        let (down_layout, drag_layout) = (layout.clone(), layout);

        div()
            .id(self.id)
            .debug_selector(|| "selection-text-0".into())
            .relative()
            .role(Role::Paragraph)
            .aria_label(self.text.clone())
            .cursor(CursorStyle::IBeam)
            .when(self.ellipsis, |text| {
                text.min_w_0().overflow_hidden().whitespace_nowrap()
            })
            .when(elide, |text| {
                text.debug_selector(|| "selection-ellipsis".into())
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                let ix = index_at(&down_layout, event.position);
                down.update(cx, |selection, cx| {
                    window.focus(&selection.focus, cx);
                    if !event.modifiers.shift {
                        selection.anchor = ix;
                    }
                    selection.head = ix;
                    selection.dragging = true;
                    selection.moved = false;
                    window.prevent_default();
                    cx.notify();
                });
            })
            .on_mouse_down_out(move |event, _, cx| {
                if event.button == MouseButton::Left {
                    outside.update(cx, |s, cx| s.clear(cx));
                }
            })
            .when(focus.is_focused(window), |element| {
                element
                    .track_focus(&focus)
                    .on_key_down(move |event, _, cx| {
                        let stroke = &event.keystroke;
                        if !stroke.modifiers.secondary(cx) || stroke.modifiers.alt {
                            return;
                        }
                        match stroke.key.as_str() {
                            "c" => {
                                let range = keys.read(cx).range();
                                if !range.is_empty() {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        keys.read(cx).text[range].to_string(),
                                    ));
                                }
                            }
                            "a" => keys.update(cx, |selection, cx| {
                                selection.anchor = 0;
                                selection.head = selection.text.len();
                                cx.notify();
                            }),
                            _ => {}
                        }
                        if matches!(stroke.key.as_str(), "a" | "c") {
                            cx.stop_propagation();
                        }
                    })
            })
            .child(styled)
            .when(elide, |text| text.child(super::fit::end_overlay(self.text)))
            .child(
                canvas(
                    |_, _, _| (),
                    move |_, _, window, _| {
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                            if phase != DispatchPhase::Bubble
                                || !event.dragging()
                                || !drag.read(cx).dragging
                            {
                                return;
                            }
                            let head = index_at(&drag_layout, event.position);
                            drag.update(cx, |s, cx| {
                                if s.head != head {
                                    s.head = head;
                                    s.moved = true;
                                    cx.notify();
                                }
                            });
                        });
                        window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                            if phase == DispatchPhase::Bubble && event.button == MouseButton::Left {
                                let suppress = up.update(cx, |s, _| {
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
                .size_full()
                .top_0()
                .left_0(),
            )
    }
}
