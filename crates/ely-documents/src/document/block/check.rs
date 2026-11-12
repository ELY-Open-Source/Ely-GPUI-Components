//! A command's result read back: its text and formats kept.

use std::ops::Range;

use crate::markdown::mdast::Node;

use crate::{
    MarkdownReader, Style,
    document::format::{Change, EditRefusal},
    is_toc,
    markdown::{file_tree, span, walk},
};

const KEPT: u16 = bit(Style::Code) | bit(Style::Math);
const CODE: u16 = 1 << 9;
const MATH: u16 = 1 << 10;
const HTML: u16 = 1 << 11;
const TOC: u16 = 1 << 12;
/// The level `at` gives a paragraph drawn as a table of contents.
const TOC_LEVEL: u8 = u8::MAX;

/// Refuses a result that reads as something else.
pub(super) fn misread(why: &str) -> EditRefusal {
    log::warn!("block command: {why}");
    EditRefusal::Misread
}

/// Refuses `change` unless every shown character and format stays.
pub(super) fn same(
    text: &str,
    root: &Node,
    change: Change,
    why: &str,
) -> Result<Change, EditRefusal> {
    let (written, after) = reread(text, &change.0);
    match Shown::new(&written, &after).all() == Shown::new(text, root).all() {
        true => Ok(change),
        false => Err(misread(why)),
    }
}

/// `text` with sorted `edits` written, and its parse.
pub(super) fn reread(text: &str, edits: &[(Range<usize>, String)]) -> (String, Node) {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    for (range, with) in edits {
        out.push_str(&text[at..range.start]);
        out.push_str(with);
        at = range.end;
    }
    out.push_str(&text[at..]);
    let root = file_tree(&out);
    (out, root)
}

const fn bit(style: Style) -> u16 {
    1 << style as u16
}

/// A parsed document read as the drawing reads it.
pub(super) struct Shown<'a> {
    source: &'a str,
    root: &'a Node,
    reader: MarkdownReader<'a>,
}

impl<'a> Shown<'a> {
    pub(super) fn new(source: &'a str, root: &'a Node) -> Self {
        let reader = MarkdownReader::new(source, root);
        Self {
            source,
            root,
            reader,
        }
    }

    /// Each shown character, with its formats.
    pub(super) fn all(&self) -> Vec<(char, u16)> {
        let mut out = Vec::new();
        self.read(self.root, &mut out);
        out
    }

    fn read(&self, node: &Node, out: &mut Vec<(char, u16)>) {
        let (value, format) = match node {
            // Drawn as a table of contents, not its words.
            Node::Paragraph(_) if is_toc(node, self.source) => return out.push(('\0', TOC)),
            Node::Paragraph(_) | Node::Heading(_) | Node::TableCell(_) => {
                return out.extend(self.inline(node));
            }
            Node::Code(code) => (code.value.as_str(), CODE),
            Node::Math(math) => (math.value.as_str(), MATH),
            Node::Html(html) => (html.value.as_str(), HTML),
            _ => {
                for kid in node.children().map(Vec::as_slice).unwrap_or_default() {
                    self.read(kid, out);
                }
                return;
            }
        };
        out.extend(value.chars().map(|ch| (ch, format)));
    }

    /// `node`'s drawn text; white space counts only in code.
    pub(super) fn inline(&self, node: &Node) -> Vec<(char, u16)> {
        let kids = node.children().map(Vec::as_slice).unwrap_or_default();
        let inline = self.reader.inline(kids);
        let mut formats = vec![0u16; inline.text.len()];
        for (range, style) in &inline.styles {
            for format in &mut formats[range.clone()] {
                *format |= bit(*style);
            }
        }
        (inline.text.char_indices())
            .map(|(at, ch)| (ch, formats[at]))
            .filter(|(ch, format)| !ch.is_whitespace() || format & KEPT != 0)
            .collect()
    }

    /// The level (0 a paragraph) and characters at `at`.
    pub(super) fn at(&self, at: usize) -> Option<(u8, Vec<(char, u16)>)> {
        let mut nodes = Vec::new();
        walk(self.root, &mut nodes);
        nodes.into_iter().find_map(|node| {
            let level = match node {
                Node::Paragraph(_) if is_toc(node, self.source) => TOC_LEVEL,
                Node::Paragraph(_) => 0,
                Node::Heading(heading) => heading.depth,
                _ => return None,
            };
            let whole = span(node);
            (whole.start <= at && at <= whole.end).then(|| (level, self.inline(node)))
        })
    }
}
