use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FontWeight, IntoElement, ParentElement, RenderOnce, Role,
    SharedString, Styled, Window, div, prelude::*,
};
use jiff::Timestamp;
use smallvec::SmallVec;

use crate::{
    buttons::{ButtonVariant, IconButton},
    data_display::{Avatar, AvatarGroup},
    forms::Run,
    layout::on_axis,
    primitives::{Divider, FocusRing, IconName},
    theme::{ActiveTheme, AvatarSize, ControlSize, Radius, TextSize},
    typography::Ellipsis,
};

/// A thread's mark under its first message: who replied, how many replies, and when the last came. With a handler, a press opens the thread.
///
/// ```
/// use ely_messaging::MessageThread;
/// use jiff::Timestamp;
///
/// let last = Timestamp::from_second(1_700_000_000).unwrap();
/// let thread = MessageThread::new("thread", 4, last).on_open(|_, _| {});
/// ```
#[derive(IntoElement)]
pub struct MessageThread {
    id: ElementId,
    replies: usize,
    last: Timestamp,
    repliers: Vec<Avatar>,
    on_open: Option<Run>,
}

impl MessageThread {
    /// A mark under `id` for `replies` replies, the last at `last`; panics on none.
    pub fn new(id: impl Into<ElementId>, replies: usize, last: Timestamp) -> Self {
        assert!(replies > 0, "a thread holds a reply");
        Self {
            id: id.into(),
            replies,
            last,
            repliers: Vec::new(),
            on_open: None,
        }
    }

    /// Who replied; the first three show.
    pub fn repliers(mut self, repliers: impl IntoIterator<Item = Avatar>) -> Self {
        self.repliers = repliers.into_iter().collect();
        assert!(
            self.repliers.len() <= self.replies,
            "more repliers than replies"
        );
        self
    }

    /// What a press does: open the thread.
    pub fn on_open(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for MessageThread {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let count = replies(self.replies, cx);
        let last =
            crate::typography::relative_text(cx, self.last, ely_core::typography::format::now());
        let when = crate::i18n::text(cx, "messaging.reply.last", &[("when", &last)]);
        let id = self.id.clone();
        div()
            .id(self.id)
            .debug_selector(|| "message-thread".into())
            .flex()
            .items_center()
            .gap_2()
            .px_1p5()
            .py_1()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(gpui::transparent_black())
            .text_size(theme.text_size(TextSize::Sm))
            .when_some(self.on_open, |row, open| {
                row.role(Role::Button)
                    .aria_label(count.clone())
                    .cursor_pointer()
                    .tab_index(0)
                    .focus_ring(cx)
                    .hover(|style| style.bg(colors.surface))
                    .on_click(move |_, window, cx| {
                        log::info!("message thread {id}: open");
                        open(window, cx)
                    })
            })
            .when(!self.repliers.is_empty(), |row| {
                row.child(
                    div()
                        .flex_none()
                        .child(AvatarGroup::new(self.repliers).max(3).size(AvatarSize::Xs)),
                )
            })
            .child(
                div()
                    .flex_none()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.link)
                    .child(count),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_color(colors.fg_subtle)
                    .child(Ellipsis::new(when)),
            )
    }
}

/// A thread beside its chat: a head with the chat's name and a close, the first message, a rule counting the replies, the replies, and a composer held at the bottom. The body scrolls; the host gives the panel its size.
///
/// ```
/// use ely_messaging::ThreadPanel;
/// use gpui::ParentElement;
///
/// let panel = ThreadPanel::new("panel", "Release on Friday?")
///     .chat("general")
///     .child("Yes, after the review.")
///     .on_close(|_, _| {});
/// ```
#[derive(IntoElement)]
pub struct ThreadPanel {
    id: ElementId,
    root: AnyElement,
    chat: Option<SharedString>,
    replies: SmallVec<[AnyElement; 4]>,
    composer: Option<AnyElement>,
    on_close: Option<Run>,
}

impl ThreadPanel {
    /// A panel under `id` over the first message, `root`.
    pub fn new(id: impl Into<ElementId>, root: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            root: root.into_any_element(),
            chat: None,
            replies: SmallVec::new(),
            composer: None,
            on_close: None,
        }
    }

    /// The chat the thread is in, after the title.
    pub fn chat(mut self, name: impl Into<SharedString>) -> Self {
        self.chat = Some(name.into());
        self
    }

    /// The field for a reply, held at the bottom.
    pub fn composer(mut self, composer: impl IntoElement) -> Self {
        self.composer = Some(composer.into_any_element());
        self
    }

    /// Shows the close, and what a press on it does.
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for ThreadPanel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.replies.extend(elements);
    }
}

impl RenderOnce for ThreadPanel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let count = self.replies.len();
        let close = self.on_close.map(|close| {
            let id = self.id.clone();
            IconButton::new((self.id.clone(), "close"), IconName::X)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Sm)
                .tooltip("Close thread")
                .on_click(move |_, window, cx| {
                    log::info!("thread panel {id}: close");
                    close(window, cx)
                })
        });
        let head = div()
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_4()
            .py_2()
            .border_b_1()
            .border_color(colors.border)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_baseline()
                    .gap_2()
                    .child(
                        div()
                            .flex_none()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.fg)
                            .child("Thread"),
                    )
                    .children(self.chat.map(|chat| {
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(theme.text_size(TextSize::Sm))
                            .text_color(colors.fg_muted)
                            .child(Ellipsis::new(chat))
                    })),
            )
            .children(close.map(|close| div().flex_none().child(close)));
        let rule = (count > 0).then(|| {
            div()
                .debug_selector(|| "thread-replies".into())
                .px_3()
                .py_1()
                .child(Divider::horizontal().label(replies(count, cx)))
        });
        let body = on_axis(
            div()
                .id((self.id.clone(), "body"))
                .flex_1()
                .min_h_0()
                .overflow_y_scroll(),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .py_2()
                .child(self.root)
                .children(rule)
                .children(self.replies),
        );
        div()
            .debug_selector(|| "thread-panel".into())
            .size_full()
            .flex()
            .flex_col()
            .child(head)
            .child(body)
            .children(self.composer.map(|composer| {
                div()
                    .flex_none()
                    .p_3()
                    .border_t_1()
                    .border_color(colors.border)
                    .child(composer)
            }))
    }
}

/// `count` replies, counted in the shown locale.
fn replies(count: usize, cx: &App) -> SharedString {
    let form = if count == 1 { "one" } else { "other" };
    let n = count.to_string();
    crate::i18n::text(cx, &format!("messaging.reply.{form}"), &[("n", &n)])
}
