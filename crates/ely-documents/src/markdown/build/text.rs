//! Text runs: pieces merged into one node, literal links split out.

use std::ops::Range;

use super::{
    super::{
        autolink,
        mdast::{self, Node},
        shape::literal,
    },
    Builder, Kind,
};

/// Text and soft breaks in a row, read as one node.
pub(super) struct Run {
    range: Range<usize>,
    value: String,
    /// Each piece's span, its place in `value`, and whether it reads as written.
    pieces: Vec<(Range<usize>, usize, bool)>,
}

impl Builder<'_> {
    pub(super) fn push_text(&mut self, span: Range<usize>, value: &str) {
        let verbatim = self.source.get(span.clone()) == Some(value);
        let run = self.text.get_or_insert_with(|| Run {
            range: span.start..span.start,
            value: String::new(),
            pieces: Vec::new(),
        });
        run.pieces.push((span.clone(), run.value.len(), verbatim));
        run.value.push_str(value);
        run.range.end = span.end;
    }

    /// The pending text as nodes, literal links split out.
    pub(super) fn flush(&mut self) {
        let Some(run) = self.text.take() else {
            return;
        };
        let linked = (self.stack.iter()).any(|frame| matches!(frame.kind, Kind::Link { .. }));
        let found = match linked {
            true => Vec::new(),
            false => links(self.source, &run),
        };
        let (mut source_from, mut value_from, mut nodes) = (run.range.start, 0, Vec::new());
        let mut piece = 0;
        for (range, url) in found {
            let start = value_at(&run, range.start, &mut piece);
            let end = value_at(&run, range.end, &mut piece);
            if start > value_from {
                let before = &run.value[value_from..start];
                nodes.push(Node::Text(literal(before, source_from..range.start)));
            }
            nodes.push(Node::Link(mdast::Link {
                children: vec![Node::Text(literal(&run.value[start..end], range.clone()))],
                position: range.clone(),
                url,
            }));
            (source_from, value_from) = (range.end, end);
        }
        if value_from < run.value.len() || nodes.is_empty() {
            let rest = &run.value[value_from..];
            nodes.push(Node::Text(literal(rest, source_from..run.range.end)));
        }
        self.top().children.extend(nodes);
    }
}

/// Literal links in the written stretches of `run`.
fn links(source: &str, run: &Run) -> Vec<(Range<usize>, String)> {
    let (mut out, mut stretch) = (Vec::new(), None::<Range<usize>>);
    for (span, _, verbatim) in &run.pieces {
        let joins = *verbatim && stretch.as_ref().is_some_and(|open| open.end == span.start);
        if joins {
            stretch.as_mut().expect("a stretch to join").end = span.end;
            continue;
        }
        if let Some(done) = stretch.take() {
            out.extend(autolink::find(source, done));
        }
        if *verbatim {
            stretch = Some(span.clone());
        }
    }
    if let Some(done) = stretch {
        out.extend(autolink::find(source, done));
    }
    out
}

/// The byte of `run`'s value written at source byte `at`.
fn value_at(run: &Run, at: usize, piece: &mut usize) -> usize {
    // Links come in order, so the search moves forward.
    let holds = |(span, _, verbatim): &(Range<usize>, usize, bool)| {
        *verbatim && span.start <= at && at <= span.end
    };
    while !holds(&run.pieces[*piece]) {
        *piece += 1;
    }
    let (span, from, _) = &run.pieces[*piece];
    from + (at - span.start)
}
