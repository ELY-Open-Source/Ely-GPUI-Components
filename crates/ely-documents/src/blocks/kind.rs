use gpui::{App, SharedString};

use crate::primitives::IconName;

/// Where a picture sits across its column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// At the column's start.
    Left,
    /// In the middle.
    #[default]
    Center,
    /// At the column's end.
    Right,
}

/// A picture, clip or page shown in a block: its address, and its width as a share of the column.
#[derive(Clone, Debug, PartialEq)]
pub struct Media {
    /// Its address.
    pub source: SharedString,
    /// Its width as a share of the column, 0 to 1.
    pub width: f32,
    /// Where it sits across the column.
    pub align: Align,
}

impl Media {
    /// Media at `source`, the column's full width, centered.
    pub fn new(source: impl Into<SharedString>) -> Self {
        Self {
            source: source.into(),
            width: 1.0,
            align: Align::Center,
        }
    }
}

/// What a block is. Each holds its text in fields: one for most, a title and a body for a toggle, one per column, a caption for media, one per cell for a table, none for a divider.
#[derive(Clone, Debug, PartialEq)]
pub enum BlockKind {
    /// Prose.
    Paragraph,
    /// A heading of its level, 1 to 3.
    Heading(u8),
    /// A bulleted item.
    Bullet,
    /// A numbered item.
    Numbered,
    /// A to-do, done or not.
    Todo(bool),
    /// A quote.
    Quote,
    /// A callout with an icon.
    Callout,
    /// Open or closed; its body shows while open.
    Toggle(bool),
    /// Source code.
    Code,
    /// A TeX formula.
    Math,
    /// Diagram source, such as Mermaid; the host renders the picture.
    Diagram,
    /// A rule across the column.
    Divider,
    /// Columns across, the header row first; each `merged` cell, as row and column, folds into the cell on its left.
    Table {
        /// How many columns.
        columns: usize,
        /// Cells folded into the cell on their left, as row and column.
        merged: Vec<(usize, usize)>,
    },
    /// A picture with a caption.
    Image(Media),
    /// A clip with a caption.
    Video(Media),
    /// An embedded page with a caption.
    Embed(Media),
    /// Side by side, one field each.
    Columns(usize),
    /// Text shared with every other block synced to the same source.
    Synced(SharedString),
}

impl BlockKind {
    /// Its key, as logs and the catalog name it.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Paragraph => "paragraph",
            Self::Heading(1) => "heading-1",
            Self::Heading(2) => "heading-2",
            Self::Heading(_) => "heading-3",
            Self::Bullet => "bullet",
            Self::Numbered => "numbered",
            Self::Todo(_) => "todo",
            Self::Quote => "quote",
            Self::Callout => "callout",
            Self::Toggle(_) => "toggle",
            Self::Code => "code",
            Self::Math => "math",
            Self::Diagram => "diagram",
            Self::Divider => "divider",
            Self::Table { .. } => "table",
            Self::Image(_) => "image",
            Self::Video(_) => "video",
            Self::Embed(_) => "embed",
            Self::Columns(_) => "columns",
            Self::Synced(_) => "synced",
        }
    }

    /// Its name in the slash menu, in the shown locale.
    pub fn label(&self, cx: &App) -> SharedString {
        crate::i18n::text(cx, &format!("block.{}", self.key()), &[])
    }

    /// The icon its slash menu row shows.
    pub fn icon(&self) -> IconName {
        match self {
            Self::Paragraph => IconName::Type,
            Self::Heading(1) => IconName::Heading1,
            Self::Heading(2) => IconName::Heading2,
            Self::Heading(_) => IconName::Heading3,
            Self::Bullet => IconName::List,
            Self::Numbered => IconName::ListOrdered,
            Self::Todo(_) => IconName::ListTodo,
            Self::Quote => IconName::Quote,
            Self::Callout => IconName::Info,
            Self::Toggle(_) => IconName::ChevronRight,
            Self::Code => IconName::Code,
            Self::Math => IconName::SquareFunction,
            Self::Diagram => IconName::Workflow,
            Self::Divider => IconName::Minus,
            Self::Table { .. } => IconName::Table,
            Self::Image(_) => IconName::Image,
            Self::Video(_) => IconName::Video,
            Self::Embed(_) => IconName::Globe,
            Self::Columns(_) => IconName::Columns2,
            Self::Synced(_) => IconName::RefreshCw,
        }
    }

    /// What an empty field of this kind says while it has focus.
    pub fn placeholder(&self, cx: &App) -> SharedString {
        let key = match self {
            Self::Paragraph => "block.hint.paragraph",
            Self::Heading(_) | Self::Todo(_) | Self::Quote | Self::Code => return self.label(cx),
            Self::Bullet | Self::Numbered => "block.hint.list",
            Self::Callout | Self::Toggle(_) | Self::Columns(_) | Self::Synced(_) => {
                "block.hint.write"
            }
            Self::Math => {
                let sample = ("sample", "e^{i\\pi} + 1 = 0");
                return crate::i18n::text(cx, "block.hint.math", &[sample]);
            }
            Self::Diagram => "block.hint.diagram",
            Self::Image(_) | Self::Video(_) | Self::Embed(_) => "block.hint.caption",
            Self::Divider | Self::Table { .. } => return SharedString::default(),
        };
        crate::i18n::text(cx, key, &[])
    }

    /// How many fields a new block of this kind holds.
    pub fn fields(&self) -> usize {
        match self {
            Self::Divider => 0,
            Self::Toggle(_) => 2,
            Self::Columns(count) => *count,
            Self::Table { columns, .. } => columns * 2,
            _ => 1,
        }
    }

    /// Whether it can hold `count` fields: a table any whole rows past its header, the rest their own number.
    pub fn accepts(&self, count: usize) -> bool {
        match self {
            Self::Table { columns, .. } => {
                *columns > 0 && count >= columns * 2 && count.is_multiple_of(*columns)
            }
            kind => count == kind.fields(),
        }
    }

    /// Whether a field made for this kind serves `other` unchanged: the same highlighter and hint.
    pub(crate) fn same_fields(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
            && match (self, other) {
                (Self::Heading(one), Self::Heading(two)) => one == two,
                (Self::Synced(one), Self::Synced(two)) => one == two,
                _ => true,
            }
    }

    /// Prose: Enter splits it, Backspace at its start turns it back into text or joins it up.
    pub fn is_prose(&self) -> bool {
        matches!(
            self,
            Self::Paragraph
                | Self::Heading(_)
                | Self::Bullet
                | Self::Numbered
                | Self::Todo(_)
                | Self::Quote
                | Self::Callout
        )
    }

    /// The kind Enter starts after this one: lists go on, the rest return to text.
    pub fn next(&self) -> Self {
        match self {
            Self::Bullet => Self::Bullet,
            Self::Numbered => Self::Numbered,
            Self::Todo(_) => Self::Todo(false),
            _ => Self::Paragraph,
        }
    }

    /// Lines of monospaced source, where Enter breaks a line.
    pub fn is_source(&self) -> bool {
        matches!(self, Self::Code | Self::Math | Self::Diagram)
    }
}

