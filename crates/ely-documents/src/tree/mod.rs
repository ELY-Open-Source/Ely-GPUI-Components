use std::{collections::HashMap, ops::Range};

use crate::markdown::mdast::{self, AlignKind};

use super::markdown::{span, tree, walk};

mod blocks;
mod dom;
mod drawn;
pub(crate) mod emoji;
pub(crate) mod html;
mod html_block;
mod sources;
mod written;

pub use blocks::{AlertKind, DocumentHeading};
pub use blocks::{alert, is_toc};
use drawn::{Drawn, Mode, draw};
pub use emoji::ShortcodeRun;
use html_block::html_block_text;
pub(crate) use sources::Escapes;
pub(crate) use sources::{inside, lines_at};
use written::{AUTO, IMAGE, MATH, NOTE, Written, quoted};

/// How a stretch of inline text prints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    /// Bold.
    Strong,
    /// Italic.
    Emphasis,
    /// Struck through.
    Strike,
    /// Inline code.
    Code,
    /// A link.
    Link,
    /// Inline math.
    Math,
    /// A footnote reference.
    Footnote,
    /// Underlined.
    Underline,
    /// A highlighted run, as `<mark>`.
    Mark,
}

/// Inline text, its styled stretches and link addresses.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Inline {
    /// The text as drawn.
    pub text: String,
    /// Styled stretches, in bytes of `text`.
    pub styles: Vec<(Range<usize>, Style)>,
    /// Link addresses, in bytes of `text`.
    pub links: Vec<(Range<usize>, String)>,
    /// Where each piece was written: drawn start, source span.
    pub sources: Vec<(usize, Range<usize>, Escapes)>,
}

/// A block of a markdown document.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// A paragraph.
    Paragraph(Inline),
    /// A heading of this level.
    Heading(u8, Inline),
    /// A quote and its blocks.
    Quote(Vec<Node>),
    /// A code block.
    Code {
        /// Its language, when named.
        language: Option<String>,
        /// Its text.
        text: String,
        /// Where its lines start in the source.
        from: usize,
    },
    /// Items and blocks; ordered from `start`; tasks carry checkboxes.
    List {
        /// The first item's number, when ordered.
        start: Option<u64>,
        /// Each item's task box, if any, and its blocks.
        items: Vec<(Option<bool>, Vec<Node>)>,
    },
    /// Each cell's blocks: its text, split at pictures.
    Table {
        /// Each column's alignment.
        aligns: Vec<AlignKind>,
        /// The head row's cells.
        head: Vec<Vec<Node>>,
        /// The body rows' cells.
        rows: Vec<Vec<Vec<Node>>>,
    },
    /// A rule.
    Rule,
    /// A math block's TeX.
    Math(String),
    /// A picture, and the link around it.
    Image {
        /// Its alt text.
        alt: String,
        /// Its address.
        url: String,
        /// The address it links to.
        link: Option<String>,
        /// Its source start.
        at: usize,
    },
    /// A footnote definition.
    Footnote {
        /// Its label.
        label: String,
        /// Its blocks.
        body: Vec<Node>,
    },
    /// A GitHub alert: a quote under its kind's title.
    Alert {
        /// Its kind.
        kind: AlertKind,
        /// Its blocks after the marker.
        body: Vec<Node>,
    },
    /// `[toc]`: the file's headings.
    Toc(Vec<DocumentHeading>),
    /// Front matter, its fences cut; lines start at `from`.
    FrontMatter {
        /// Its text.
        text: String,
        /// Where its lines start in the source.
        from: usize,
    },
}

/// A tree read against its source and its link definitions.
pub struct Reader<'a> {
    source: &'a str,
    urls: HashMap<String, String>,
    /// Footnotes numbered by their first reference.
    notes: HashMap<String, usize>,
    /// Every heading, as `[toc]` lists them.
    pub headings: Vec<DocumentHeading>,
    /// Soft breaks as lines, as the screen draws them.
    lines: bool,
}

