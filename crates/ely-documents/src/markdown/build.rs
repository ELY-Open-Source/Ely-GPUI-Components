//! pulldown-cmark's events as Ely's syntax tree, spans in the source.

use std::{borrow::Cow, ops::Range};

use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};

mod node;
mod text;

use text::Run;

use super::{
    flow::{self, Flow},
    front,
    mdast::{self, AlignKind, Leaf, Node, Parent},
    shape::{code_text, indented, literal, place, repeated, shift, tighten, trimmed, written},
};

/// GitHub's flavor, and math between dollars.
pub(super) fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_MATH
}

/// `source` as a syntax tree; front matter too for a whole file.
pub(super) fn tree(source: &str, frontmatter: bool) -> Node {
    // pulldown-cmark reads a byte order mark as text.
    let Some(rest) = source.strip_prefix('\u{FEFF}') else {
        return read(source, frontmatter);
    };
    let mut root = read(rest, frontmatter);
    shift(&mut root, '\u{FEFF}'.len_utf8());
    *root.position_mut() = 0..source.len();
    root
}

fn read(source: &str, frontmatter: bool) -> Node {
    let front = frontmatter.then(|| front::split(source)).flatten();
    let text = front
        .as_ref()
        .map_or(source, |(_, blanked)| blanked.as_str());
    // pulldown-cmark ends no line at a lone CR: it reads as LF, byte for byte.
    let parsed = match text.contains('\r') {
        true => Cow::Owned(lone_crs(text)),
        false => Cow::Borrowed(text),
    };
    let flow = flow::rewrite(&parsed, options());
    let parser = Parser::new_ext(&flow.text, options()).into_offset_iter();
    let definitions: Vec<Node> = (parser.reference_definitions().iter())
        .map(|(label, definition)| {
            Node::Definition(mdast::Definition {
                position: trimmed(text, flow.span(&definition.span)),
                url: definition.dest.to_string(),
                identifier: mdast::identifier(label),
            })
        })
        .collect();
    let mut builder = Builder {
        source: text,
        flow: &flow,
        stack: vec![Frame::new(Kind::Root, 0)],
        text: None,
    };
    for (event, range) in parser {
        builder.event(event, range);
    }
    let mut root = builder.finish();
    place(&mut root, definitions, text);
    tighten(&mut root, text);
    repeated(&mut root, text);
    if let (Some((node, _)), Some(children)) = (front, root.children_mut()) {
        children.insert(0, node);
    }
    root
}

enum Kind {
    Root,
    Paragraph {
        implicit: bool,
    },
    Heading(u8),
    Quote,
    List {
        start: Option<u32>,
        spread: bool,
    },
    Item(Option<bool>),
    Code {
        lang: Option<String>,
        math: bool,
        fenced: bool,
    },
    Html,
    Footnote(String),
    Table(Vec<AlignKind>),
    Row,
    Cell,
    Emphasis,
    Strong,
    Delete,
    Link {
        url: String,
        link: LinkType,
    },
    /// `muted`: the depth an image inside it opened at.
    Image {
        url: String,
        link: LinkType,
        depth: usize,
        muted: Option<usize>,
    },
}

struct Frame {
    kind: Kind,
    children: Vec<Node>,
    start: usize,
    /// Literal text: code, HTML, front matter, an image's alt.
    value: String,
}

impl Frame {
    fn new(kind: Kind, start: usize) -> Self {
        Self {
            kind,
            children: Vec::new(),
            start,
            value: String::new(),
        }
    }
}

struct Builder<'a> {
    source: &'a str,
    flow: &'a Flow,
    stack: Vec<Frame>,
    text: Option<Run>,
}

