//! What html5ever's tree draws: text, styles, links and sources.

use std::ops::Range;

use super::{
    Escapes, Inline, Style,
    dom::{Data, Dom, fragment},
    emoji::Shortcodes,
    html,
    written::{AUTO, CLOSE, DIGITS, ESC, IMAGE, MATH, NOTE, OPEN, Written},
};

/// How the tree's text lays out.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Mode {
    /// Markdown's inline text: written as is.
    Inline,
    /// An HTML block: white space collapsed, blocks on lines.
    Block,
}

/// A piece of drawn output, in order.
pub(super) enum Drawn {
    Text(Inline),
    /// An image placeholder's number and the link around it.
    Image(usize, Option<String>),
}

/// An element open around the text.
struct Open {
    /// Where its first character went, once one did.
    start: Option<usize>,
    style: Option<Style>,
    url: Option<String>,
}

/// What the elements around a node make of its text.
#[derive(Clone, Copy, Default)]
struct Within<'a> {
    /// Shortcodes and line breaks stay as written.
    raw: bool,
    /// White space stays as written.
    pre: bool,
    link: Option<&'a str>,
}

/// A walk step: enter a node, or leave an element.
enum Step<'a> {
    Enter(usize, Within<'a>),
    Leave { block: bool },
}

struct Draw<'a> {
    dom: &'a Dom,
    chunks: &'a [(Range<usize>, Escapes)],
    mode: Mode,
    /// Soft breaks as lines, not spaces.
    lines: bool,
    /// Images cut the text; else they only follow.
    split: bool,
    text: Inline,
    out: Vec<Drawn>,
    shortcodes: Shortcodes,
    owed: Option<char>,
    open: Vec<Open>,
    /// Open elements from here on have no character yet.
    unstarted: usize,
    /// Where an index's mark is being read.
    reading: Option<usize>,
    /// The next character is the source's own.
    literal: bool,
    /// A `pre` just opened; true once a mark followed it.
    lead: Option<bool>,
}

/// `written` between `wrap`'s tags, as GitHub's page draws it.
pub(super) fn draw(
    written: &Written,
    wrap: (&str, &str),
    mode: Mode,
    lines: bool,
    split: bool,
) -> Vec<Drawn> {
    let dom = fragment(&[wrap.0, &written.html, wrap.1].concat(), "body");
    let mut draw = Draw {
        dom: &dom,
        chunks: &written.chunks,
        mode,
        lines,
        split,
        text: Inline::default(),
        out: Vec::new(),
        shortcodes: Shortcodes::default(),
        owed: None,
        open: Vec::new(),
        unstarted: 0,
        reading: None,
        literal: false,
        lead: None,
    };
    draw.walk(dom.top());
    let last = draw.cut();
    draw.out.push(Drawn::Text(last));
    draw.out
}

impl<'a> Draw<'a> {
    /// The tree in order, without recursion: deep HTML stays safe.
    fn walk(&mut self, top: &[usize]) {
        let dom = self.dom;
        let mut steps: Vec<Step> = top
            .iter()
            .rev()
            .map(|&node| Step::Enter(node, Within::default()))
            .collect();
        while let Some(step) = steps.pop() {
            let (node, within) = match step {
                Step::Enter(node, within) => (node, within),
                Step::Leave { block } => {
                    if let Some(open) = self.open.pop() {
                        self.close(&open);
                    }
                    self.unstarted = self.unstarted.min(self.open.len());
                    self.shortcodes.split();
                    if block {
                        self.line();
                    }
                    continue;
                }
            };
            match &dom.nodes[node].data {
                Data::Text(text) => {
                    self.words(text, within);
                    self.shortcodes.split();
                }
                Data::Document | Data::Other => {
                    self.lead = None;
                    self.shortcodes.split();
                }
                Data::Element { .. } => {
                    let Some(within) = self.enter(node, within) else {
                        continue;
                    };
                    let block = self.mode == Mode::Block && html::breaks(dom.name(node));
                    steps.push(Step::Leave { block });
                    let children = &dom.nodes[node].children;
                    steps.extend(
                        children
                            .iter()
                            .rev()
                            .map(|&child| Step::Enter(child, within)),
                    );
                }
            }
        }
    }

