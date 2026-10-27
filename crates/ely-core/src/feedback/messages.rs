use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, AnyElement, App, Div, ElementId, FontWeight, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, Styled, Window, div, prelude::*,
};
use smallvec::SmallVec;

use crate::{
    buttons::{Button, ButtonVariant, IconButton},
    forms::Run,
    motion,
    primitives::{Icon, IconName, Severity},
    theme::{ActiveTheme, ControlSize, IconSize, Radius, TextSize},
};

/// A ghost close button that keeps its row's height.
fn dismiss(id: ElementId, run: Run, cx: &App) -> impl IntoElement {
    div().my_neg_1().child(
        IconButton::new(id, IconName::X)
            .tooltip(crate::i18n::text(cx, "message.dismiss", &[]))
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Sm)
            .on_click(move |_, window, cx| run(window, cx)),
    )
}

/// A message in a page's flow: the severity's icon on its tint, a title, a line, actions and a close button.
///
/// ```
/// use ely_core::{buttons::Button, feedback::Alert, primitives::Severity};
///
/// let quota = Alert::new("quota", Severity::Warning, "Storage almost full")
///     .body("You have used 92% of your space.")
///     .action(Button::new("upgrade", "Upgrade"))
///     .on_dismiss(|_, _| {});
/// ```
#[derive(IntoElement)]
pub struct Alert {
    id: ElementId,
    severity: Severity,
    title: SharedString,
    body: Option<SharedString>,
    actions: SmallVec<[AnyElement; 2]>,
    on_dismiss: Option<Run>,
}

impl Alert {
    /// An alert under `id` with `severity`'s color and icon, titled `title`.
    pub fn new(
        id: impl Into<ElementId>,
        severity: Severity,
        title: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            severity,
            title: title.into(),
            body: None,
            actions: SmallVec::new(),
            on_dismiss: None,
        }
    }

    /// A line under the title.
    pub fn body(mut self, text: impl Into<SharedString>) -> Self {
        self.body = Some(text.into());
        self
    }

    /// Adds an action at its end.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    /// Shows a close button that runs `handler`.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Alert {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let close = (self.id.clone(), "dismiss");
        // Assistive tech reads it as a banner does.
        let role = match self.severity {
            Severity::Danger | Severity::Warning => Role::Alert,
            Severity::Info | Severity::Success => Role::Status,
        };
        let label = match &self.body {
            Some(body) => format!("{}. {body}", self.title),
            None => self.title.to_string(),
        };
        div()
            .id(self.id)
            .role(role)
            .aria_label(label)
            .flex()
            .items_start()
            .gap_3()
            .p_3()
            .rounded(theme.radius(Radius::Lg))
            .bg(self.severity.subtle(colors))
            .text_size(theme.text_size(TextSize::Base))
            .child(
                div().mt_0p5().child(
                    Icon::new(self.severity.icon())
                        .size(IconSize::Md)
                        .color(self.severity.color(colors)),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.fg)
                            .child(self.title),
                    )
                    .when_some(self.body, |text, body| {
                        text.child(div().text_color(colors.fg_muted).child(body))
                    })
                    .when(!self.actions.is_empty(), |text| {
                        text.child(div().flex().gap_2().pt_1().children(self.actions))
                    }),
            )
            .when_some(self.on_dismiss, |alert, run| {
                alert.child(dismiss(close.into(), run, cx))
            })
    }
}

/// A strip across the top of a window or page: an icon, one line, a link and a close button.
///
/// ```
/// use ely_core::{feedback::Banner, primitives::Severity};
///
/// let outage = Banner::new("outage", Severity::Danger, "Payments are delayed.")
///     .action("Status page", |_, _| {})
///     .on_dismiss(|_, _| {});
/// ```
#[derive(IntoElement)]
pub struct Banner {
    id: ElementId,
    severity: Severity,
    message: SharedString,
    action: Option<(SharedString, Run)>,
    on_dismiss: Option<Run>,
}

impl Banner {
    /// A bar under `id` with `severity`'s color that reads `message`.
    pub fn new(
        id: impl Into<ElementId>,
        severity: Severity,
        message: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            severity,
            message: message.into(),
            action: None,
            on_dismiss: None,
        }
    }

    /// A button at its end that reads `label` and runs `handler`.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(handler)));
        self
    }

    /// Shows a close button that runs `handler`.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Banner {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let (link, close) = ((self.id.clone(), "action"), (self.id.clone(), "dismiss"));
        // Assistive tech reads the message; danger interrupts.
        let role = match self.severity {
            Severity::Danger | Severity::Warning => Role::Alert,
            Severity::Info | Severity::Success => Role::Status,
        };
        div()
            .id(self.id)
            .role(role)
            .aria_label(self.message.clone())
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .min_h(theme.control_height(ControlSize::Lg))
            .py_1()
            .px_10()
            .bg(self.severity.subtle(colors))
            .border_b_1()
            .border_color(colors.border)
            .text_size(theme.text_size(TextSize::Base))
            .text_color(colors.fg)
            .child(
                div().flex_none().child(
                    Icon::new(self.severity.icon())
                        .size(IconSize::Sm)
                        .color(self.severity.color(colors)),
                ),
            )
            // Narrow, the message wraps and the action goes below it.
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .flex_shrink_1()
                    .min_w_0()
                    .items_center()
                    .gap_x_2()
                    .child(div().min_w_0().child(self.message))
                    .when_some(self.action, |text, (label, run)| {
                        text.child(
                            Button::new(link, label)
                                .variant(ButtonVariant::Link)
                                .on_click(move |_, window, cx| run(window, cx)),
                        )
                    }),
            )
            .when_some(self.on_dismiss, |banner, run| {
                banner.child(
                    div()
                        .absolute()
                        .right_1()
                        .top_0()
                        .bottom_0()
                        .flex()
                        .items_center()
                        .child(dismiss(close.into(), run, cx)),
                )
            })
    }
}

