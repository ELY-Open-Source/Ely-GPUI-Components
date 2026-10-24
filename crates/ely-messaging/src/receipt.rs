use gpui::{
    App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, prelude::*,
};

use crate::{
    data_display::{Avatar, AvatarGroup},
    primitives::{Icon, IconName},
    theme::{ActiveTheme, AvatarSize, IconSize, TextSize},
};

/// Where a sent message stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    /// On its way.
    Sending,
    /// Sent.
    Sent,
    /// Delivered.
    Delivered,
    /// Read.
    Read,
}

impl Delivery {
    /// Its word in the shown locale.
    pub fn label(self, cx: &App) -> SharedString {
        let key = match self {
            Self::Sending => "messaging.delivery.sending",
            Self::Sent => "messaging.delivery.sent",
            Self::Delivered => "messaging.delivery.delivered",
            Self::Read => "messaging.delivery.read",
        };
        crate::i18n::text(cx, key, &[])
    }

    fn icon(self) -> IconName {
        match self {
            Self::Sending => IconName::Clock,
            Self::Sent => IconName::Check,
            Self::Delivered | Self::Read => IconName::CheckCheck,
        }
    }
}

/// A sent message's receipt: a clock while it goes, one tick once sent, two once delivered, two in the info tone once read, each with its word. Read in a group, it shows who saw it.
///
/// ```
/// use ely_core::data_display::Avatar;
/// use ely_messaging::{Delivery, ReadReceipt};
///
/// let receipt = ReadReceipt::new(Delivery::Read).readers([Avatar::new("ben", "Ben Ito")]);
/// ```
#[derive(IntoElement)]
pub struct ReadReceipt {
    delivery: Delivery,
    readers: Vec<Avatar>,
}

impl ReadReceipt {
    /// A receipt for a message at `delivery`, with no readers.
    pub fn new(delivery: Delivery) -> Self {
        Self {
            delivery,
            readers: Vec::new(),
        }
    }

    /// Who read it; only a read message has readers, and the first three show.
    pub fn readers(mut self, readers: impl IntoIterator<Item = Avatar>) -> Self {
        self.readers = readers.into_iter().collect();
        assert!(
            self.readers.is_empty() || self.delivery == Delivery::Read,
            "only a read message has readers"
        );
        self
    }
}

impl RenderOnce for ReadReceipt {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let tint = if self.delivery == Delivery::Read {
            colors.info
        } else {
            colors.fg_subtle
        };
        let seen = !self.readers.is_empty();
        div()
            .flex()
            .items_center()
            .gap_1()
            .text_size(theme.text_size(TextSize::Xs))
            .text_color(colors.fg_subtle)
            .child(
                Icon::new(self.delivery.icon())
                    .size(IconSize::Xs)
                    .color(tint),
            )
            .child(if seen {
                crate::i18n::text(cx, "messaging.seen", &[])
            } else {
                self.delivery.label(cx)
            })
            .when(seen, |receipt| {
                receipt.child(AvatarGroup::new(self.readers).max(3).size(AvatarSize::Xs))
            })
    }
}
