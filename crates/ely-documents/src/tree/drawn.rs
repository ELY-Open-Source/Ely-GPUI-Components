//! What html5ever's tree draws: text, styles, links and sources.

use std::ops::Range;

use super::{
    Escapes, Inline, Style,
    dom::{Data, Dom, fragment},
    emoji::Shortcodes,
    html,
};

/// A chunk's mark opens with this; hex digits follow.
const OPEN: char = '\u{FDD0}';
const CLOSE: char = '\u{FDD1}';
/// Before a reserved character the source itself holds.
const ESC: char = '\u{FDD2}';

/// Characters the marks use; the source's own are escaped.
pub(super) fn reserved(ch: char) -> bool {
    ('\u{FDD0}'..='\u{FDEF}').contains(&ch)
}
/// Noncharacters for the hex digits of a chunk's index.
const DIGITS: [char; 16] = [
    '\u{FDE0}', '\u{FDE1}', '\u{FDE2}', '\u{FDE3}', '\u{FDE4}', '\u{FDE5}', '\u{FDE6}', '\u{FDE7}',
    '\u{FDE8}', '\u{FDE9}', '\u{FDEA}', '\u{FDEB}', '\u{FDEC}', '\u{FDED}', '\u{FDEE}', '\u{FDEF}',
];

/// Elements open at most; deeper start tags go as text.
///
/// HTML5's scope checks walk the open elements: this bounds them.
const DEEPEST: usize = 1024;

/// Elements a later start tag may end, and voids: not counted.
fn ends_itself(name: &str) -> bool {
    matches!(
        name,
        "li" | "p"
            | "dd"
            | "dt"
            | "option"
            | "optgroup"
            | "tr"
            | "td"
            | "th"
            | "thead"
            | "tbody"
            | "tfoot"
            | "rb"
            | "rt"
            | "rtc"
            | "rp"
            | "colgroup"
            | "caption"
            | "html"
            | "head"
            | "body"
            | "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "source"
            | "track"
            | "wbr"
    )
}

/// HTML written for html5ever, and where its chunks were written.
#[derive(Default)]
pub(super) struct Written {
    pub html: String,
    chunks: Vec<(Range<usize>, Escapes)>,
    /// Counted elements open, by name, and in all.
    open: std::collections::HashMap<String, usize>,
    depth: usize,
    /// Start tags past `DEEPEST` went as text.
    capped: bool,
}

impl Written {
    /// Marks where a chunk written at `span` starts drawing.
    pub(super) fn chunk(&mut self, span: Range<usize>, escapes: Escapes) {
        self.html.push(OPEN);
        let index = format!("{:x}", self.chunks.len());
        for digit in index.bytes() {
            let value = match digit {
                b'0'..=b'9' => digit - b'0',
                _ => digit - b'a' + 10,
            };
            self.html.push(DIGITS[usize::from(value)]);
        }
        self.html.push(CLOSE);
        self.chunks.push((span, escapes));
    }

    /// Text, escaped so the parser reads it as text.
    pub(super) fn text(&mut self, text: &str) {
        self.html.push_str(&escaped(text));
    }

    /// Markup as written; GFM's tag filter shows some as text.
    pub(super) fn raw(&mut self, html: &str) {
        let Some(tag) = html::tag(html) else {
            return self.html.push_str(html);
        };
        if html::filtered(&tag.name) {
            return self.text(html);
        }
        if !ends_itself(&tag.name) {
            if tag.closing {
                if let Some(count) = self.open.get_mut(&tag.name).filter(|count| **count > 0) {
                    *count -= 1;
                    self.depth -= 1;
                }
            } else if !html.trim_end().ends_with("/>") {
                if self.depth >= DEEPEST {
                    if !self.capped {
                        log::warn!("markdown: HTML nested past {DEEPEST} elements reads as text");
                        self.capped = true;
                    }
                    return self.text(html);
                }
                *self.open.entry(tag.name).or_default() += 1;
                self.depth += 1;
            }
        }
        self.html.push_str(html);
    }
}

/// `text` safe as HTML text or a quoted attribute value.
pub(super) fn escaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            ch if reserved(ch) => out.extend([ESC, ch]),
            ch => out.push(ch),
        }
    }
    out
}

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

/// One step of the walk: a node to read, or an element to close.
enum Step<'a> {
    Enter(usize, bool, Option<&'a str>),
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
}

