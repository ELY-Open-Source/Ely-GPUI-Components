//! `\(…\)` and `\[…\]` read as math between dollars.

use std::ops::Range;

use super::{build, mdast::Node, span, walk};
use crate::tree::{Style, html};

/// Bytes free for a bare dollar and a delimiter's pad while parsing.
const STAND_INS: std::ops::RangeInclusive<u8> = 0x01..=0x08;

/// One pair's delimiters as written.
struct Pair {
    open: Range<usize>,
    close: Range<usize>,
}

/// `text` as a syntax tree, each pair read as inline math.
pub(super) fn parse(text: &str, frontmatter: bool) -> Node {
    let plain = build::tree(text, frontmatter);
    if !text.contains("\\(") && !text.contains("\\[") {
        return plain;
    }
    let mut free = STAND_INS.filter(|byte| !text.as_bytes().contains(byte));
    let (Some(stand_in), Some(pad)) = (free.next(), free.next()) else {
        log::warn!("markdown: \\( reads as text; the stand-in bytes are in use");
        return plain;
    };
    let mut nodes = Vec::new();
    walk(&plain, &mut nodes);
    let mut kept = vec![false; text.len()];
    let (mut pairs, mut dollars) = (Vec::new(), Vec::new());
    for node in nodes {
        if matches!(
            node,
            Node::Paragraph(_) | Node::Heading(_) | Node::TableCell(_)
        ) {
            keep(node, text, &mut kept);
            paired(text, span(node), &kept, &mut pairs, &mut dollars);
        }
    }
    loop {
        if pairs.is_empty() {
            return plain;
        }
        let swapped = written(text, &pairs, &dollars, stand_in, pad);
        let swapped = String::from_utf8(swapped).expect("ASCII swaps keep UTF-8");
        let mut root = build::tree(&swapped, frontmatter);
        let mut maths = Vec::new();
        walk(&root, &mut maths);
        let opens: Vec<usize> = (maths.into_iter())
            .filter(|node| matches!(node, Node::InlineMath(_)))
            .map(|node| span(node).start)
            .collect();
        let count = pairs.len();
        // Unmatched braces keep a pair from reading as math.
        pairs.retain(|pair| opens.binary_search(&pair.open.start).is_ok());
        if pairs.len() == count {
            mend(&mut root, &pairs, stand_in, pad);
            return root;
        }
        log::debug!(
            "markdown: {} of {count} \\( pairs read as math",
            pairs.len()
        );
    }
}

/// Marks what in `container` is not prose: code, tags, addresses.
fn keep(container: &Node, text: &str, kept: &mut [bool]) {
    let mut nodes = Vec::new();
    walk(container, &mut nodes);
    // Open styled tags, and where code text last started.
    let (mut open, mut from) = (Vec::new(), 0);
    for node in &nodes[1..] {
        use Node::*;
        let whole = span(node);
        match node {
            InlineCode(_) | InlineMath(_) | Image(_) | ImageReference(_) | FootnoteReference(_) => {
                kept[whole].fill(true)
            }
            Link(_) | LinkReference(_) => {
                kept[whole.clone()].fill(true);
                // Bracketed text is prose; its address is not.
                let last = node.children().and_then(|children| children.last());
                if text[whole.clone()].starts_with('[')
                    && let Some(last) = last
                {
                    // A text's span skips a leading escape: start at `[`.
                    kept[whole.start + 1..span(last).end].fill(false);
                }
            }
            Html(_) => {
                kept[whole.clone()].fill(true);
                let Some(tag) = html::tag(&text[whole.clone()]) else {
                    continue;
                };
                if html::inline_style(&tag.name).is_none() {
                    continue;
                }
                let was = coded(&open);
                match tag.closing {
                    true => end(&mut open, &tag.name),
                    false => {
                        if tag.name == "a" {
                            end(&mut open, "a");
                        }
                        open.push(tag.name);
                    }
                }
                match (was, coded(&open)) {
                    (false, true) => from = whole.end,
                    (true, false) => kept[from..whole.start].fill(true),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    // An unclosed `<code>` runs to its block's end.
    if coded(&open) {
        kept[from..span(container).end].fill(true);
    }
}

/// Whether an open tag shows its text as code.
fn coded(open: &[String]) -> bool {
    (open.iter()).any(|name| html::inline_style(name) == Some(Style::Code))
}

/// Ends the innermost `name` as the reader does: inner tags close.
fn end(open: &mut Vec<String>, name: &str) {
    let Some(at) = open.iter().rposition(|opened| opened == name) else {
        return;
    };
    let closed = open.split_off(at);
    open.extend(
        closed
            .into_iter()
            .skip(1)
            .filter(|name| html::reopens(name)),
    );
}

/// Pairs and bare dollars in `within`; pairs cross nothing kept.
fn paired(
    text: &str,
    within: Range<usize>,
    kept: &[bool],
    pairs: &mut Vec<Pair>,
    dollars: &mut Vec<usize>,
) {
    let bytes = text.as_bytes();
    let mut open: Option<(usize, u8)> = None;
    let mut at = within.start;
    while at < within.end {
        if kept[at] {
            open = None;
        } else if bytes[at] == b'$' {
            dollars.push(at);
        } else if bytes[at] == b'\\' && at + 1 < within.end {
            match (bytes[at + 1], open) {
                (b'(', None) => open = Some((at, b')')),
                (b'[', None) => open = Some((at, b']')),
                (close, Some((start, wanted))) if close == wanted && !kept[at + 1] => {
                    if !bytes[start + 2..at].iter().all(u8::is_ascii_whitespace) {
                        pairs.push(Pair {
                            open: start..start + 2,
                            close: at..at + 2,
                        });
                    }
                    open = None;
                }
                _ => {}
            }
            // An escaped byte escapes nothing after it.
            at += 2;
            continue;
        }
        at += 1;
    }
}

/// The swapped bytes: each delimiter a dollar and its pad, bare dollars stood in.
fn written(text: &str, pairs: &[Pair], dollars: &[usize], stand_in: u8, pad: u8) -> Vec<u8> {
    let mut out = text.as_bytes().to_vec();
    for at in dollars {
        out[*at] = stand_in;
    }
    for pair in pairs {
        out[pair.open.clone()].copy_from_slice(&[b'$', pad]);
        out[pair.close.clone()].copy_from_slice(&[pad, b'$']);
    }
    out
}

/// Puts each formula's TeX and every bare dollar back.
fn mend(node: &mut Node, pairs: &[Pair], stand_in: u8, pad: u8) {
    let restore = |value: &str| value.replace(char::from(stand_in), "$");
    match node {
        Node::InlineMath(math)
            if pairs
                .iter()
                .any(|pair| pair.open.start == math.position.start) =>
        {
            let pad = char::from(pad);
            let tex = (math.value.strip_prefix(pad))
                .and_then(|value| value.strip_suffix(pad))
                .expect("a swapped pair keeps its pads");
            math.value = restore(tex);
        }
        Node::Text(text) => text.value = restore(&text.value),
        _ => {}
    }
    for child in node.children_mut().into_iter().flatten() {
        mend(child, pairs, stand_in, pad);
    }
}

#[cfg(test)]
mod tests;
