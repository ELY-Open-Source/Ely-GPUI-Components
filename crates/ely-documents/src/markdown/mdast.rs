//! The syntax tree Ely reads: mdast's shape, byte spans for positions.

use std::ops::Range;

/// A parsed node; `position` is its span in the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// The document.
    Root(Parent),
    /// A paragraph.
    Paragraph(Parent),
    /// An ATX or setext heading.
    Heading(Heading),
    /// A thematic break.
    ThematicBreak(Leaf),
    /// A block quote.
    Blockquote(Parent),
    /// A list.
    List(List),
    /// A list item.
    ListItem(ListItem),
    /// Raw HTML, a block or inline.
    Html(Literal),
    /// A fenced or indented code block.
    Code(Code),
    /// A `$$` math block.
    Math(Literal),
    /// A link reference definition.
    Definition(Definition),
    /// Plain text.
    Text(Literal),
    /// Emphasis.
    Emphasis(Parent),
    /// Strong emphasis.
    Strong(Parent),
    /// Struck-through text.
    Delete(Parent),
    /// Inline code.
    InlineCode(Literal),
    /// Inline math.
    InlineMath(Literal),
    /// A hard line break.
    Break(Leaf),
    /// An inline link or autolink.
    Link(Link),
    /// An inline image.
    Image(Image),
    /// A link by reference.
    LinkReference(LinkReference),
    /// An image by reference.
    ImageReference(ImageReference),
    /// A footnote definition.
    FootnoteDefinition(FootnoteDefinition),
    /// A footnote reference.
    FootnoteReference(FootnoteReference),
    /// A GFM table.
    Table(Table),
    /// A table row.
    TableRow(Parent),
    /// A table cell.
    TableCell(Parent),
    /// YAML front matter.
    Yaml(Literal),
    /// TOML front matter.
    Toml(Literal),
}

/// A node of nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parent {
    /// The nodes inside.
    pub children: Vec<Node>,
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
}

/// A node with nothing inside.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leaf {
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
}

/// A node of text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Literal {
    /// Its text, unescaped.
    pub value: String,
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
}

/// A heading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heading {
    /// The nodes inside.
    pub children: Vec<Node>,
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// Its level, 1 to 6.
    pub depth: u8,
}

/// A list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct List {
    /// The nodes inside.
    pub children: Vec<Node>,
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// Numbered.
    pub ordered: bool,
    /// A numbered list's first number.
    pub start: Option<u32>,
    /// Loose: its items' text sits in paragraphs.
    pub spread: bool,
}

/// A list item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem {
    /// The nodes inside.
    pub children: Vec<Node>,
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// A task box: ticked or not.
    pub checked: Option<bool>,
}

/// A code block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code {
    /// Its text, unescaped.
    pub value: String,
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// The fence's info string's first word.
    pub lang: Option<String>,
}

/// A link reference definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// The address.
    pub url: String,
    /// The label, as references match it.
    pub identifier: String,
}

/// A link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// The nodes inside.
    pub children: Vec<Node>,
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// The address.
    pub url: String,
}

/// An image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// The alternative text.
    pub alt: String,
    /// The address.
    pub url: String,
}

/// A link by reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkReference {
    /// The nodes inside.
    pub children: Vec<Node>,
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// The label, as references match it.
    pub identifier: String,
}

/// An image by reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageReference {
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// The alternative text.
    pub alt: String,
    /// The label, as references match it.
    pub identifier: String,
}

/// A footnote definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FootnoteDefinition {
    /// The nodes inside.
    pub children: Vec<Node>,
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// The label, as references match it.
    pub identifier: String,
    /// The label as written.
    pub label: String,
}

/// A footnote reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FootnoteReference {
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// The label, as references match it.
    pub identifier: String,
}

/// A table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    /// The nodes inside.
    pub children: Vec<Node>,
    /// Its span in the source, in bytes.
    pub position: Range<usize>,
    /// Each column's alignment.
    pub align: Vec<AlignKind>,
}