/// The kinds the slash menu offers, in order; media and synced blocks come from their owner.
pub(crate) fn offered() -> Vec<BlockKind> {
    vec![
        BlockKind::Paragraph,
        BlockKind::Heading(1),
        BlockKind::Heading(2),
        BlockKind::Heading(3),
        BlockKind::Bullet,
        BlockKind::Numbered,
        BlockKind::Todo(false),
        BlockKind::Quote,
        BlockKind::Callout,
        BlockKind::Toggle(true),
        BlockKind::Code,
        BlockKind::Math,
        BlockKind::Diagram,
        BlockKind::Divider,
        BlockKind::Table {
            columns: 2,
            merged: Vec::new(),
        },
        BlockKind::Columns(2),
    ]
}

/// The named kinds whose name holds `query`, for the slash menu.
pub(crate) fn matching(
    query: &str,
    named: &[(BlockKind, SharedString)],
) -> Vec<(BlockKind, SharedString)> {
    let query = query.to_lowercase();
    (named.iter())
        .filter(|(_, name)| name.to_lowercase().contains(&query))
        .cloned()
        .collect()
}

/// A paragraph's leading markdown that makes it another kind, and how many bytes it takes; the space after it is typed last.
pub(crate) fn shortcut(text: &str) -> Option<(BlockKind, usize)> {
    let prefixes: [(&str, BlockKind); 11] = [
        ("# ", BlockKind::Heading(1)),
        ("## ", BlockKind::Heading(2)),
        ("### ", BlockKind::Heading(3)),
        ("- ", BlockKind::Bullet),
        ("* ", BlockKind::Bullet),
        ("1. ", BlockKind::Numbered),
        ("[] ", BlockKind::Todo(false)),
        ("[ ] ", BlockKind::Todo(false)),
        ("[x] ", BlockKind::Todo(true)),
        ("> ", BlockKind::Quote),
        ("``` ", BlockKind::Code),
    ];
    if text == "---" {
        return Some((BlockKind::Divider, 3));
    }
    prefixes
        .into_iter()
        .find(|(prefix, _)| text.starts_with(prefix))
        .map(|(prefix, kind)| (kind, prefix.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_at_a_paragraphs_start_turns_it_into_a_kind() {
        assert_eq!(shortcut("## Lift"), Some((BlockKind::Heading(2), 3)));
        assert_eq!(shortcut("[x] done"), Some((BlockKind::Todo(true), 4)));
        assert_eq!(shortcut("---"), Some((BlockKind::Divider, 3)));
        assert_eq!(shortcut("#hashtag"), None);
        assert_eq!(shortcut("1.5 pages"), None);
    }

    #[test]
    fn the_slash_menu_filters_by_name() {
        let named: Vec<(BlockKind, SharedString)> = [
            (BlockKind::Paragraph, "Text"),
            (BlockKind::Heading(1), "Heading 1"),
            (BlockKind::Heading(2), "Heading 2"),
            (BlockKind::Quote, "Quote"),
        ]
        .map(|(kind, name)| (kind, name.into()))
        .into();
        let found: Vec<SharedString> = (matching("head", &named).into_iter())
            .map(|(_, name)| name)
            .collect();
        assert_eq!(found, ["Heading 1", "Heading 2"]);
        assert!(matching("zzz", &named).is_empty());
    }

    #[test]
    fn a_table_holds_whole_rows_past_its_header() {
        let table = BlockKind::Table {
            columns: 3,
            merged: Vec::new(),
        };
        assert!(table.accepts(9));
        assert!(!table.accepts(7) && !table.accepts(3));
        assert!(BlockKind::Toggle(true).accepts(2) && !BlockKind::Paragraph.accepts(2));
    }

    #[test]
    fn lists_go_on_and_the_rest_return_to_text() {
        assert_eq!(BlockKind::Todo(true).next(), BlockKind::Todo(false));
        assert_eq!(BlockKind::Heading(1).next(), BlockKind::Paragraph);
    }
}
