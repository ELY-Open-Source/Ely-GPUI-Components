//! A format over several blocks, block by block.

use std::ops::Range;

use crate::markdown::mdast::Node;

use super::{
    Change, EditRefusal, InlineFormat, block, in_marks, room, styled, toggle_in, underline,
};
use crate::markdown::span;

/// Formats that run across blocks, block by block.
pub(super) fn spreads(format: InlineFormat) -> bool {
    matches!(
        format,
        InlineFormat::Strong
            | InlineFormat::Emphasis
            | InlineFormat::Strike
            | InlineFormat::Underline
            | InlineFormat::Code
    )
}

/// `selection` trimmed of the spaces at its ends.
pub(super) fn trimmed(text: &str, selection: Range<usize>) -> Range<usize> {
    let picked = &text[selection.clone()];
    let s = selection.start + (picked.len() - picked.trim_start().len());
    let e = (selection.end - (picked.len() - picked.trim_end().len())).max(s);
    s..e
}

/// The smallest `format` node holding `s..e`, marks aside.
pub(super) fn holding<'a>(
    text: &str,
    nodes: &[&'a Node],
    Range { start: s, end: e }: Range<usize>,
    format: InlineFormat,
) -> Option<&'a Node> {
    let holds = |node: &Node| {
        let whole = span(node);
        let inside = match s == e {
            true => whole.start < s && s < whole.end,
            false => whole.start <= s && e <= whole.end,
        };
        inside && !in_marks(node, text, s) && !in_marks(node, text, e)
    };
    (nodes.iter().copied())
        .filter(|node| styled(format, node) && holds(node))
        .min_by_key(|node| span(node).len())
}

/// What a `<u>` pair holds in `part`, end tags aside.
fn underlined(
    text: &str,
    nodes: &[&Node],
    room: &Range<usize>,
    part: &Range<usize>,
) -> Option<Range<usize>> {
    let tag = |at: Range<usize>, name: &str| {
        (text.get(at)).is_some_and(|tag| tag.eq_ignore_ascii_case(name))
    };
    let mut inside = part.clone();
    if inside.len() >= 3 && tag(inside.start..inside.start + 3, "<u>") {
        inside.start += 3;
    }
    if inside.len() >= 4 && tag(inside.end - 4..inside.end, "</u>") {
        inside.end -= 4;
    }
    underline(nodes, room, inside.start, inside.end).map(|_| inside)
}

/// `format` in each block `s..e` touches, as one toggle.
pub(super) fn each(
    text: &str,
    nodes: &[&Node],
    Range { start: s, end: e }: Range<usize>,
    format: InlineFormat,
) -> Result<Change, EditRefusal> {
    // A block's own nodes follow it in the walk.
    let mut parts = Vec::new();
    for (ix, node) in nodes.iter().enumerate().filter(|(_, node)| block(node)) {
        let (whole, room) = (span(node), room(node, text));
        if !(room.start < e && s < room.end) {
            continue;
        }
        let part = trimmed(text, s.max(room.start)..e.min(room.end));
        let count = (nodes[ix + 1..].iter())
            .take_while(|kid| whole.start <= span(kid).start && span(kid).end <= whole.end)
            .count();
        let kids = &nodes[ix + 1..ix + 1 + count];
        // Where it toggles, and whether it has the format.
        let (picked, has) = match format {
            InlineFormat::Underline => match underlined(text, kids, &room, &part) {
                Some(inside) => (inside, true),
                None => (part.clone(), false),
            },
            _ => (
                part.clone(),
                holding(text, kids, part.clone(), format).is_some(),
            ),
        };
        if !part.is_empty() {
            parts.push((picked, has, room, kids));
        }
    }
    parts.sort_by_key(|(picked, ..)| picked.start);
    if parts.is_empty() {
        return Err(EditRefusal::Outside);
    }
    // All with it lose it; else the rest gain it.
    let all = parts.iter().all(|(_, has, ..)| *has);
    let mut edits = Vec::new();
    for (picked, _, room, kids) in parts.into_iter().filter(|(_, has, ..)| all || !has) {
        let rooms = std::slice::from_ref(&room);
        edits.extend(toggle_in(text, kids, rooms, picked, format)?.0);
    }
    let selection = moved(&edits, s, false)..moved(&edits, e, true);
    Ok((edits, selection))
}

/// Where byte `at` lands after sorted `edits`; inside, an end.
fn moved(edits: &[(Range<usize>, String)], at: usize, end: bool) -> usize {
    let mut grown = 0isize;
    for (range, with) in edits {
        if range.end <= at {
            grown += with.len() as isize - range.len() as isize;
        } else if range.start < at {
            let start = range.start.saturating_add_signed(grown);
            return if end { start + with.len() } else { start };
        }
    }
    at.saturating_add_signed(grown)
}
