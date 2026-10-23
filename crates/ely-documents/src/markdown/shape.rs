//! Spans and shapes as Ely reads them, past what pulldown-cmark gives.

use std::ops::Range;

use pulldown_cmark::{Alignment, LinkType, Parser};

use super::{
    build,
    mdast::{self, AlignKind, Literal, Node, Parent},
};

/// A row's cells, each from its pipe to the next; the last to `end`.
pub(super) fn cells(source: &str, cells: Vec<Node>, end: usize) -> Vec<Node> {
    // pulldown-cmark pads a short row past its end.
    let mut cells: Vec<Node> = (cells.into_iter())
        .filter(|cell| cell.position().start < end)
        .collect();
    let past = cells.last().map_or(end, |cell| cell.position().end);
    cells.extend(beyond(source, past..end));
    for cell in &mut cells {
        let position = cell.position_mut();
        if position.start > 0 && source.as_bytes()[position.start - 1] == b'|' {
            position.start -= 1;
        }
    }
    let starts: Vec<usize> = cells.iter().map(|cell| cell.position().start).collect();
    for (ix, cell) in cells.iter_mut().enumerate() {
        cell.position_mut().end = starts.get(ix + 1).copied().unwrap_or(end);
    }
    cells
}

/// Cells past the head's columns, which pulldown-cmark drops: their text as written, for edits.
fn beyond(source: &str, rest: Range<usize>) -> Vec<Node> {
    let bytes = source.as_bytes();
    if bytes.get(rest.start) != Some(&b'|') {
        return Vec::new();
    }
    // As pulldown-cmark splits a row, any `\|` is no pipe.
    let pipes =
        (rest.start + 1..rest.end).filter(|at| bytes[*at] == b'|' && bytes[at - 1] != b'\\');
    let (mut out, mut from) = (Vec::new(), rest.start + 1);
    for pipe in pipes.chain(std::iter::once(rest.end)) {
        let content = trim(source, from..pipe);
        // Text after the last pipe is a cell too.
        if pipe < rest.end || !content.is_empty() {
            let written = (!content.is_empty())
                .then(|| Node::Text(literal(&source[content.clone()], content)));
            out.push(Node::TableCell(Parent {
                children: written.into_iter().collect(),
                position: from..pipe,
            }));
        }
        from = pipe + 1;
    }
    out
}

/// Whether the byte at `at` follows an odd run of backslashes.
fn escaped(source: &str, at: usize) -> bool {
    let before = &source[..at];
    (before.len() - before.trim_end_matches('\\').len()) % 2 == 1
}

fn trim(source: &str, range: Range<usize>) -> Range<usize> {
    let text = &source[range.clone()];
    let start = range.start + (text.len() - text.trim_start_matches([' ', '\t']).len());
    let end = range.end - (text.len() - text.trim_end_matches([' ', '\t']).len());
    start..end.max(start)
}

/// Ends each list and item at its last child, once definitions sit in them.
pub(super) fn tighten(node: &mut Node, source: &str) {
    node.children_mut()
        .into_iter()
        .flatten()
        .for_each(|child| tighten(child, source));
    let last = node.children().and_then(|children| children.last());
    let end = last.map(|last| last.position().end);
    if let (Node::List(_) | Node::ListItem(_), Some(end)) = (&node, end) {
        let position = node.position_mut();
        *position = trimmed(source, position.start..end);
    }
}

pub(super) fn shift(node: &mut Node, by: usize) {
    let position = node.position_mut();
    *position = position.start + by..position.end + by;
    for child in node.children_mut().into_iter().flatten() {
        shift(child, by);
    }
}

/// GFM's task box at an item's first text, which pulldown-cmark reads only beside the marker.
pub(super) fn boxed(source: &str, mut children: Vec<Node>) -> (Option<bool>, Vec<Node>) {
    let Some(Node::Paragraph(paragraph)) = children.first_mut() else {
        return (None, children);
    };
    let Some(Node::Text(text)) = paragraph.children.first_mut() else {
        return (None, children);
    };
    let written = &source[text.position.clone()];
    let checked = match written.get(..3) {
        Some("[ ]") => false,
        Some("[x]" | "[X]") => true,
        _ => return (None, children),
    };
    let space = written[3..].len() - written[3..].trim_start_matches([' ', '\t']).len();
    if space == 0 || space == written.len() - 3 || !text.value.starts_with(&written[..3 + space]) {
        return (None, children);
    }
    let cut = 3 + space;
    text.value.drain(..cut);
    text.position.start += cut;
    paragraph.position.start = text.position.start;
    (Some(checked), children)
}

