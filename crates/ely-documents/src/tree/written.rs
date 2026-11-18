//! The HTML written for html5ever, marked with its sources.

use std::{collections::HashMap, ops::Range};

use super::{Escapes, html};

/// A chunk's mark opens with this; hex digits follow.
pub(super) const OPEN: char = '\u{FDD0}';
pub(super) const CLOSE: char = '\u{FDD1}';
/// Before a reserved character the source's text holds.
pub(super) const ESC: char = '\u{FDD2}';
/// Noncharacters for the hex digits of a chunk's index.
pub(super) const DIGITS: [char; 16] = [
    '\u{FDE0}', '\u{FDE1}', '\u{FDE2}', '\u{FDE3}', '\u{FDE4}', '\u{FDE5}', '\u{FDE6}', '\u{FDE7}',
    '\u{FDE8}', '\u{FDE9}', '\u{FDEA}', '\u{FDEB}', '\u{FDEC}', '\u{FDED}', '\u{FDEE}', '\u{FDEF}',
];

/// Attribute names source HTML cannot write.
pub(super) const MATH: &str = "\u{FDD0}math";
pub(super) const NOTE: &str = "\u{FDD0}note";
pub(super) const IMAGE: &str = "\u{FDD0}image";
pub(super) const AUTO: &str = "\u{FDD0}auto";

/// Characters the marks use; the source's own are escaped.
pub(super) fn reserved(ch: char) -> bool {
    ('\u{FDD0}'..='\u{FDEF}').contains(&ch)
}

/// Open elements at most: HTML5's scope checks walk them.
const DEEPEST: usize = 1024;

/// Voids and elements later tags end: never counted.
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

/// SVG and MathML names HTML lacks: they stay foreign.
fn foreign(name: &str) -> bool {
    matches!(
        name,
        "g" | "path"
            | "rect"
            | "circle"
            | "ellipse"
            | "line"
            | "polyline"
            | "polygon"
            | "defs"
            | "use"
            | "symbol"
            | "stop"
            | "lineargradient"
            | "radialgradient"
            | "clippath"
            | "mask"
            | "pattern"
            | "marker"
            | "text"
            | "tspan"
            | "foreignobject"
            | "desc"
            | "mrow"
            | "mi"
            | "mo"
            | "mn"
            | "ms"
            | "mtext"
            | "mfrac"
            | "msup"
            | "msub"
            | "msqrt"
            | "mroot"
            | "annotation-xml"
    )
}

/// Foreign elements whose children are HTML again.
fn integrates(name: &str) -> bool {
    matches!(
        name,
        "foreignobject" | "desc" | "mi" | "mo" | "mn" | "ms" | "mtext" | "annotation-xml"
    )
}

/// HTML written for html5ever, and where its chunks were written.
#[derive(Default)]
pub(super) struct Written {
    pub html: String,
    pub(super) chunks: Vec<(Range<usize>, Escapes)>,
    /// Counted elements open, by name, and in all.
    open: HashMap<String, usize>,
    depth: usize,
    /// Inside SVG or MathML, so far as tags tell.
    foreign: bool,
    /// Start tags past `DEEPEST` went as text.
    capped: bool,
}

impl Written {
    /// Marks where a chunk written at `span` starts drawing.
    pub(super) fn chunk(&mut self, span: Range<usize>, escapes: Escapes) {
        self.html.push(OPEN);
        let index = self.chunks.len();
        // Hex digits, most significant first, as `{:x}` writes them.
        let mut shift = (usize::BITS - index.leading_zeros()).div_ceil(4).max(1) * 4;
        while shift > 0 {
            shift -= 4;
            self.html.push(DIGITS[(index >> shift) & 0xF]);
        }
        self.html.push(CLOSE);
        self.chunks.push((span, escapes));
    }

    /// Text, escaped so the parser reads it as text.
    pub(super) fn text(&mut self, text: &str) {
        escape(&mut self.html, text, |out, ch| out.extend([ESC, ch]));
    }

    /// Markup as written; GFM's tag filter shows some as text.
    pub(super) fn raw(&mut self, html: &str) {
        let Some(tag) = html::tag(html) else {
            return self.html.push_str(&referenced(html));
        };
        if html::filtered(&tag.name) {
            return self.text(html);
        }
        if tag.closing {
            self.close(&tag.name);
        } else if !self.opens(&tag.name, html.trim_end().ends_with("/>")) {
            return self.text(html);
        }
        self.html.push_str(&referenced(html));
    }

    fn close(&mut self, name: &str) {
        self.foreign &= foreign(name);
        if let Some(count) = self.open.get_mut(name).filter(|count| **count > 0) {
            *count -= 1;
            self.depth -= 1;
        }
    }

    /// Counts a start tag; `false` past the bound.
    fn opens(&mut self, name: &str, slash: bool) -> bool {
        let outer = std::mem::take(&mut self.foreign);
        let root = matches!(name, "svg" | "math");
        // HTML ignores the slash, but on voids and foreign elements.
        if slash && (root || (outer && foreign(name))) {
            self.foreign = outer;
            return true;
        }
        self.foreign = root || (outer && foreign(name) && !integrates(name));
        if ends_itself(name) {
            return true;
        }
        if self.depth >= DEEPEST {
            if !self.capped {
                log::warn!("markdown: HTML nested past {DEEPEST} elements reads as text");
                self.capped = true;
            }
            return false;
        }
        *self.open.entry(name.to_string()).or_default() += 1;
        self.depth += 1;
        true
    }
}

/// `text` safe as HTML text.
pub(super) fn escaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    escape(&mut out, text, |out, ch| out.extend([ESC, ch]));
    out
}

/// `text` safe as a quoted attribute value.
pub(super) fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    escape(&mut out, text, |out, ch| out.push_str(&reference(ch)));
    out
}

fn escape(out: &mut String, text: &str, reserved_as: impl Fn(&mut String, char)) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            ch if reserved(ch) => reserved_as(out, ch),
            ch => out.push(ch),
        }
    }
}

/// Markup with reserved characters as references: names never decode them.
fn referenced(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    for ch in html.chars() {
        match reserved(ch) {
            true => out.push_str(&reference(ch)),
            false => out.push(ch),
        }
    }
    out
}

fn reference(ch: char) -> String {
    format!("&#x{:X};", u32::from(ch))
}
