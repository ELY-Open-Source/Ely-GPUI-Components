//! A find's matches washed in drawn text.

use std::{cell::RefCell, ops::Range};

use gpui::{HighlightStyle, Hsla};

use crate::tree::Inline;

/// Highlights laid over a drawn text.
pub(crate) type Over = Vec<(Range<usize>, HighlightStyle)>;

/// Matches in one block: its source from its start, each match's span there and wash.
#[derive(Clone, Copy)]
pub(crate) struct Washes<'a> {
    pub source: &'a str,
    pub hits: &'a [(Range<usize>, Hsla)],
    /// Which matches some drawn text washed.
    pub seen: &'a RefCell<Vec<bool>>,
}

fn wash(range: Range<usize>, color: Hsla) -> (Range<usize>, HighlightStyle) {
    let style = HighlightStyle {
        background_color: Some(color),
        ..HighlightStyle::default()
    };
    (range, style)
}

impl Washes<'_> {
    /// `over` for match `at`, now seen.
    fn saw(
        &self,
        at: usize,
        over: (Range<usize>, HighlightStyle),
    ) -> (Range<usize>, HighlightStyle) {
        if let Some(seen) = self.seen.borrow_mut().get_mut(at) {
            *seen = true;
        }
        over
    }

    /// Each match's wash over the text `inline` draws for it.
    pub(crate) fn inline(&self, inline: &Inline) -> Over {
        let spans = inline.sources.iter().map(|(_, span, _)| span);
        let (Some(low), Some(high)) = (
            spans.clone().map(|span| span.start).min(),
            spans.map(|span| span.end).max(),
        ) else {
            return Vec::new();
        };
        (self.hits.iter().enumerate())
            .filter_map(|(at, (hit, color))| {
                let (start, end) = (hit.start.max(low), hit.end.min(high));
                if start >= end {
                    return None;
                }
                let first = inline.drawn_at(self.source, start)?;
                let last = inline.drawn_at(self.source, end - 1)?;
                let after = last + inline.text[last..].chars().next()?.len_utf8();
                (first < after).then(|| self.saw(at, wash(first..after, *color)))
            })
            .collect()
    }

    /// Each match's wash over code `text` whose lines start at source byte `from`.
    pub(crate) fn code(&self, text: &str, from: usize) -> Over {
        (self.hits.iter().enumerate())
            .filter(|(_, (hit, _))| from <= hit.start)
            .filter_map(|(at, (hit, color))| {
                let drawn = hit.start - from..hit.end - from;
                // Lines under a quote or list differ from their source.
                (text.get(drawn.clone())? == self.source.get(hit.clone())?)
                    .then(|| self.saw(at, wash(drawn, *color)))
            })
            .collect()
    }

    /// Each match's wash over the block's source drawn as written.
    pub(crate) fn raw(&self) -> Over {
        (self.hits.iter().enumerate())
            .map(|(at, (hit, color))| self.saw(at, wash(hit.clone(), *color)))
            .collect()
    }

    /// The strongest wash of matches no drawn text showed: pictures, formulas, generated text.
    pub(crate) fn missed(&self) -> Option<Hsla> {
        let seen = self.seen.borrow();
        (self.hits.iter().zip(seen.iter()))
            .filter(|(_, seen)| !**seen)
            .map(|((_, color), _)| *color)
            .max_by(|one, other| one.a.total_cmp(&other.a))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::red;

    use super::Washes;
    use crate::tree::{Node, parse};

    fn first(nodes: &[Node]) -> &crate::tree::Inline {
        match &nodes[0] {
            Node::Paragraph(inline) | Node::Heading(_, inline) => inline,
            other => panic!("a paragraph, not {other:?}"),
        }
    }

    fn washed(source: &str, word: &str) -> Vec<String> {
        let at = source.find(word).expect("the word is in the source");
        let hits = [(at..at + word.len(), red())];
        let nodes = parse(source);
        let inline = first(&nodes);
        let washes = Washes {
            source,
            hits: &hits,
            seen: &RefCell::new(vec![false; hits.len()]),
        };
        (washes.inline(inline).into_iter())
            .map(|(range, _)| inline.text[range].to_string())
            .collect()
    }

    #[test]
    fn a_match_is_washed_where_it_is_drawn() {
        assert_eq!(washed("A **bold** needle.", "needle"), ["needle"]);
        assert_eq!(washed("A **bold** needle.", "bold"), ["bold"]);
        assert_eq!(washed("# A heading needle", "needle"), ["needle"]);
        assert_eq!(washed("A **bo", "bo"), ["bo"]);
        assert_eq!(washed("A [link](http://x) here", "link"), ["link"]);
        assert_eq!(washed("Ein Bär und ein Bär.", "Bär und"), ["Bär und"]);
    }

    #[test]
    fn a_match_washes_only_the_text_it_lies_in() {
        let source = "- one needle\n- two needle\n";
        let second = source.rfind("needle").expect("the second");
        let hits = [(second..second + 6, red())];
        let nodes = parse(source);
        let Node::List { items, .. } = &nodes[0] else {
            panic!("a list")
        };
        let inline = |at: usize| match &items[at].1[0] {
            Node::Paragraph(inline) => inline.clone(),
            other => panic!("an item's paragraph, not {other:?}"),
        };
        let washes = Washes {
            source,
            hits: &hits,
            seen: &RefCell::new(vec![false; hits.len()]),
        };
        assert!(
            washes.inline(&inline(0)).is_empty(),
            "not over the first item"
        );
        let over = washes.inline(&inline(1));
        assert_eq!(over.len(), 1);
        assert_eq!(&inline(1).text[over[0].0.clone()], "needle");
    }

    #[test]
    fn code_is_washed_only_where_it_reads_as_written() {
        let source = "```\nlet needle = 1;\n```\n";
        let at = source.find("needle").expect("the word");
        let hits = [(at..at + 6, red())];
        let Node::Code { text, from, .. } = &parse(source)[0] else {
            panic!("a code block")
        };
        let washes = Washes {
            source,
            hits: &hits,
            seen: &RefCell::new(vec![false; hits.len()]),
        };
        let over = washes.code(text, *from);
        assert_eq!(over.len(), 1);
        assert_eq!(&text[over[0].0.clone()], "needle");
        let moved = Washes {
            source: "x",
            hits: &hits,
            seen: &RefCell::new(vec![false; hits.len()]),
        };
        assert!(moved.code(text, *from).is_empty(), "other text, no wash");
    }

    #[test]
    fn a_match_no_text_shows_is_missed_in_its_strongest_wash() {
        let source = "A needle and a needle.";
        let (dim, strong) = (red().opacity(0.3), red().opacity(0.6));
        let hits = [(2..8, dim), (15..21, strong)];
        let seen = RefCell::new(vec![false; hits.len()]);
        let washes = Washes {
            source,
            hits: &hits,
            seen: &seen,
        };
        assert_eq!(washes.missed(), Some(strong), "nothing drawn yet");
        washes.inline(first(&parse(source)));
        assert_eq!(washes.missed(), None, "both drawn");
    }
}