/// `markdown` as blocks.
pub(crate) fn parse(markdown: &str) -> Vec<Node> {
    let root = tree(markdown);
    let (reader, mut out) = (Reader::new(markdown, &root), Vec::new());
    for node in root.children().expect("the root holds the blocks") {
        reader.top(node, &mut out);
    }
    out
}

impl<'a> Reader<'a> {
    /// Reads `root`, parsed from `source`, as drawn on screen.
    pub(crate) fn new(source: &'a str, root: &mdast::Node) -> Self {
        Self::reading(source, root, true)
    }

    /// The same, soft breaks joined as GFM output joins them.
    pub fn joined(source: &'a str, root: &mdast::Node) -> Self {
        Self::reading(source, root, false)
    }

    fn reading(source: &'a str, root: &mdast::Node, lines: bool) -> Self {
        let mut nodes = Vec::new();
        walk(root, &mut nodes);
        let (mut urls, mut notes, mut headings) = (HashMap::new(), HashMap::new(), Vec::new());
        for node in nodes {
            match node {
                mdast::Node::Definition(definition) => {
                    urls.entry(definition.identifier.clone())
                        .or_insert_with(|| definition.url.clone());
                }
                mdast::Node::FootnoteReference(note) => {
                    let next = notes.len() + 1;
                    notes.entry(note.identifier.clone()).or_insert(next);
                }
                mdast::Node::Heading(heading) => headings.push((heading, span(node).start)),
                _ => {}
            }
        }
        let mut reader = Self {
            source,
            urls,
            notes,
            headings: Vec::new(),
            lines: false,
        };
        reader.headings = headings
            .into_iter()
            .map(|(heading, at)| DocumentHeading {
                depth: heading.depth,
                title: reader.inline(&heading.children).text,
                at,
            })
            .collect();
        // Titles read on one line; the body as asked.
        reader.lines = lines;
        reader
    }

    fn blocks(&self, nodes: &[mdast::Node], out: &mut Vec<Node>) {
        for node in nodes {
            self.block(node, out);
        }
    }

    fn nested(&self, nodes: &[mdast::Node]) -> Vec<Node> {
        let mut out = Vec::new();
        self.blocks(nodes, &mut out);
        out
    }

    /// A reference's address; the parser checked its definition.
    fn url(&self, identifier: &str) -> String {
        self.urls
            .get(identifier)
            .cloned()
            .expect("a reference names a definition")
    }

    /// `node`'s blocks, as the drawing reads them.
    pub fn block(&self, node: &mdast::Node, out: &mut Vec<Node>) {
        match node {
            mdast::Node::Paragraph(paragraph) => self.paragraph(node, &paragraph.children, out),
            mdast::Node::Heading(heading) => {
                let wrap = (
                    format!("<h{}>", heading.depth),
                    format!("</h{}>", heading.depth),
                );
                let (written, images) = self.write(&heading.children, false, false);
                let mut text = Inline::default();
                let mut after = Vec::new();
                let drawn = draw(
                    &written,
                    (&wrap.0, &wrap.1),
                    Mode::Inline,
                    self.lines,
                    false,
                );
                for piece in drawn {
                    match piece {
                        Drawn::Text(drawn) => text = drawn,
                        Drawn::Image(index, link) => after.extend(image(&images, index, link)),
                    }
                }
                out.push(Node::Heading(heading.depth, text));
                out.extend(after);
            }
            mdast::Node::Blockquote(quote) => out.push(Node::Quote(self.nested(&quote.children))),
            mdast::Node::Code(code) => out.push(Node::Code {
                language: code.lang.clone(),
                text: code.value.clone(),
                from: sources::body(self.source, node),
            }),
            mdast::Node::Math(math) => out.push(Node::Math(math.value.clone())),
            mdast::Node::List(list) => {
                let items = list
                    .children
                    .iter()
                    .map(|item| match item {
                        mdast::Node::ListItem(item) => (item.checked, self.nested(&item.children)),
                        other => unreachable!("a list holds items, not {other:?}"),
                    })
                    .collect();
                let start = list.ordered.then(|| u64::from(list.start.unwrap_or(1)));
                out.push(Node::List { start, items });
            }
            mdast::Node::Table(table) => {
                let mut rows = table.children.iter().map(|row| match row {
                    // Cells past the head's columns stay undrawn, as on GitHub.
                    mdast::Node::TableRow(row) => (row.children.iter())
                        .take(table.align.len())
                        .map(|cell| {
                            let mut blocks = Vec::new();
                            let inlines = cell.children().expect("a cell holds inlines");
                            self.paragraph(cell, inlines, &mut blocks);
                            blocks
                        })
                        .collect::<Vec<_>>(),
                    other => unreachable!("a table holds rows, not {other:?}"),
                });
                let head = rows.next().expect("a table has its head row");
                out.push(Node::Table {
                    aligns: table.align.clone(),
                    head,
                    rows: rows.collect(),
                });
            }
            mdast::Node::ThematicBreak(_) => out.push(Node::Rule),
            mdast::Node::FootnoteDefinition(note) => out.push(Node::Footnote {
                label: self.label(note),
                body: self.nested(&note.children),
            }),
            mdast::Node::Html(html) => {
                text_block(html_block_text(&html.value, self.source, span(node)), out)
            }
            mdast::Node::Yaml(mdast::Literal { value, .. })
            | mdast::Node::Toml(mdast::Literal { value, .. }) => out.push(Node::FrontMatter {
                text: value.clone(),
                from: sources::body(self.source, node),
            }),
            mdast::Node::Definition(_) => {}
            other => unreachable!("{other:?} is no block outside MDX"),
        }
    }