/// A note set apart in reading: a rule in the severity's color, a labeled icon, then the text.
///
/// ```
/// use ely_core::{feedback::Callout, primitives::Severity};
/// use gpui::ParentElement;
///
/// let tip = Callout::new(Severity::Info).title("Tip").child("Hold Shift to select a range.");
/// ```
#[derive(IntoElement)]
pub struct Callout {
    severity: Severity,
    title: SharedString,
    body: SmallVec<[AnyElement; 2]>,
}

impl Callout {
    /// Titled Note, Tip, Warning or Danger until `title` says otherwise.
    pub fn new(severity: Severity) -> Self {
        let title = match severity {
            Severity::Info => "Note",
            Severity::Success => "Tip",
            Severity::Warning => "Warning",
            Severity::Danger => "Danger",
        };
        Self {
            severity,
            title: title.into(),
            body: SmallVec::new(),
        }
    }

    /// Its title.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }
}

impl ParentElement for Callout {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

impl RenderOnce for Callout {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let tone = self.severity.color(colors);
        div()
            .flex()
            .flex_col()
            .gap_1()
            .py_1()
            .pl_4()
            .border_l_2()
            .border_color(tone)
            .text_size(theme.text_size(TextSize::Base))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(tone)
                    .child(
                        Icon::new(self.severity.icon())
                            .size(IconSize::Sm)
                            .color(tone),
                    )
                    .child(self.title),
            )
            .child(div().text_color(colors.fg).children(self.body))
    }
}

/// A line beside what it is about, in the severity's color; a long one wraps.
///
/// ```
/// use ely_core::{feedback::InlineMessage, primitives::Severity};
///
/// let taken = InlineMessage::new(Severity::Danger, "That name is taken.");
/// ```
#[derive(IntoElement)]
pub struct InlineMessage {
    severity: Severity,
    text: SharedString,
}

impl InlineMessage {
    /// One line in `severity`'s color and icon that reads `text`.
    pub fn new(severity: Severity, text: impl Into<SharedString>) -> Self {
        Self {
            severity,
            text: text.into(),
        }
    }
}

impl RenderOnce for InlineMessage {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let tone = self.severity.color(&theme.colors);
        div()
            .flex()
            .items_center()
            .gap_1()
            .text_size(theme.text_size(TextSize::Sm))
            .text_color(tone)
            .child(
                Icon::new(self.severity.icon())
                    .size(IconSize::Xs)
                    .color(tone),
            )
            .child(
                div()
                    .debug_selector(|| "inline-message-text".into())
                    .flex_1()
                    .min_w_0()
                    .child(self.text),
            )
    }
}

/// A line that says what just happened. Each new message rises in; the first stays still.
///
/// ```
/// use ely_core::{feedback::StatusMessage, primitives::IconName};
///
/// let copied = StatusMessage::new("copied", "Copied to the clipboard").icon(IconName::Check);
/// ```
#[derive(IntoElement)]
pub struct StatusMessage {
    id: ElementId,
    text: SharedString,
    icon: Option<IconName>,
}

impl StatusMessage {
    /// A line under `id` that reads `text`.
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            icon: None,
        }
    }

    /// An icon before the text.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl RenderOnce for StatusMessage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let muted = theme.colors.fg_muted;
        let line = div()
            .flex()
            .items_center()
            .gap_1p5()
            .text_size(theme.text_size(TextSize::Sm))
            .text_color(muted)
            .when_some(self.icon, |line, icon| {
                line.child(Icon::new(icon).size(IconSize::Xs).color(muted))
            })
            .child(self.text.clone());
        rise(self.id, self.text, line, window, cx)
    }
}

/// `line`, rising in each time `key` changes; still on first paint.
pub(super) fn rise<T: Clone + PartialEq + 'static>(
    id: ElementId,
    key: T,
    line: Div,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<T> {
    let turn = motion::changes((id.clone(), "key"), key, window, cx);
    line.with_animation(
        (id, format!("turn-{turn}")),
        Animation::new(motion::duration(motion::BASE, cx)).with_easing(motion::ease_out_cubic),
        move |line, t| {
            if turn == 0 {
                line
            } else {
                line.opacity(t).mt(motion::NUDGE * (1.0 - t))
            }
        },
    )
}
