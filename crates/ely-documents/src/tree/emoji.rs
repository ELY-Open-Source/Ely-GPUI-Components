use std::borrow::Cow;

use super::{dom::Stream, wrap, written::Written};
use crate::markdown::mdast;

/// `text` with each known `:shortcode:` as its emoji.
fn emoji(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(':') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let len = after
            .find(|ch: char| !shortcode_char(ch))
            .unwrap_or(after.len());
        let found = after[len..]
            .starts_with(':')
            .then(|| emojis::get_by_shortcode(&after[..len]))
            .flatten();
        match found {
            Some(found) => {
                out.push_str(found.as_str());
                rest = &after[len + 1..];
            }
            None => {
                out.push(':');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn shortcode_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '+' | '-')
}

/// Where the first `:name:` starts.
fn candidate(text: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(at) = text[from..].find(':').map(|at| from + at) {
        let after = &text[at + 1..];
        let len = after
            .find(|ch: char| !shortcode_char(ch))
            .unwrap_or(after.len());
        if len > 0 && after[len..].starts_with(':') {
            return Some(at);
        }
        from = at + 1 + len;
    }
    None
}

/// One DOM text node's shortcodes, as GitHub reads them.
#[derive(Default)]
pub struct Shortcodes {
    /// Settled by the node's first `:name:`.
    reads: Option<bool>,
    /// Text before it held more than white space.
    inked: bool,
    last: Option<char>,
}

impl Shortcodes {
    /// A new DOM text node starts.
    pub fn split(&mut self) {
        *self = Self::default();
    }

    /// Blank or spaced text before the first reads them all.
    pub fn read(&mut self, text: &str) -> String {
        if self.reads.is_none() {
            let at = candidate(text);
            let lead = &text[..at.unwrap_or(text.len())];
            self.inked |= !lead.chars().all(|ch| ch.is_ascii_whitespace());
            self.last = lead.chars().next_back().or(self.last);
            self.reads = at.map(|_| !self.inked || self.last == Some(' '));
        }
        match self.reads {
            Some(true) => emoji(text),
            _ => text.to_string(),
        }
    }
}

/// Shortcodes for HTML an owner writes; `code`, `pre` stay literal.
pub struct ShortcodeRun {
    /// The markup the composer reads the block inside.
    open: Cow<'static, str>,
    shortcodes: Shortcodes,
    /// Markup not yet fed, as the drawing writes it.
    written: Written,
    /// The tree so far, once a tag came.
    tree: Option<Stream>,
}

/// A paragraph's run.
impl Default for ShortcodeRun {
    fn default() -> Self {
        Self::new(&mdast::Node::Paragraph(mdast::Parent {
            children: Vec::new(),
            position: 0..0,
        }))
    }
}

impl ShortcodeRun {
    /// A run for `block`, read as the composer reads it.
    pub fn new(block: &mdast::Node) -> Self {
        Self {
            open: wrap(block).0,
            shortcodes: Shortcodes::default(),
            written: Written::default(),
            tree: None,
        }
    }

    /// A new DOM text node starts.
    pub fn split(&mut self) {
        self.shortcodes.split();
    }

    /// Text as shown; inside `code` or `pre` it stays.
    pub fn text(&mut self, written: &str) -> String {
        let raw = match &mut self.tree {
            None => false,
            Some(tree) => {
                self.written.text(written);
                // A comment places text a table holds back.
                self.written.html.push_str("<!---->");
                tree.feed(std::mem::take(&mut self.written.html));
                tree.text_within(&["code", "pre"])
            }
        };
        match raw {
            true => written.to_string(),
            false => self.shortcodes.read(written),
        }
    }

    /// Inline HTML as written; a filtered tag shows as text.
    pub fn html(&mut self, written: &str) -> String {
        self.shortcodes.split();
        let tag = super::html::tag(written);
        if let Some(tag) = &tag
            && super::html::filtered(&tag.name)
        {
            return self.text(written);
        }
        if tag.is_some() && self.tree.is_none() {
            self.tree = Some(Stream::within(&self.open));
        }
        if let Some(tree) = &mut self.tree {
            self.written.raw(written);
            tree.feed(std::mem::take(&mut self.written.html));
        }
        written.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_shortcodes_become_emoji() {
        assert_eq!(emoji("hi :smile: :+1:"), "hi 😄 👍");
        assert_eq!(emoji(":nope: at 10:30 and ::"), ":nope: at 10:30 and ::");
        assert_eq!(emoji("a:smile:b :x:smile:"), "a😄b ❌smile:");
        assert_eq!(emoji("é :smile: 中"), "é 😄 中");
    }

    #[test]
    fn long_runs_scan_in_linear_time() {
        let colons = ":zq".repeat(1 << 16);
        let started = web_time::Instant::now();
        assert_eq!(emoji(&colons).len(), colons.len());
        assert_eq!(Shortcodes::default().read(&colons).len(), colons.len());
        let spent = started.elapsed();
        assert!(spent.as_secs() < 2, "192 KiB took {spent:?}");
    }

    #[test]
    fn a_close_ends_only_its_own_kind() {
        let mut run = ShortcodeRun::default();
        let mut read = |parts: &[&str]| -> String {
            parts
                .iter()
                .map(|part| match part.starts_with('<') {
                    true => run.html(part),
                    false => run.text(part),
                })
                .collect()
        };
        let crossed = read(&[
            "a ", "<pre>", "x", "</code>", " :smile:", "</pre>", " :smile:",
        ]);
        assert_eq!(crossed, "a <pre>x</code> :smile:</pre> 😄");
        let unmatched = read(&["<code>", "</pre>", " :smile:", "</code>", " :smile:"]);
        assert_eq!(unmatched, "<code></pre> :smile:</code> 😄");
    }

    #[test]
    fn code_and_pre_end_where_the_html5_tree_ends_them() {
        let read = |parts: &[&str]| -> String {
            let mut run = ShortcodeRun::default();
            parts
                .iter()
                .map(|part| match part.starts_with('<') {
                    true => run.html(part),
                    false => run.text(part),
                })
                .collect()
        };
        // The `div`'s end closes the `pre` inside it.
        let closed = read(&["a ", "<div>", "<pre>", "x", "</div>", " :smile:"]);
        assert_eq!(closed, "a <div><pre>x</div> 😄");
        // A formatting `code` reopens past the block's end.
        let reopened = read(&["<div>", "<code>", "x", "</div>", " :smile:"]);
        assert_eq!(reopened, "<div><code>x</div> :smile:");
        // Text in a row moves out before the table.
        let row = ["<table>", "<tr>", "<td>", "<pre>", "x", "</td>", " :smile:"];
        assert_eq!(read(&row), "<table><tr><td><pre>x</td> 😄");
    }

    #[test]
    fn a_cell_s_end_closes_pre_as_the_composer_reads_it() {
        let cell = mdast::Node::TableCell(mdast::Parent {
            children: Vec::new(),
            position: 0..0,
        });
        let mut run = ShortcodeRun::new(&cell);
        let read = [run.html("<pre>"), run.text("x"), run.html("</td>")];
        assert_eq!(read.concat(), "<pre>x</td>");
        assert_eq!(run.text(" :smile:"), " 😄");
        // In a paragraph `</td>` ends nothing.
        let mut run = ShortcodeRun::default();
        let read = [run.html("<pre>"), run.text("x"), run.html("</td>")];
        assert_eq!(read.concat(), "<pre>x</td>");
        assert_eq!(run.text(" :smile:"), " :smile:");
    }
}