    /// Text splits at images; a lone formula is block.
    fn paragraph(&self, node: &mdast::Node, children: &[mdast::Node], out: &mut Vec<Node>) {
        if let [mdast::Node::InlineMath(math)] = children
            && ["$$", "\\["]
                .iter()
                .any(|open| self.source[span(&children[0])].starts_with(open))
        {
            return out.push(Node::Math(math.value.trim().to_string()));
        }
        let table = matches!(node, mdast::Node::TableCell(_));
        let (written, images) = self.write(children, table, false);
        // GitHub writes a cell's text inside its table.
        let wrap = match table {
            true => ("<table><tbody><tr><td>", "</td></tr></tbody></table>"),
            false => ("<p>", "</p>"),
        };
        for piece in draw(&written, wrap, Mode::Inline, self.lines, true) {
            match piece {
                Drawn::Text(text) => text_block(text, out),
                Drawn::Image(index, link) => out.extend(image(&images, index, link)),
            }
        }
    }

    /// A footnote's number, by its first reference.
    pub fn note(&self, identifier: &str) -> Option<usize> {
        self.notes.get(identifier).copied()
    }

    /// A note's label: its number, once referenced.
    pub fn label(&self, note: &mdast::FootnoteDefinition) -> String {
        match self.note(&note.identifier) {
            Some(number) => number.to_string(),
            None => note.label.clone(),
        }
    }

    /// Inline text where images draw as their alt text.
    pub fn inline(&self, nodes: &[mdast::Node]) -> Inline {
        let (written, _) = self.write(nodes, false, true);
        let drawn = draw(&written, ("<p>", "</p>"), Mode::Inline, self.lines, false);
        let mut text = Inline::default();
        for piece in drawn {
            if let Drawn::Text(drawn) = piece {
                text = drawn;
            }
        }
        text
    }

    /// Inline nodes as the HTML GitHub writes, images set aside.
    fn write(&self, nodes: &[mdast::Node], table: bool, alt: bool) -> (Written, Vec<Picture>) {
        let (mut written, mut images) = (Written::default(), Vec::new());
        for node in nodes {
            self.write_into(node, &mut written, &mut images, (false, table, alt));
        }
        (written, images)
    }