/// A code span's text as CommonMark reads it; pulldown-cmark reads a CRLF as two spaces.
pub(super) fn code_text(source: &str, span: Range<usize>, cell: bool) -> String {
    let inner = &source[crate::tree::inside(source, span, '`')];
    let text = inner.replace("\r\n", " ").replace(['\r', '\n'], " ");
    match cell {
        true => text.replace("\\|", "|"),
        false => text,
    }
}

/// `text` as written at `span` when only its line endings differ.
pub(super) fn written<'a>(source: &'a str, span: &Range<usize>, text: &'a str) -> &'a str {
    let raw = &source[span.clone()];
    let same = raw.contains('\r') && raw.replace("\r\n", "\n").replace('\r', "\n") == text;
    if same { raw } else { text }
}

/// A reference link's identifier, from its label as written.
pub(super) fn reference(source: &str, range: &Range<usize>, link: LinkType) -> Option<String> {
    let written = &source[range.clone()];
    let written = written.strip_prefix('!').unwrap_or(written);
    let label = match link {
        LinkType::Reference | LinkType::ReferenceUnknown => {
            let open = (1..written.len() - 1)
                .rev()
                .find(|ix| written.as_bytes()[*ix] == b'[' && !escaped(written, *ix))
                .expect("a full reference ends in its label");
            &written[open + 1..written.len() - 1]
        }
        LinkType::Collapsed | LinkType::CollapsedUnknown => &written[1..written.len() - 3],
        LinkType::Shortcut | LinkType::ShortcutUnknown => &written[1..written.len() - 1],
        _ => return None,
    };
    Some(mdast::identifier(label))
}

/// `[text][]` with its `[]`, which pulldown-cmark leaves out.
pub(super) fn collapsed(source: &str, range: Range<usize>, link: LinkType) -> Range<usize> {
    let collapsed = matches!(link, LinkType::Collapsed | LinkType::CollapsedUnknown);
    match collapsed && source[range.end..].starts_with("[]") {
        true => range.start..range.end + 2,
        false => range,
    }
}

/// Puts each definition in the deepest container around it, one pass per container.
pub(super) fn place(parent: &mut Node, mut definitions: Vec<Node>, source: &str) {
    if definitions.is_empty() {
        return;
    }
    definitions.sort_by_key(|definition| definition.position().start);
    if matches!(parent, Node::Root(_)) {
        for definition in &mut definitions {
            let position = definition.position_mut();
            position.start = indented(source, position.start);
        }
    }
    let children = parent
        .children_mut()
        .expect("a definition sits in a container");
    let (mut here, mut inner): (Vec<Node>, Vec<(usize, Vec<Node>)>) = (Vec::new(), Vec::new());
    for definition in definitions {
        let at = definition.position().clone();
        let ix = children.partition_point(|child| child.position().start <= at.start);
        let holder = ix.checked_sub(1).filter(|ix| {
            let child = &children[*ix];
            let holds = matches!(
                child,
                Node::Blockquote(_)
                    | Node::List(_)
                    | Node::ListItem(_)
                    | Node::FootnoteDefinition(_)
            );
            holds && at.end <= child.position().end
        });
        match (holder, inner.last_mut()) {
            (Some(ix), Some((last, group))) if *last == ix => group.push(definition),
            (Some(ix), _) => inner.push((ix, vec![definition])),
            (None, _) => here.push(definition),
        }
    }
    for (ix, group) in inner {
        place(&mut children[ix], group, source);
    }
    merge(children, here);
}

