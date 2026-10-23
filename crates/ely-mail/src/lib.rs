//! Mail, chapter 29: mailboxes, lists, threads, the reader, the composer, labels, and times to send or snooze.
#![deny(missing_docs)]

use ely_chat as chat;
use ely_core::{
    buttons, data_display, forms, i18n, layout, lists, menus, overlays, primitives, theme,
    typography,
};

mod boxes;
mod compose;
mod item;
mod labels;
mod list;
mod quoted;
mod reader;
mod send;
mod signature;
mod snooze;
mod thread;
mod times;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use boxes::{Mailbox, MailboxList};
pub use compose::{Draft, MailComposer};
pub use item::{Mail, MailItem};
pub use labels::{Label, LabelPicker};
pub use list::MailList;
pub use quoted::QuotedText;
pub use reader::{Contact, MailReader, Message};
pub use send::ScheduleSend;
pub use signature::SignatureEditor;
pub use snooze::SnoozePicker;
pub use thread::MailThreadView;