    /// One node; `how`: in autolink, in cell, images as alt.
    fn write_into(
        &self,
        node: &mdast::Node,
        out: &mut Written,
        images: &mut Vec<Picture>,
        how: (bool, bool, bool),
    ) {
        let (verbatim, table, alt) = how;
        if let Some((written, escapes)) = sources::written(node, self.source) {
            let escapes = match (escapes, node) {
                (Escapes::Markdown, _) if verbatim => Escapes::Plain,
                (_, mdast::Node::InlineCode(_)) if table => Escapes::Pipes,
                _ => escapes,
            };
            out.chunk(written, escapes);
        }
        let (children, name, attributes) = match node {
            mdast::Node::Text(words) => return out.text(&words.value),
            mdast::Node::Break(_) => return out.html.push_str("<br>"),
            mdast::Node::InlineCode(code) => return wrapped(out, "code", "", &code.value),
            mdast::Node::InlineMath(math) => {
                return wrapped(out, "span", &format!(" {MATH}"), &math.value);
            }
            mdast::Node::FootnoteReference(note) => {
                // GitHub writes an `a`: it ends the open one.
                let number = html::superscript(self.notes[&note.identifier]);
                return wrapped(out, "a", &format!(" {NOTE}"), &number);
            }
            mdast::Node::Image(_) | mdast::Node::ImageReference(_) => {
                let (alt_text, url) = match node {
                    mdast::Node::Image(image) => (image.alt.clone(), image.url.clone()),
                    mdast::Node::ImageReference(image) => {
                        (image.alt.clone(), self.url(&image.identifier))
                    }
                    _ => unreachable!("matched as an image"),
                };
                if alt {
                    return out.text(&alt_text);
                }
                let index = images.len();
                images.push(Picture {
                    alt: alt_text,
                    url,
                    at: span(node).start,
                });
                return out.html.push_str(&format!("<img {IMAGE}=\"{index}\">"));
            }
            mdast::Node::Html(written) => return out.raw(&written.value),
            mdast::Node::Strong(strong) => (&strong.children, "strong", String::new()),
            mdast::Node::Emphasis(emphasis) => (&emphasis.children, "em", String::new()),
            mdast::Node::Delete(delete) => (&delete.children, "del", String::new()),
            mdast::Node::Link(link) => {
                let auto = !self.source[span(node)].starts_with('[');
                let tail = if auto {
                    format!(" {AUTO}")
                } else {
                    String::new()
                };
                (
                    &link.children,
                    "a",
                    format!(" href=\"{}\"{tail}", quoted(&link.url)),
                )
            }
            mdast::Node::LinkReference(link) => {
                let url = quoted(&self.url(&link.identifier));
                (&link.children, "a", format!(" href=\"{url}\""))
            }
            other => unreachable!("{other:?} is no inline outside MDX"),
        };
        // A container is the element GitHub writes.
        out.html.push_str(&format!("<{name}{attributes}>"));
        let autolink =
            matches!(node, mdast::Node::Link(_)) && !self.source[span(node)].starts_with('[');
        for child in children {
            self.write_into(child, out, images, (verbatim || autolink, table, alt));
        }
        out.html.push_str(&format!("</{name}>"));
    }
}

/// An image set aside while its text was written.
#[derive(Clone)]
struct Picture {
    alt: String,
    url: String,
    at: usize,
}

/// The image numbered `index`, linked by `link`.
fn image(images: &[Picture], index: usize, link: Option<String>) -> Option<Node> {
    let Some(picture) = images.get(index) else {
        log::error!("markdown: an image placeholder names no image");
        return None;
    };
    let Picture { alt, url, at } = picture.clone();
    Some(Node::Image { alt, url, link, at })
}

/// `text` inside a `name` element with `attributes`.
fn wrapped(out: &mut Written, name: &str, attributes: &str, text: &str) {
    out.html.push_str(&format!("<{name}{attributes}>"));
    out.text(text);
    out.html.push_str(&format!("</{name}>"));
}

/// Text left between images, unless it is blank.
fn text_block(text: Inline, out: &mut Vec<Node>) {
    if !text.text.trim().is_empty() {
        out.push(Node::Paragraph(text));
    }
}

#[cfg(test)]
mod tests;
