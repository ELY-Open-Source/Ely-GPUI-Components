//! A command's result read back: text, formats, kinds kept.

use std::ops::Range;

use crate::markdown::mdast::Node;

use crate::{
    MarkdownReader, Style, alert,
    document::format::{Change, EditRefusal},
    is_toc,
    markdown::{file_tree, span, walk},
};

const KEPT: u32 = bit(Style::Code) | bit(Style::Math);
const CODE: u32 = 1 << 9;
const MATH: u32 = 1 << 10;
const HTML: u32 = 1 << 11;
const TOC: u32 = 1 << 12;
const BREAK: u32 = 1 << 13;
/// Where a heading's depth, then an alert's kind, sit.
const DEPTH: u32 = 14;
const DEPTHS: u32 = 0b111 << DEPTH;
const ALERT: u32 = 17;
/// A mark whose block may take any depth.
const LOOSE: u32 = 1 << 20;
/// The level `at` gives a paragraph drawn as contents.
const TOC_LEVEL: u8 = u8::MAX;

/// A shown character, with its formats and block kind.
pub(super) type Mark = (char, u32);

/// What a command makes of one block.
#[derive(Clone, Copy)]
pub(super) enum Goal {
    /// Some of its lines change depth.
    Relevel,
    /// An alert's body, out of its quote.
    Unquoted,
}

/// Refuses a result that reads as something else.
pub(super) fn misread(why: &str) -> EditRefusal {
    log::warn!("block command: {why}");
    EditRefusal::Misread
}

/// Refuses `change` unless `before`'s every mark stays.
pub(super) fn same(before: &Shown, change: Change, why: &str) -> Result<Change, EditRefusal> {
    let (written, after) = reread(before.source, &change.0);
    match kept(&before.all(), &Shown::new(&written, &after).all()) {
        true => Ok(change),
        false => Err(misread(why)),
    }
}

/// Whether `got` reads as `want`; loose marks take any depth.
pub(super) fn kept(want: &[Mark], got: &[Mark]) -> bool {
    want.len() == got.len()
        && want
            .iter()
            .zip(got)
            .all(|(want, got)| match want.1 & LOOSE {
                0 => want == got,
                _ => want.0 == got.0 && want.1 & !LOOSE == got.1 & !DEPTHS,
            })
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

const fn bit(style: Style) -> u32 {
    1 << style as u32
}

/// A parsed document read as the drawing reads it.
pub(super) struct Shown<'a> {
    source: &'a str,
    root: &'a Node,
    reader: MarkdownReader<'a>,
    /// One block read as its command leaves it.
    goal: Option<(&'a Node, Goal)>,
}

impl<'a> Shown<'a> {
    pub(super) fn new(source: &'a str, root: &'a Node) -> Self {
        let reader = MarkdownReader::new(source, root);
        Self {
            source,
            root,
            reader,
            goal: None,
        }
    }

    /// Reads `node` as `goal` makes it.
    pub(super) fn with(mut self, node: &'a Node, goal: Goal) -> Self {
        self.goal = Some((node, goal));
        self
    }

    /// Each shown character, with its formats and kind.
    pub(super) fn all(&self) -> Vec<Mark> {
        let mut out = Vec::new();
        self.read(self.root, 0, &mut out);
        out
    }

    /// Whether the drawing reads `node` as a root block.
    fn top(&self, node: &Node) -> bool {
        let top = |kids: &Vec<Node>| kids.iter().any(|kid| std::ptr::eq(kid, node));
        self.root.children().is_some_and(top)
    }

    /// A paragraph the drawing shows as a table of contents.
    fn toc(&self, node: &Node) -> bool {
        is_toc(node, self.source) && self.top(node)
    }

    fn read(&self, node: &Node, kind: u32, out: &mut Vec<Mark>) {
        let goal = self
            .goal
            .filter(|(target, _)| std::ptr::eq(*target, node))
            .map(|(_, goal)| goal);
        let (value, format) = match (node, goal) {
            (_, Some(Goal::Relevel)) => return self.marks(node, kind | LOOSE, out),
            // Drawn as a table of contents, not its words.
            (Node::Paragraph(_), _) if self.toc(node) => return out.push(('\0', TOC)),
            (Node::Paragraph(_) | Node::TableCell(_), _) => return self.marks(node, kind, out),
            (Node::Heading(heading), _) => {
                return self.marks(node, kind | u32::from(heading.depth) << DEPTH, out);
            }
            (Node::ThematicBreak(_), _) => return out.push(('\0', BREAK | kind)),
            (Node::Blockquote(quote), _) if self.top(node) => {
                if let Some((alerted, body)) = alert(quote) {
                    let kind = match goal {
                        Some(Goal::Unquoted) => kind,
                        _ => kind | (alerted as u32 + 1) << ALERT,
                    };
                    // The rest are the tree's own nodes: goals find them.
                    let rest = &quote.children[1..];
                    let head = &body[..body.len() - rest.len()];
                    // The head stands for the marker's paragraph.
                    let first = quote.children.first();
                    let loose = match self.goal {
                        Some((target, Goal::Relevel))
                            if first.is_some_and(|first| std::ptr::eq(first, target)) =>
                        {
                            LOOSE
                        }
                        _ => 0,
                    };
                    head.iter()
                        .for_each(|kid| self.read(kid, kind | loose, out));
                    return rest.iter().for_each(|kid| self.read(kid, kind, out));
                }
                return self.kids(node, kind, out);
            }
            (Node::Code(code), _) => (code.value.as_str(), CODE),
            (Node::Math(math), _) => (math.value.as_str(), MATH),
            (Node::Html(html), _) => (html.value.as_str(), HTML),
            _ => return self.kids(node, kind, out),
        };
        out.extend(value.chars().map(|ch| (ch, format | kind)));
    }

    fn kids(&self, node: &Node, kind: u32, out: &mut Vec<Mark>) {
        for kid in node.children().map(Vec::as_slice).unwrap_or_default() {
            self.read(kid, kind, out);
        }
    }

    fn marks(&self, node: &Node, kind: u32, out: &mut Vec<Mark>) {
        out.extend(self.inline(node).into_iter().map(|(ch, f)| (ch, f | kind)));
    }

    /// `node`'s drawn text; white space counts only in code.
    pub(super) fn inline(&self, node: &Node) -> Vec<Mark> {
        let kids = node.children().map(Vec::as_slice).unwrap_or_default();
        let inline = self.reader.inline(kids);
        let mut formats = vec![0u32; inline.text.len()];
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
    pub(super) fn at(&self, at: usize) -> Option<(u8, Vec<Mark>)> {
        let mut nodes = Vec::new();
        walk(self.root, &mut nodes);
        nodes.into_iter().find_map(|node| {
            let whole = span(node);
            if !(whole.start <= at && at <= whole.end) {
                return None;
            }
            let level = match node {
                Node::Paragraph(_) if self.toc(node) => TOC_LEVEL,
                Node::Paragraph(_) => 0,
                Node::Heading(heading) => heading.depth,
                _ => return None,
            };
            Some((level, self.inline(node)))
        })
    }
}
