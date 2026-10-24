//! Tags as frames, and closed frames as nodes.

use std::ops::Range;

use pulldown_cmark::{CodeBlockKind, Tag};

use super::{
    super::{
        mdast::{self, Node, Parent},
        shape::{address, align, boxed, cells, collapsed, literal, reference, trimmed, unended},
    },
    Builder, Frame, Kind,
};

impl Builder<'_> {
    pub(super) fn kind(&self, tag: Tag, range: &Range<usize>) -> Kind {
        match tag {
            Tag::Paragraph => Kind::Paragraph { implicit: false },
            Tag::Heading { level, .. } => Kind::Heading(level as u8),
            Tag::BlockQuote(_) => Kind::Quote,
            Tag::CodeBlock(CodeBlockKind::Fenced(info)) => {
                let text = &self.flow.text;
                let end = (text[range.start..].find(['\n', '\r']))
                    .map_or(text.len(), |ix| range.start + ix);
                let math = self.flow.opens_math(&(range.start..end));
                let lang = info.split_whitespace().next().map(str::to_string);
                Kind::Code {
                    lang: lang.filter(|_| !math),
                    math,
                    fenced: true,
                }
            }
            Tag::CodeBlock(CodeBlockKind::Indented) => Kind::Code {
                lang: None,
                math: false,
                fenced: false,
            },
            Tag::HtmlBlock => Kind::Html,
            Tag::List(start) => Kind::List {
                start: start.map(|start| {
                    u32::try_from(start).expect("a list starts at nine digits at most")
                }),
                spread: false,
            },
            Tag::Item => Kind::Item(None),
            Tag::FootnoteDefinition(label) => Kind::Footnote(label.to_string()),
            Tag::Table(aligns) => Kind::Table(aligns.iter().map(align).collect()),
            Tag::TableHead | Tag::TableRow => Kind::Row,
            Tag::TableCell => Kind::Cell,
            Tag::Emphasis => Kind::Emphasis,
            Tag::Strong => Kind::Strong,
            Tag::Strikethrough => Kind::Delete,
            Tag::Link {
                link_type,
                dest_url,
                ..
            } => Kind::Link {
                url: address(link_type, &dest_url),
                link: link_type,
            },
            Tag::Image {
                link_type,
                dest_url,
                ..
            } => Kind::Image {
                url: address(link_type, &dest_url),
                link: link_type,
                depth: 0,
                muted: None,
            },
            other => unreachable!("{other:?} is off in the parser's options"),
        }
    }

    pub(super) fn node(&self, frame: Frame, range: Range<usize>) -> Node {
        let Frame {
            kind,
            children,
            value,
            ..
        } = frame;
        let range = match &kind {
            Kind::Link { link, .. } | Kind::Image { link, .. } => {
                collapsed(self.source, range, *link)
            }
            _ => range,
        };
        let block = trimmed(self.source, range.clone());
        match kind {
            Kind::Root => unreachable!("the root closes at the end"),
            Kind::Paragraph { .. } => Node::Paragraph(Parent {
                children,
                position: block,
            }),
            Kind::Heading(depth) => Node::Heading(mdast::Heading {
                children,
                position: block,
                depth,
            }),
            Kind::Quote => Node::Blockquote(Parent {
                children,
                position: block,
            }),
            Kind::List { start, spread } => Node::List(mdast::List {
                position: block,
                children,
                ordered: start.is_some(),
                start,
                spread,
            }),
            Kind::Item(checked) => {
                let (checked, children) = match checked {
                    Some(_) => (checked, children),
                    None => boxed(self.source, children),
                };
                Node::ListItem(mdast::ListItem {
                    position: block,
                    children,
                    checked,
                })
            }
            Kind::Code { math: true, .. } => Node::Math(literal(unended(&value), block.clone())),
            Kind::Code { lang, .. } => Node::Code(mdast::Code {
                value: unended(&value).to_string(),
                position: block,
                lang,
            }),
            Kind::Html => Node::Html(literal(unended(&value), block.clone())),
            Kind::Footnote(label) => Node::FootnoteDefinition(mdast::FootnoteDefinition {
                children,
                position: block,
                identifier: mdast::identifier(&label),
                label,
            }),
            Kind::Table(align) => Node::Table(mdast::Table {
                children,
                position: block,
                align,
            }),
            Kind::Row => Node::TableRow(Parent {
                children: cells(self.source, children, block.end),
                position: block,
            }),
            Kind::Cell => Node::TableCell(Parent {
                children,
                position: range,
            }),
            Kind::Emphasis => Node::Emphasis(Parent {
                children,
                position: range,
            }),
            Kind::Strong => Node::Strong(Parent {
                children,
                position: range,
            }),
            Kind::Delete => Node::Delete(Parent {
                children,
                position: range,
            }),
            Kind::Link { url, link } => match reference(self.source, &range, link) {
                Some(identifier) => Node::LinkReference(mdast::LinkReference {
                    children,
                    position: range,
                    identifier,
                }),
                None => Node::Link(mdast::Link {
                    children,
                    position: range,
                    url,
                }),
            },
            Kind::Image { url, link, .. } => match reference(self.source, &range, link) {
                Some(identifier) => Node::ImageReference(mdast::ImageReference {
                    position: range,
                    alt: value,
                    identifier,
                }),
                None => Node::Image(mdast::Image {
                    position: range,
                    alt: value,
                    url,
                }),
            },
        }
    }
}
