mod copy;
mod effects;
mod emoji;
mod fit;
/// Numbers, sizes, durations and times as text.
pub mod format;
mod formatted;
mod inline;
/// Keystrokes and their key caps on each platform.
pub mod keys;
mod link;
mod rolling;
mod select;
mod text;

pub use copy::CopyableText;
pub use effects::{GradientText, ShimmerText, Typewriter};
pub use emoji::Emoji;
pub use fit::{Ellipsis, EllipsisTooltip, LEADING, MiddleEllipsis, text_width};
pub use format::DurationStyle;
pub use formatted::{
    CurrencyText, DateTimeText, DurationText, FileSizeText, NumberText, PercentText, PluralText,
    RelativeTime, fresh, relative_text,
};
pub use inline::{Blockquote, Code, Highlight, Kbd, KbdCombo};
pub use link::{ExternalLink, Link};
pub use rolling::AnimatedNumber;
pub use select::SelectableText;
pub use text::{Caption, Heading, Label, Overline, Paragraph, Subtitle, Title, literal, tabular};
