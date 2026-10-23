//! `$$` fences as math blocks: written as tilde fences pulldown-cmark places.

use std::ops::Range;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

/// A text with its `$$` fences as tildes, and the way back.
pub(super) struct Flow {
    pub text: String,
    /// Each fence's tildes in the new text, and its old start.
    fences: Vec<(Range<usize>, usize)>,
    /// Tildes added to each fence.
    extra: usize,
    /// Whether each fence opens math.
    opens: Vec<bool>,
}

impl Flow {
    /// The source byte for byte `at` of the new text.
    pub(crate) fn source(&self, at: usize) -> usize {
        let ix = self.fences.partition_point(|(run, _)| run.start <= at);
        let Some((run, old)) = ix.checked_sub(1).map(|ix| &self.fences[ix]) else {
            return at;
        };
        // Added tildes lead the run and stand for its start.
        old + (at - run.start).saturating_sub(self.extra)
    }

    /// `range` of the new text in the source.
    pub(crate) fn span(&self, range: &Range<usize>) -> Range<usize> {
        self.source(range.start)..self.source(range.end)
    }

    /// Whether a block whose first line is `line`, in the new text, opens math.
    pub(crate) fn opens_math(&self, line: &Range<usize>) -> bool {
        self.on(line).is_some_and(|ix| self.opens[ix])
    }

    /// The fence that starts in `line`, by index.
    fn on(&self, line: &Range<usize>) -> Option<usize> {
        let ix = self
            .fences
            .partition_point(|(run, _)| run.start < line.start);
        let fence = self
            .fences
            .get(ix)
            .filter(|(run, _)| line.contains(&run.start));
        fence.map(|_| ix)
    }
}

/// `$$` runs that may fence: after container marks, alone with a meta.
fn fences(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < text.len() {
        let end = text[at..]
            .find(['\n', '\r'])
            .map_or(text.len(), |ix| at + ix);
        if let Some(run) = fence(&text[at..end]) {
            out.push(at + run.start..at + run.end);
        }
        at = end + 1 + usize::from(text[end..].starts_with("\r\n"));
    }
    out
}