/// `written` between `wrap`'s tags, as GitHub's page draws it.
pub(super) fn draw(
    written: &Written,
    wrap: (&str, &str),
    mode: Mode,
    lines: bool,
    split: bool,
) -> Vec<Drawn> {
    let dom = fragment(&format!("{}{}{}", wrap.0, written.html, wrap.1), "body");
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
            .map(|&node| Step::Enter(node, false, None))
            .collect();
        while let Some(step) = steps.pop() {
            let (node, raw, link) = match step {
                Step::Enter(node, raw, link) => (node, raw, link),
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
                    self.words(text, raw);
                    self.shortcodes.split();
                }
                Data::Document | Data::Other => self.shortcodes.split(),
                Data::Element { .. } => {
                    let Some((raw, link)) = self.enter(node, raw, link) else {
                        continue;
                    };
                    let block = self.mode == Mode::Block && html::breaks(dom.name(node));
                    steps.push(Step::Leave { block });
                    let children = &dom.nodes[node].children;
                    steps.extend(
                        children
                            .iter()
                            .rev()
                            .map(|&child| Step::Enter(child, raw, link)),
                    );
                }
            }
        }
    }

    /// Opens an element; `None` when nothing inside it draws.
    fn enter<'d>(
        &mut self,
        node: usize,
        raw: bool,
        link: Option<&'d str>,
    ) -> Option<(bool, Option<&'d str>)>
    where
        'a: 'd,
    {
        let dom = self.dom;
        let name = dom.name(node);
        self.shortcodes.split();
        if let Some(index) = dom.attribute(node, "data-ely-image") {
            let Ok(index) = index.parse() else {
                log::error!("markdown: an image placeholder lost its number");
                return None;
            };
            if self.split {
                let text = self.cut();
                self.out.push(Drawn::Text(text));
            }
            self.out.push(Drawn::Image(index, link.map(str::to_string)));
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
        let (style, url) = if dom.attribute(node, "data-ely-note").is_some() {
            (Some(Style::Footnote), None)
        } else if dom.attribute(node, "data-ely-math").is_some() {
            (Some(Style::Math), None)
        } else {
            match (name, dom.attribute(node, "href")) {
                ("a", None) => (None, None),
                ("a", Some(href)) => (Some(Style::Link), Some(href)),
                _ => (html::inline_style(name), None),
            }
        };
        let raw = raw
            || matches!(name, "code" | "pre")
            || style == Some(Style::Math)
            || dom.attribute(node, "data-ely-auto").is_some();
        self.open.push(Open {
            start: None,
            style,
            url: url.map(str::to_string),
        });
        Some((raw, url.or(link)))
    }

    /// A text node: marks note sources, the rest draws.
    fn words(&mut self, text: &str, raw: bool) {
        let mut segment = String::new();
        for ch in text.chars() {
            if std::mem::take(&mut self.literal) {
                segment.push(ch);
                continue;
            }
            match (ch, self.reading) {
                (ESC, None) => self.literal = true,
                (OPEN, _) => {
                    self.segment(&std::mem::take(&mut segment), raw);
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
                        self.reading = None;
                        segment.push(digit);
                    }
                },
                (ch, None) => segment.push(ch),
            }
        }
        self.segment(&segment, raw);
    }

    fn segment(&mut self, written: &str, raw: bool) {
        if written.is_empty() {
            return;
        }
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
                    if !html::spacing(ch) {
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

    /// Writes the white space owed before a character.
    ///
    /// It belongs to no element that has not drawn yet.
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

    /// Records an element's style and link over `range`.
    ///
    /// One range per element: they nest, as the tree does.
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

/// Shortcodes for an owner writing its own HTML.
///
/// One value per block: `split` at every element, `text` on text,
/// `html` on inline HTML. Text inside `code` or `pre` stays as written.
#[derive(Default)]
pub struct ShortcodeRun {
    shortcodes: Shortcodes,
    /// How many `code` and `pre` are open.
    verbatim: usize,
}

impl ShortcodeRun {
    /// A new DOM text node starts.
    pub fn split(&mut self) {
        self.shortcodes.split();
    }

    /// Text as shown; an open `code` or `pre` keeps it.
    pub fn text(&mut self, written: &str) -> String {
        match self.verbatim {
            0 => self.shortcodes.read(written),
            _ => written.to_string(),
        }
    }

    /// Inline HTML as written; a filtered tag shows as text.
    pub fn html(&mut self, written: &str) -> String {
        self.shortcodes.split();
        match html::tag(written) {
            Some(tag) if html::filtered(&tag.name) => self.text(written),
            Some(tag) if matches!(tag.name.as_str(), "code" | "pre") => {
                self.verbatim = match tag.closing {
                    true => self.verbatim.saturating_sub(1),
                    false => self.verbatim + 1,
                };
                written.to_string()
            }
            _ => written.to_string(),
        }
    }
}
