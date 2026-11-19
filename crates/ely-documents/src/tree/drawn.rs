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

/// White space owed before the next character.
#[derive(Clone, Copy, PartialEq)]
enum Owed {
    Nothing,
    Space,
    /// Line ends a block owes; they merge.
    Lines(usize),
}

/// A walk step: enter a node, or leave an element.
enum Step<'a> {
    /// A node; true for a cell another follows.
    Enter(usize, Within<'a>, bool),
    /// Line ends owed after it; a tab after a cell.
    Leave { lines: usize, tab: bool },
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
    owed: Owed,
    /// Text a break keeps; spaces after it hide.
    solid: usize,
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
        owed: Owed::Nothing,
        solid: 0,
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
            .map(|&node| Step::Enter(node, Within::default(), false))
            .collect();
        while let Some(step) = steps.pop() {
            let (node, within, tab) = match step {
                Step::Enter(node, within, tab) => (node, within, tab),
                Step::Leave { lines, tab } => {
                    if let Some(open) = self.open.pop() {
                        self.close(&open);
                    }
                    self.unstarted = self.unstarted.min(self.open.len());
                    self.shortcodes.split();
                    self.line(lines);
                    if tab {
                        self.tab();
                    }
                    continue;
                }
            };
            if dom.veiled && !html::shown(dom, node) {
                self.lead = None;
                self.shortcodes.split();
                continue;
            }
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
                    let Some((within, lines)) = self.enter(node, within) else {
                        continue;
                    };
                    steps.push(Step::Leave { lines, tab });
                    let children = &dom.nodes[node].children;
                    // Hidden cells owe no tab, nor take one.
                    let cell = |child: usize| {
                        matches!(dom.name(child), "td" | "th")
                            && (!dom.veiled || html::shown(dom, child))
                    };
                    // Cells but a row's last owe a tab.
                    let last = match dom.name(node) {
                        "tr" => children.iter().rposition(|&child| cell(child)),
                        _ => None,
                    };
                    steps.extend(children.iter().enumerate().rev().map(|(at, &child)| {
                        let tab = last.is_some_and(|last| at < last) && cell(child);
                        Step::Enter(child, within, tab)
                    }));
                }
            }
        }
    }

    /// Opens an element, owing lines; `None` if nothing draws.
    fn enter<'d>(&mut self, node: usize, within: Within<'d>) -> Option<(Within<'d>, usize)>
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
            // A picture is ink: spaces before it stay.
            self.solid = self.text.text.len();
            self.out
                .push(Drawn::Image(index, within.link.map(str::to_string)));
            return None;
        }
        if dom.veiled && html::replaced(name) {
            self.pay();
            return None;
        }
        // Paragraphs part with a blank line, as innerText.
        let lines = match name {
            "p" => 2,
            _ => usize::from(html::breaks(name)),
        };
        self.line(lines);
        if name == "br" {
            self.owed = Owed::Nothing;
            self.trim();
            self.push('\n', true);
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
        let within = Within {
            raw,
            pre: within.pre || matches!(name, "pre" | "listing"),
            link: url.or(within.link),
        };
        Some((within, lines))
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
                let kept = raw || within.pre;
                let mut shown = shown.as_str();
                if self.owed != Owed::Nothing {
                    // A break hides the white space after it.
                    if !kept {
                        shown = shown.trim_start_matches(html::spacing);
                    }
                    if shown.is_empty() {
                        return;
                    }
                    self.pay();
                }
                let loose = match kept {
                    true => 0,
                    false => shown.len() - shown.trim_end_matches(html::spacing).len(),
                };
                let mut chars = shown.chars();
                if let Some(first) = chars.next() {
                    self.push(first, false);
                    self.text.text.push_str(chars.as_str());
                }
                if loose < shown.len() {
                    self.solid = self.text.text.len() - loose;
                }
            }
            Mode::Block => {
                for ch in shown.chars() {
                    if within.pre || !html::spacing(ch) {
                        self.pay();
                        self.push(ch, true);
                    } else if self.owed == Owed::Nothing
                        && !(self.text.text.is_empty() || self.text.text.ends_with(['\n', '\t']))
                    {
                        self.owed = Owed::Space;
                    }
                }
            }
        }
    }

    /// Draws `ch`; elements still empty start before it.
    fn push(&mut self, ch: char, solid: bool) {
        let at = self.text.text.len();
        for open in &mut self.open[self.unstarted..] {
            open.start = Some(at);
        }
        self.unstarted = self.open.len();
        self.text.text.push(ch);
        if solid {
            self.solid = self.text.text.len();
        }
    }

    /// `count` line ends owed where a block starts or ends.
    fn line(&mut self, count: usize) {
        if count == 0 || self.text.text.is_empty() {
            return;
        }
        // Owed breaks merge; a `br`'s own line does not.
        self.owed = match self.owed {
            Owed::Lines(had) => Owed::Lines(had.max(count)),
            _ => Owed::Lines(count),
        };
    }

    /// A tab between cells, written at once.
    fn tab(&mut self) {
        match self.owed {
            Owed::Lines(_) => self.pay(),
            _ => {
                self.owed = Owed::Nothing;
                self.trim();
            }
        }
        self.text.text.push('\t');
        self.solid = self.text.text.len();
    }

    /// Writes owed white space; it starts no element.
    fn pay(&mut self) {
        match std::mem::replace(&mut self.owed, Owed::Nothing) {
            Owed::Nothing => return,
            Owed::Space => self.text.text.push(' '),
            Owed::Lines(count) => {
                self.trim();
                self.text.text.extend(std::iter::repeat_n('\n', count));
            }
        }
        self.solid = self.text.text.len();
    }

    /// Drops the spaces before a break, as a browser does.
    fn trim(&mut self) {
        let kept = self.solid;
        if kept == self.text.text.len() {
            return;
        }
        self.text.text.truncate(kept);
        clamp(&mut self.text.styles, kept);
        clamp(&mut self.text.links, kept);
        let moved = self.text.sources.partition_point(|(at, ..)| *at <= kept);
        for source in &mut self.text.sources[moved..] {
            source.0 = kept;
        }
        // Elements that started in the spaces start again.
        let open = &mut self.open[..self.unstarted];
        let from = open.partition_point(|open| open.start.is_some_and(|start| start < kept));
        for open in &mut open[from..] {
            open.start = None;
        }
        self.unstarted = from;
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
        if let Owed::Lines(_) = self.owed {
            self.trim();
        }
        let end = self.text.text.len();
        for ix in (0..self.open.len()).rev() {
            if let Some(start) = self.open[ix].start.take().filter(|start| *start < end) {
                let (style, url) = (self.open[ix].style, self.open[ix].url.clone());
                self.record(start..end, style, url);
            }
        }
        self.unstarted = 0;
        self.owed = Owed::Nothing;
        self.solid = 0;
        let mut done = std::mem::take(&mut self.text);
        // By start, the wider first: the walk's order is incidental.
        done.styles
            .sort_by_key(|(range, _)| (range.start, std::cmp::Reverse(range.end)));
        done.links
            .sort_by_key(|(range, _)| (range.start, std::cmp::Reverse(range.end)));
        done
    }
}

/// Ranges, by end, cut at `end`; emptied ones go.
fn clamp<T>(ranges: &mut Vec<(Range<usize>, T)>, end: usize) {
    let from = ranges.partition_point(|(range, _)| range.end <= end);
    let cut = ranges.split_off(from);
    ranges.extend(
        cut.into_iter()
            .filter(|(range, _)| range.start < end)
            .map(|(range, it)| (range.start..end, it)),
    );
}