/// The `$` run of `line` if it can fence math.
fn fence(line: &str) -> Option<Range<usize>> {
    let bytes = line.as_bytes();
    let mut at = 0;
    loop {
        while bytes
            .get(at)
            .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
        {
            at += 1;
        }
        let marked = match bytes.get(at) {
            Some(b'>') => 1,
            Some(b'-' | b'+' | b'*') => 1,
            Some(b'0'..=b'9') => {
                let digits = bytes[at..]
                    .iter()
                    .take_while(|b| b.is_ascii_digit())
                    .count();
                match bytes.get(at + digits) {
                    Some(b'.' | b')') if digits <= 9 => digits + 1,
                    _ => break,
                }
            }
            _ => break,
        };
        let spaced = bytes[at] == b'>'
            || (bytes.get(at + marked))
                .is_none_or(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'));
        if !spaced {
            break;
        }
        at += marked;
    }
    let run = bytes[at..].iter().take_while(|byte| **byte == b'$').count();
    let rest = &line[at + run..];
    (run >= 2 && !rest.contains('$')).then_some(at..at + run)
}

/// `text` with every fence in `runs` written as tildes.
fn written(text: &str, runs: &[Range<usize>], extra: usize) -> Flow {
    let mut out = String::with_capacity(text.len() + runs.len() * extra);
    let (mut fences, mut from) = (Vec::new(), 0);
    for run in runs {
        out.push_str(&text[from..run.start]);
        let start = out.len();
        out.push_str(&"~".repeat(run.len() + extra));
        fences.push((start..out.len(), run.start));
        from = run.end;
    }
    out.push_str(&text[from..]);
    Flow {
        text: out,
        fences,
        extra,
        opens: Vec::new(),
    }
}

/// `text` with its math fences as tildes; each kept fence opens or closes math.
pub(super) fn rewrite(text: &str, options: Options) -> Flow {
    let mut active = outside_code(text, fences(text), options);
    loop {
        // One tilde more keeps a longer `$` run the longer fence.
        let mut flow = written(text, &active, 1);
        if active.is_empty() {
            return flow;
        }
        let kept = kept(&flow, options);
        if kept.len() == active.len() {
            flow.opens = vec![false; active.len()];
            for (ix, opens) in kept {
                flow.opens[ix] = opens;
            }
            return flow;
        }
        log::debug!(
            "markdown: {} of {} `$$` lines fence math",
            kept.len(),
            active.len()
        );
        active = kept.iter().map(|(ix, _)| active[*ix].clone()).collect();
    }
}

/// `runs` less those in code or HTML as the text reads without math.
fn outside_code(text: &str, runs: Vec<Range<usize>>, options: Options) -> Vec<Range<usize>> {
    if runs.is_empty() {
        return runs;
    }
    let held = Parser::new_ext(text, options)
        .into_offset_iter()
        .filter_map(|(event, range)| match event {
            Event::Start(Tag::CodeBlock(_) | Tag::HtmlBlock) => Some(range),
            _ => None,
        });
    let held: Vec<Range<usize>> = held.collect();
    let mut ix = 0;
    (runs.into_iter())
        .filter(|run| {
            while held.get(ix).is_some_and(|block| block.end <= run.start) {
                ix += 1;
            }
            held.get(ix).is_none_or(|block| run.start < block.start)
        })
        .collect()
}

/// Fences pulldown-cmark took as a block's opening or closing line, by index; whether each opens.
fn kept(flow: &Flow, options: Options) -> Vec<(usize, bool)> {
    let text = &flow.text;
    let ends = |byte: &u8| matches!(byte, b'\n' | b'\r');
    let line_of = |at: usize| {
        let bytes = text.as_bytes();
        let start = bytes[..at].iter().rposition(ends).map_or(0, |ix| ix + 1);
        let end = bytes[at..]
            .iter()
            .position(ends)
            .map_or(text.len(), |ix| at + ix);
        start..end
    };
    let mut out = Vec::new();
    for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
        let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) = event else {
            continue;
        };
        let Some(open) = flow.on(&line_of(range.start)) else {
            continue;
        };
        out.push((open, true));
        let last = line_of(range.end.saturating_sub(1).max(range.start));
        let Some(close) = flow.on(&last).filter(|close| *close != open) else {
            continue;
        };
        let (closing, opening) = (&flow.fences[close].0, &flow.fences[open].0);
        let bare = text[closing.end..last.end].trim().is_empty();
        if bare && closing.len() >= opening.len() {
            out.push((close, false));
        }
    }
    out.sort_unstable();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fence_follows_container_marks_and_takes_no_dollar_after() {
        assert_eq!(fence("$$\n"), Some(0..2));
        assert_eq!(fence("> - $$$ x\n"), Some(4..7));
        assert_eq!(fence("1. $$"), Some(3..5));
        assert_eq!(fence("$$x$$"), None);
        assert_eq!(fence("a $$"), None);
        assert_eq!(fence("-$$"), None);
    }

    #[test]
    fn new_bytes_map_back_to_the_source() {
        let flow = rewrite("a\n\n$$\nx\n$$\nb", Options::empty());
        assert_eq!(flow.text, "a\n\n~~~\nx\n~~~\nb");
        let map: Vec<usize> = (0..flow.text.len()).map(|at| flow.source(at)).collect();
        assert_eq!(map, [0, 1, 2, 3, 3, 4, 5, 6, 7, 8, 8, 9, 10, 11]);
        assert!(flow.opens_math(&(3..6)));
        assert!(!flow.opens_math(&(9..12)));
        let carried = rewrite("$$\rx\r$$", Options::empty());
        assert_eq!(carried.text, "~~~\rx\r~~~");
    }

    #[test]
    fn dollars_inside_code_stay_dollars() {
        let flow = rewrite("```\n$$\n```\n", Options::empty());
        assert_eq!(flow.text, "```\n$$\n```\n");
    }

    #[test]
    fn a_shorter_or_meta_line_inside_math_stays_content() {
        let flow = rewrite("$$$\n$$\n$$ x\n", Options::empty());
        assert_eq!(flow.text, "~~~~\n$$\n$$ x\n");
    }
}
