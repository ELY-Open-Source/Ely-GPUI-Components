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