impl Builder<'_> {
    fn top(&mut self) -> &mut Frame {
        self.stack.last_mut().expect("the root frame stays")
    }

    fn top_kind(&self) -> &Kind {
        &self.stack.last().expect("the root frame stays").kind
    }

    fn event(&mut self, event: Event, range: Range<usize>) {
        let span = self.flow.span(&range);
        if let Kind::Image { depth, .. } = self.top_kind()
            && !(*depth == 0 && matches!(event, Event::End(_)))
        {
            return self.alt(event);
        }
        if matches!(self.top_kind(), Kind::Code { .. } | Kind::Html) {
            match event {
                Event::Text(text) | Event::Html(text) => {
                    // pulldown-cmark drops the CR of a CRLF.
                    let crlf = text.starts_with('\n') && self.source[..span.start].ends_with('\r');
                    let text = written(self.source, &span, &text).to_string();
                    let frame = self.top();
                    if crlf {
                        frame.value.push('\r');
                    }
                    return frame.value.push_str(&text);
                }
                Event::SoftBreak => return self.top().value.push('\n'),
                _ => {}
            }
        }
        if !matches!(event, Event::Text(_) | Event::SoftBreak) {
            self.flush();
        }
        if inline(&event) && matches!(self.top_kind(), Kind::Item(_)) {
            let paragraph = Frame::new(Kind::Paragraph { implicit: true }, span.start);
            self.stack.push(paragraph);
        }
        match event {
            Event::Start(tag) => {
                if !inline_tag(&tag) {
                    self.close_implicit();
                }
                if matches!(tag, Tag::Paragraph) {
                    self.loosen();
                }
                let kind = self.kind(tag, &range);
                self.stack.push(Frame::new(kind, span.start));
            }
            Event::End(end) => {
                let inline = matches!(
                    end,
                    TagEnd::Emphasis
                        | TagEnd::Strong
                        | TagEnd::Strikethrough
                        | TagEnd::Link
                        | TagEnd::Image
                );
                if !inline {
                    self.close_implicit();
                }
                self.close(span.end);
            }
            Event::Text(text) => self.push_text(span, &text),
            Event::SoftBreak => {
                let ending = self.source[span.clone()].trim_start_matches([' ', '\t']);
                match ending.is_empty() {
                    true => self.push_text(span, "\n"),
                    false => self.push_text(span.end - ending.len()..span.end, ending),
                }
            }
            Event::Code(_) => {
                let cell = (self.stack.iter()).any(|frame| matches!(frame.kind, Kind::Cell));
                let value = code_text(self.source, span.clone(), cell);
                self.leaf(Node::InlineCode(literal(&value, span)))
            }
            Event::InlineMath(value) | Event::DisplayMath(value) => {
                self.leaf(Node::InlineMath(literal(&value, span)))
            }
            Event::InlineHtml(value) => self.leaf(Node::Html(literal(&value, span))),
            Event::Html(_) => unreachable!("HTML lines sit in an HTML block"),
            Event::FootnoteReference(label) => {
                self.leaf(Node::FootnoteReference(mdast::FootnoteReference {
                    position: span,
                    identifier: mdast::identifier(&label),
                }))
            }
            Event::HardBreak => self.leaf(Node::Break(Leaf { position: span })),
            Event::Rule => {
                self.close_implicit();
                let start = match self.stack.len() {
                    1 => indented(self.source, span.start),
                    _ => span.start,
                };
                let position = trimmed(self.source, start..span.end);
                self.leaf(Node::ThematicBreak(Leaf { position }))
            }
            Event::TaskListMarker(checked) => {
                let item = (self.stack.iter_mut().rev()).find_map(|frame| match &mut frame.kind {
                    Kind::Item(item) => Some(item),
                    _ => None,
                });
                *item.expect("a task box sits in an item") = Some(checked);
            }
        }
    }

    /// An image's alt: the text of what it holds.
    fn alt(&mut self, event: Event) {
        let frame = self.top();
        let Kind::Image { depth, muted, .. } = &mut frame.kind else {
            unreachable!("alt text belongs to an image")
        };
        match event {
            Event::Start(tag) => {
                // An image inside adds nothing.
                if matches!(tag, Tag::Image { .. }) && muted.is_none() {
                    *muted = Some(*depth);
                }
                *depth += 1;
            }
            Event::End(_) => {
                *depth -= 1;
                if *muted == Some(*depth) {
                    *muted = None;
                }
            }
            _ if muted.is_some() => {}
            Event::Text(text)
            | Event::Code(text)
            | Event::InlineMath(text)
            | Event::DisplayMath(text) => frame.value.push_str(&text),
            Event::SoftBreak | Event::HardBreak => frame.value.push('\n'),
            _ => {}
        }
    }

    fn close(&mut self, end: usize) {
        let mut frame = self.stack.pop().expect("an end has its start");
        let inline = matches!(
            frame.kind,
            Kind::Emphasis | Kind::Strong | Kind::Delete | Kind::Link { .. } | Kind::Image { .. }
        );
        if !inline {
            // pulldown-cmark may start a nested block at the line end before it.
            let rest = &self.source[frame.start..end];
            frame.start += rest.len() - rest.trim_start_matches(['\n', '\r']).len();
        }
        // A fence starts its block; other blocks start at their indent.
        let fenced = matches!(frame.kind, Kind::Code { fenced: true, .. });
        if self.stack.len() == 1 && !inline && !fenced {
            frame.start = indented(self.source, frame.start);
        }
        let range = frame.start..end;
        let node = self.node(frame, range);
        self.top().children.push(node);
    }

    /// A paragraph in an item: its list is loose.
    fn loosen(&mut self) {
        let [.., list, item] = self.stack.as_mut_slice() else {
            return;
        };
        if let (Kind::List { spread, .. }, Kind::Item(_)) = (&mut list.kind, &item.kind) {
            *spread = true;
        }
    }

    /// Ends a tight item's text, which pulldown-cmark leaves bare.
    fn close_implicit(&mut self) {
        self.flush();
        if !matches!(self.top_kind(), Kind::Paragraph { implicit: true }) {
            return;
        }
        let frame = self.stack.pop().expect("an implicit paragraph");
        let end = (frame.children.last()).map_or(frame.start, |child| child.position().end);
        let paragraph = Node::Paragraph(Parent {
            children: frame.children,
            position: frame.start..end,
        });
        self.top().children.push(paragraph);
    }

    fn leaf(&mut self, node: Node) {
        self.top().children.push(node);
    }

    fn finish(mut self) -> Node {
        self.close_implicit();
        let root = self.stack.pop().expect("the root frame stays");
        assert!(self.stack.is_empty(), "every start has its end");
        Node::Root(Parent {
            children: root.children,
            position: 0..self.source.len(),
        })
    }
}

fn inline(event: &Event) -> bool {
    match event {
        Event::Text(_)
        | Event::SoftBreak
        | Event::HardBreak
        | Event::Code(_)
        | Event::InlineMath(_)
        | Event::DisplayMath(_)
        | Event::InlineHtml(_)
        | Event::FootnoteReference(_) => true,
        Event::Start(tag) => inline_tag(tag),
        _ => false,
    }
}

fn inline_tag(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. } | Tag::Image { .. }
    )
}

/// `text` with each lone CR as LF.
fn lone_crs(text: &str) -> String {
    let mut bytes = text.as_bytes().to_vec();
    for ix in 0..bytes.len() {
        if bytes[ix] == b'\r' && bytes.get(ix + 1) != Some(&b'\n') {
            bytes[ix] = b'\n';
        }
    }
    String::from_utf8(bytes).expect("LF for CR keeps UTF-8")
}

#[cfg(test)]
mod tests;
