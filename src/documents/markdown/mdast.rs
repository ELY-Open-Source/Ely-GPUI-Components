//! The syntax tree Ely reads: mdast's shape, byte spans for positions.

use std::ops::Range;

/// A parsed node; `position` is its span in the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Node {
    Root(Parent),
    Paragraph(Parent),
    Heading(Heading),
    ThematicBreak(Leaf),
    Blockquote(Parent),
    List(List),
    ListItem(ListItem),
    Html(Literal),
    Code(Code),
    Math(Literal),
    Definition(Definition),
    Text(Literal),
    Emphasis(Parent),
    Strong(Parent),
    Delete(Parent),
    InlineCode(Literal),
    InlineMath(Literal),
    Break(Leaf),
    Link(Link),
    Image(Image),
    LinkReference(LinkReference),
    ImageReference(ImageReference),
    FootnoteDefinition(FootnoteDefinition),
    FootnoteReference(FootnoteReference),
    Table(Table),
    TableRow(Parent),
    TableCell(Parent),
    Yaml(Literal),
    Toml(Literal),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Parent {
    pub children: Vec<Node>,
    pub position: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Leaf {
    pub position: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Literal {
    pub value: String,
    pub position: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Heading {
    pub children: Vec<Node>,
    pub position: Range<usize>,
    pub depth: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct List {
    pub children: Vec<Node>,
    pub position: Range<usize>,
    pub ordered: bool,
    pub start: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ListItem {
    pub children: Vec<Node>,
    pub position: Range<usize>,
    pub checked: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Code {
    pub value: String,
    pub position: Range<usize>,
    pub lang: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Definition {
    pub position: Range<usize>,
    pub url: String,
    pub identifier: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Link {
    pub children: Vec<Node>,
    pub position: Range<usize>,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Image {
    pub position: Range<usize>,
    pub alt: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LinkReference {
    pub children: Vec<Node>,
    pub position: Range<usize>,
    pub identifier: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ImageReference {
    pub position: Range<usize>,
    pub alt: String,
    pub identifier: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FootnoteDefinition {
    pub children: Vec<Node>,
    pub position: Range<usize>,
    pub identifier: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FootnoteReference {
    pub position: Range<usize>,
    pub identifier: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Table {
    pub children: Vec<Node>,
    pub position: Range<usize>,
    pub align: Vec<AlignKind>,
}

/// A table column's alignment, as its delimiter row writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignKind {
    None,
    Left,
    Right,
    Center,
}

impl Node {
    pub(crate) fn position(&self) -> &Range<usize> {
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

    pub(crate) fn children(&self) -> Option<&Vec<Node>> {
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
