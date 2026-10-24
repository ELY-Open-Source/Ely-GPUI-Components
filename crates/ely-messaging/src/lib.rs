//! Messaging, chapter 28: channels, messages, threads, members, statuses, pickers and calls.
#![deny(missing_docs)]

use ely_core::{
    buttons, data_display, feedback, forms, i18n, layout, lists, motion, overlays, primitives,
    theme, typography,
};
use ely_media as media;

mod call;
mod channels;
mod gifs;
mod header;
mod incoming;
mod members;
mod messages;
mod people;
mod pins;
mod receipt;
mod status;
mod stickers;
mod thread;
mod tiles;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use call::{CallControls, HuddleIndicator, VoiceCallBar};
pub use channels::{ChannelItem, ChannelList, ChatRow, DirectMessageItem};
pub use gifs::{Gif, GifPicker, Gifs};
pub use header::ChannelHeader;
pub use incoming::IncomingCallDialog;
pub use members::{Member, MemberList};
pub use messages::{ChatMessage, UnreadDivider};
pub use people::{OnlineStatus, UserProfileCard};
pub use pins::{PinnedMessage, PinnedMessages};
pub use receipt::{Delivery, ReadReceipt};
pub use status::{ClearAfter, Status, StatusSetter};
pub use stickers::{Sticker, StickerPack, StickerPicker};
pub use thread::{MessageThread, ThreadPanel};
pub use tiles::{ParticipantTile, ScreenShareView, VideoCallGrid};
