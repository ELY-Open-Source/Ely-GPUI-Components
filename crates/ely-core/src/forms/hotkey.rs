use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, Keystroke, ParentElement, RenderOnce, Role,
    SharedString, Styled, Window, div, prelude::*,
};

use crate::{
    primitives::{FocusRing, Icon, IconName},
    theme::{ActiveTheme, ControlSize, IconSize, Radius, TextSize},
    typography::{KbdCombo, keys::os},
};

type OnChange = Rc<dyn Fn(Option<Keystroke>, &mut Window, &mut App)>;

/// Records one shortcut, then lets go. Backspace clears; Escape leaves.
///
/// ```
/// use ely_core::forms::HotkeyInput;
/// use gpui::Keystroke;
///
/// let save = HotkeyInput::new("save", Keystroke::parse("cmd-s").ok()).on_change(|_keystroke, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct HotkeyInput {
    id: ElementId,
    label: Option<SharedString>,
    value: Option<Keystroke>,
    on_change: Option<OnChange>,
}

impl HotkeyInput {
    /// A recorder under `id` holding `value`.
    pub fn new(id: impl Into<ElementId>, value: Option<Keystroke>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value,
            on_change: None,
        }
    }

    /// The name assistive technology reads; nothing is drawn.
    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Runs with the keystroke recorded, or `None` once Backspace or Delete clears it.
    pub fn on_change(
        mut self,
        handler: impl Fn(Option<Keystroke>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for HotkeyInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (recording_words, none_words) = (
            crate::i18n::text(cx, "hotkey.recording", &[]),
            crate::i18n::text(cx, "hotkey.none", &[]),
        );
        let focus = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let recording = focus.is_focused(window);
        let theme = cx.theme();
        let colors = &theme.colors;
        let os = os(theme.platform);
        let on_change = self.on_change;
        div()
            .id(self.id)
            .role(Role::TextInput)
            .when_some(self.label.clone(), |field, label| field.aria_label(label))
            .when_some(self.value.as_ref(), |field, stroke| {
                field.aria_value(stroke.unparse(os))
            })
            .track_focus(&focus)
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .h(theme.control_height(ControlSize::Md))
            .px(theme.control_padding(ControlSize::Md))
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(colors.border_strong)
            .bg(colors.surface)
            .focus_ring(cx)
            .cursor_pointer()
            .text_size(theme.text_size(TextSize::Base))
            .on_key_down(move |event, window, cx| {
                cx.stop_propagation();
                let stroke = &event.keystroke;
                let bare = !stroke.modifiers.modified();
                let next = match stroke.key.as_str() {
                    "escape" if bare => {
                        window.blur(cx);
                        return;
                    }
                    "backspace" | "delete" if bare => None,
                    _ => Some(Keystroke {
                        key_char: None,
                        ..stroke.clone()
                    }),
                };
                log::info!(
                    "hotkey input: {}",
                    next.as_ref()
                        .map_or("cleared".into(), |stroke| stroke.unparse(os))
                );
                window.blur(cx);
                if let Some(on_change) = &on_change {
                    on_change(next, window, cx);
                }
            })
            .map(|field| match (&self.value, recording) {
                (_, true) => field.child(div().text_color(colors.fg_subtle).child(recording_words)),
                (Some(stroke), false) => field.child(KbdCombo::new(&stroke.unparse(os))),
                (None, false) => field.child(div().text_color(colors.fg_subtle).child(none_words)),
            })
            .child(
                Icon::new(IconName::Keyboard)
                    .size(IconSize::Sm)
                    .color(colors.fg_subtle),
            )
    }
}

crate::capable!(HotkeyInput: change(Option<Keystroke>));