/// A table column's alignment, as its delimiter row writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignKind {
    /// `---`, no colon.
    None,
    /// `:---`, at the start.
    Left,
    /// `---:`, at the end.
    Right,
    /// `:---:`, in the middle.
    Center,
}

impl Node {
    /// Its span in the source, in bytes.
    pub fn position(&self) -> &Range<usize> {
        use Node::*;
        match self {
            Root(x) | Paragraph(x) | Blockquote(x) | Emphasis(x) | Strong(x) | Delete(x)
            | TableRow(x) | TableCell(x) => &x.position,
            ThematicBreak(x) | Break(x) => &x.position,
            Html(x) | Math(x) | Text(x) | InlineCode(x) | InlineMath(x) | Yaml(x) | Toml(x) => {
                &x.position
            }
            Heading(x) => &x.position,
            List(x) => &x.position,
            ListItem(x) => &x.position,
            Code(x) => &x.position,
            Definition(x) => &x.position,
            Link(x) => &x.position,
            Image(x) => &x.position,
            LinkReference(x) => &x.position,
            ImageReference(x) => &x.position,
            FootnoteDefinition(x) => &x.position,
            FootnoteReference(x) => &x.position,
            Table(x) => &x.position,
        }
    }

    pub(crate) fn position_mut(&mut self) -> &mut Range<usize> {
        use Node::*;
        match self {
            Root(x) | Paragraph(x) | Blockquote(x) | Emphasis(x) | Strong(x) | Delete(x)
            | TableRow(x) | TableCell(x) => &mut x.position,
            ThematicBreak(x) | Break(x) => &mut x.position,
            Html(x) | Math(x) | Text(x) | InlineCode(x) | InlineMath(x) | Yaml(x) | Toml(x) => {
                &mut x.position
            }
            Heading(x) => &mut x.position,
            List(x) => &mut x.position,
            ListItem(x) => &mut x.position,
            Code(x) => &mut x.position,
            Definition(x) => &mut x.position,
            Link(x) => &mut x.position,
            Image(x) => &mut x.position,
            LinkReference(x) => &mut x.position,
            ImageReference(x) => &mut x.position,
            FootnoteDefinition(x) => &mut x.position,
            FootnoteReference(x) => &mut x.position,
            Table(x) => &mut x.position,
        }
    }

    /// The nodes inside, if it holds any.
    pub fn children(&self) -> Option<&Vec<Node>> {
        use Node::*;
        match self {
            Root(x) | Paragraph(x) | Blockquote(x) | Emphasis(x) | Strong(x) | Delete(x)
            | TableRow(x) | TableCell(x) => Some(&x.children),
            Heading(x) => Some(&x.children),
            List(x) => Some(&x.children),
            ListItem(x) => Some(&x.children),
            Link(x) => Some(&x.children),
            LinkReference(x) => Some(&x.children),
            FootnoteDefinition(x) => Some(&x.children),
            Table(x) => Some(&x.children),
            _ => None,
        }
    }

    pub(crate) fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        use Node::*;
        match self {
            Root(x) | Paragraph(x) | Blockquote(x) | Emphasis(x) | Strong(x) | Delete(x)
            | TableRow(x) | TableCell(x) => Some(&mut x.children),
            Heading(x) => Some(&mut x.children),
            List(x) => Some(&mut x.children),
            ListItem(x) => Some(&mut x.children),
            Link(x) => Some(&mut x.children),
            LinkReference(x) => Some(&mut x.children),
            FootnoteDefinition(x) => Some(&mut x.children),
            Table(x) => Some(&mut x.children),
            _ => None,
        }
    }
}

/// A label as definitions and references match it.
pub(crate) fn identifier(label: &str) -> String {
    let words: Vec<&str> = (label.split([' ', '\t', '\n', '\r']))
        .filter(|word| !word.is_empty())
        .collect();
    words.join(" ").to_lowercase().to_uppercase().to_lowercase()
}