/// `children` with `sorted` laid in by start.
fn merge(children: &mut Vec<Node>, sorted: Vec<Node>) {
    if sorted.is_empty() {
        return;
    }
    let old = std::mem::take(children);
    let (mut old, mut new) = (old.into_iter().peekable(), sorted.into_iter().peekable());
    while let (Some(a), Some(b)) = (old.peek(), new.peek()) {
        let next = match a.position().start <= b.position().start {
            true => old.next(),
            false => new.next(),
        };
        children.extend(next);
    }
    children.extend(old.chain(new));
}

/// Top-level definitions pulldown-cmark drops as repeats, read from the gaps between blocks.
pub(super) fn repeated(root: &mut Node, source: &str) {
    let children = root.children_mut().expect("the root holds blocks");
    let (mut gaps, mut from) = (Vec::new(), 0);
    let ends = std::iter::once(source.len()..source.len());
    for block in children
        .iter()
        .map(|child| child.position().clone())
        .chain(ends)
    {
        if !source[from..block.start.max(from)].trim().is_empty() {
            gaps.push(from..block.start);
        }
        from = from.max(block.end);
    }
    let found = gaps
        .into_iter()
        .flat_map(|gap| definitions(source, gap))
        .collect();
    merge(children, found);
}

/// The definitions in `gap`, one per line that opens with `[`.
fn definitions(source: &str, gap: Range<usize>) -> Vec<Node> {
    let mut starts = Vec::new();
    let mut at = gap.start;
    while at < gap.end {
        let line = &source[at..gap.end];
        let indent = line.len() - line.trim_start_matches(' ').len();
        if indent <= 3 && line[indent..].starts_with('[') {
            starts.push(at);
        }
        at = line.find(['\n', '\r']).map_or(gap.end, |ix| at + ix + 1);
    }
    if starts.first() != Some(&gap.start)
        && !source[gap.start..*starts.first().unwrap_or(&gap.end)]
            .trim()
            .is_empty()
    {
        log::error!("markdown: {gap:?} reads as no block");
    }
    let mut out = Vec::new();
    for (ix, start) in starts.iter().enumerate() {
        let chunk = *start..starts.get(ix + 1).copied().unwrap_or(gap.end);
        let parser = Parser::new_ext(&source[chunk.clone()], build::options());
        let read = parser
            .reference_definitions()
            .iter()
            .map(|(label, definition)| {
                let span = chunk.start + definition.span.start..chunk.start + definition.span.end;
                Node::Definition(mdast::Definition {
                    position: trimmed(source, indented(source, span.start)..span.end),
                    url: definition.dest.to_string(),
                    identifier: mdast::identifier(label),
                })
            });
        let before = out.len();
        out.extend(read);
        if out.len() == before {
            log::error!("markdown: {chunk:?} reads as no block");
        }
    }
    out
}

/// `range` to the end of its last written line, spaces kept.
pub(super) fn trimmed(source: &str, range: Range<usize>) -> Range<usize> {
    let kept = source[range.clone()]
        .trim_end_matches([' ', '\t', '\n', '\r'])
        .len();
    let end = range.start + kept;
    let spaces = source[end..].len() - source[end..].trim_start_matches([' ', '\t']).len();
    range.start..end + spaces
}

/// `start` back over its indent, when only indent leads its line.
pub(super) fn indented(source: &str, start: usize) -> usize {
    let lead = source[..start].trim_end_matches([' ', '\t']);
    match lead.is_empty() || lead.ends_with(['\n', '\r']) {
        true => lead.len(),
        false => start,
    }
}

pub(super) fn align(align: &Alignment) -> AlignKind {
    match align {
        Alignment::None => AlignKind::None,
        Alignment::Left => AlignKind::Left,
        Alignment::Center => AlignKind::Center,
        Alignment::Right => AlignKind::Right,
    }
}

pub(super) fn address(link: LinkType, url: &str) -> String {
    match link {
        LinkType::Email => format!("mailto:{url}"),
        _ => url.to_string(),
    }
}

pub(super) fn literal(value: &str, position: Range<usize>) -> Literal {
    Literal {
        value: value.to_string(),
        position,
    }
}

/// `value` less one closing line ending.
pub(super) fn unended(value: &str) -> &str {
    let value = value.strip_suffix('\n').unwrap_or(value);
    value.strip_suffix('\r').unwrap_or(value)
}