    /// Opens an element; `None` when nothing inside it draws.
    fn enter<'d>(&mut self, node: usize, within: Within<'d>) -> Option<Within<'d>>
    where
        'a: 'd,
    {
        let dom = self.dom;
        let name = dom.name(node);
        self.shortcodes.split();
        self.lead = matches!(name, "pre" | "listing").then_some(false);
        if let Some(index) = dom.attribute(node, IMAGE) {
            let Ok(index) = index.parse() else {
                log::error!("markdown: an image placeholder lost its number");
                return None;
            };
            if self.split {
                let text = self.cut();
                self.out.push(Drawn::Text(text));
            }
            self.out
                .push(Drawn::Image(index, within.link.map(str::to_string)));
            return None;
        }
        // A template's contents never draw.
        if name == "template" {
            return None;
        }
        if self.mode == Mode::Block && html::breaks(name) {
            self.line();
        }
        if name == "br" {
            self.owed = None;
            self.push('\n');
            return None;
        }
        let (style, url) = if dom.attribute(node, NOTE).is_some() {
            (Some(Style::Footnote), None)
        } else if dom.attribute(node, MATH).is_some() {
            (Some(Style::Math), None)
        } else {
            match (name, dom.attribute(node, "href")) {
                ("a", None) => (None, None),
                ("a", Some(href)) => (Some(Style::Link), Some(href)),
                _ => (html::inline_style(name), None),
            }
        };
        let raw = within.raw
            || matches!(name, "code" | "pre")
            || style == Some(Style::Math)
            || dom.attribute(node, AUTO).is_some();
        self.open.push(Open {
            start: None,
            style,
            url: url.map(str::to_string),
        });
        Some(Within {
            raw,
            pre: within.pre || matches!(name, "pre" | "listing"),
            link: url.or(within.link),
        })
    }

    /// A text node: marks note sources, the rest draws.
    fn words(&mut self, text: &str, within: Within) {
        let mut segment = String::new();
        for ch in text.chars() {
            if std::mem::take(&mut self.literal) {
                self.lead = None;
                segment.push(ch);
                continue;
            }
            match (ch, self.reading) {
                (ESC, None) => self.literal = true,
                (OPEN, _) => {
                    self.lead = self.lead.map(|_| true);
                    self.segment(&std::mem::take(&mut segment), within);
                    self.reading = Some(0);
                }
                (CLOSE, Some(index)) => {
                    self.reading = None;
                    match self.chunks.get(index) {
                        Some((span, escapes)) => {
                            let at = self.text.text.len();
                            self.text.sources.push((at, span.clone(), *escapes));
                        }
                        None => log::error!("markdown: a mark names no chunk"),
                    }
                }
                (digit, Some(index)) => match DIGITS.iter().position(|d| *d == digit) {
                    Some(value) => self.reading = Some(index * 16 + value),
                    // The source held a noncharacter like ours.
                    None => {
                        log::warn!("markdown: a noncharacter in the text broke a mark");
                        self.lead = None;
                        self.reading = None;
                        segment.push(digit);
                    }
                },
                // Our mark hid the line feed HTML drops.
                ('\n', None) if self.lead.take() == Some(true) => {}
                (ch, None) => {
                    self.lead = None;
                    segment.push(ch);
                }
            }
        }
        self.segment(&segment, within);
    }

    fn segment(&mut self, written: &str, within: Within) {
        if written.is_empty() {
            return;
        }
        let raw = within.raw;
        let shown = match raw {
            true => written.to_string(),
            false => self.shortcodes.read(written),
        };
        match self.mode {
            Mode::Inline => {
                let soft = if self.lines || raw { "\n" } else { " " };
                let shown = match raw {
                    true => shown,
                    false => shown.replace("\r\n", soft).replace(['\r', '\n'], soft),
                };
                for ch in shown.chars() {
                    self.push(ch);
                }
            }
            Mode::Block => {
                for ch in shown.chars() {
                    if within.pre || !html::spacing(ch) {
                        self.pay();
                        self.push(ch);
                    } else if self.owed.is_none() && !self.text.text.ends_with('\n') {
                        self.owed = (!self.text.text.is_empty()).then_some(' ');
                    }
                }
            }
        }
    }

    /// Draws `ch`; elements still empty start before it.
    fn push(&mut self, ch: char) {
        let at = self.text.text.len();
        for open in &mut self.open[self.unstarted..] {
            open.start = Some(at);
        }
        self.unstarted = self.open.len();
        self.text.text.push(ch);
    }

    /// A line owed where a block element starts or ends.
    fn line(&mut self) {
        let text = &self.text.text;
        self.owed = (!(text.is_empty() || text.ends_with('\n'))).then_some('\n');
    }

    /// Writes owed white space; it starts no element.
    fn pay(&mut self) {
        if let Some(owed) = self.owed.take() {
            self.text.text.push(owed);
        }
    }

    fn close(&mut self, open: &Open) {
        let end = self.text.text.len();
        if let Some(start) = open.start.filter(|start| *start < end) {
            self.record(start..end, open.style, open.url.clone());
        }
    }

    /// One range per element: they nest as the tree.
    fn record(&mut self, range: std::ops::Range<usize>, style: Option<Style>, url: Option<String>) {
        if let Some(style) = style {
            self.text.styles.push((range.clone(), style));
        }
        if let Some(url) = url {
            self.text.links.push((range, url));
        }
    }

    /// The text so far; open elements carry on after it.
    fn cut(&mut self) -> Inline {
        let end = self.text.text.len();
        for ix in (0..self.open.len()).rev() {
            if let Some(start) = self.open[ix].start.take().filter(|start| *start < end) {
                let (style, url) = (self.open[ix].style, self.open[ix].url.clone());
                self.record(start..end, style, url);
            }
        }
        self.unstarted = 0;
        self.owed = None;
        let mut done = std::mem::take(&mut self.text);
        // By start, the wider first: the walk's order is incidental.
        done.styles
            .sort_by_key(|(range, _)| (range.start, std::cmp::Reverse(range.end)));
        done.links
            .sort_by_key(|(range, _)| (range.start, std::cmp::Reverse(range.end)));
        done
    }
}
