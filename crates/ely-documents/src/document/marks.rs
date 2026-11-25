//! A find's matches marked in every mode, and the one stepped to mid-view.

use std::{cell::RefCell, ops::Range, rc::Rc};

use gpui::{App, Context, Div, Hsla, InteractiveElement as _, Styled as _, Window, div};

use super::MarkdownDocument;
use crate::{MarkdownMode, forms::Highlight, theme::ActiveTheme};

/// The open field's washes, shared with its highlighter.
pub(super) type FoundOpen = Rc<RefCell<Vec<(Range<usize>, Highlight)>>>;

/// Matches as a find marked them, and the one stepped to.
#[derive(Clone, Debug, Default)]
pub(super) struct Found {
    pub(super) hits: Vec<Range<usize>>,
    pub(super) current: Option<usize>,
    /// The wash behind each match, then the current one's.
    washes: (Hsla, Hsla),
    /// The bar beside a match's line or block, then the current one's.
    bars: (Hsla, Hsla),
}

impl Found {
    fn tones(&self, at: usize) -> (Hsla, Hsla) {
        match Some(at) == self.current {
            true => (self.washes.1, self.bars.1),
            false => (self.washes.0, self.bars.0),
        }
    }

    /// The bar for `range`: the current match's when it holds that, else any match's.
    fn bar_over(&self, range: &Range<usize>) -> Option<Hsla> {
        let reaches = |hit: &Range<usize>| hit.start <= range.end && range.start <= hit.end;
        let first = self.hits.partition_point(|hit| hit.end < range.start);
        let inside: Vec<usize> = (first..self.hits.len())
            .take_while(|&at| self.hits[at].start <= range.end)
            .filter(|&at| reaches(&self.hits[at]))
            .collect();
        let current = self.current.filter(|at| inside.contains(at));
        match (current, inside.first()) {
            (Some(_), _) => Some(self.bars.1),
            (None, Some(_)) => Some(self.bars.0),
            (None, None) => None,
        }
    }
}

impl MarkdownDocument {
    /// Marks a find's `hits`, sorted, in every mode: a wash behind each and a bar beside its line or block, `current` stronger; none clears.
    pub fn set_found(
        &mut self,
        hits: Vec<Range<usize>>,
        current: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        let tone = cx.theme().colors.warning;
        let found = Found {
            current: current.filter(|at| *at < hits.len()),
            hits,
            washes: (tone.opacity(0.3), tone.opacity(0.6)),
            bars: (tone.opacity(0.5), tone),
        };
        let (washes, bars) = (found.hits.iter().enumerate())
            .map(|(at, hit)| {
                let (wash, bar) = found.tones(at);
                ((hit.clone(), wash), (hit.clone(), bar))
            })
            .unzip();
        self.editor.update(cx, |editor, cx| {
            editor.set_backgrounds(washes, cx);
            editor.set_gutter_bars(bars, cx);
        });
        log::debug!(
            "markdown document: {} matches marked, current {:?}",
            found.hits.len(),
            found.current
        );
        self.found = found;
        cx.notify();
    }

    /// The open field's matches, for its highlighter to read as it draws.
    pub(super) fn mark_open(&self, cx: &App) {
        let marks = self.open.as_ref().map_or_else(Vec::new, |open| {
            self.found_in(open.range.start, open.input.read(cx).text(), cx)
        });
        *self.found_open.borrow_mut() = marks;
    }

    /// Washes behind the matches in a field whose text starts at byte `start`.
    fn found_in(&self, start: usize, text: &str, cx: &App) -> Vec<(Range<usize>, Highlight)> {
        let color = cx.theme().colors.fg;
        (self.found.hits.iter().enumerate())
            .filter(|(_, hit)| start <= hit.start && hit.end <= start + text.len())
            .map(|(at, hit)| (hit.start - start..hit.end - start, at))
            .filter(|(range, _)| {
                text.is_char_boundary(range.start) && text.is_char_boundary(range.end)
            })
            .map(|(range, at)| {
                let wash = self.found.tones(at).0;
                (
                    range,
                    Highlight {
                        background: Some(wash),
                        ..Highlight::new(color)
                    },
                )
            })
            .collect()
    }

    /// The washes behind matches in `range`, from its start.
    pub(super) fn washes_in(&self, range: &Range<usize>) -> Vec<(Range<usize>, Hsla)> {
        let first = (self.found.hits).partition_point(|hit| hit.end <= range.start);
        (first..self.found.hits.len())
            .map(|at| (at, &self.found.hits[at]))
            .take_while(|(_, hit)| hit.start < range.end)
            .map(|(at, hit)| {
                let inside =
                    hit.start.max(range.start) - range.start..hit.end.min(range.end) - range.start;
                (inside, self.found.tones(at).0)
            })
            .collect()
    }

    /// A bar on the left edge of block `ix` when it holds a match.
    pub(super) fn found_bar(&self, ix: usize) -> Option<Div> {
        let range = &self.parsed.blocks.get(ix)?.range;
        let color = self.found.bar_over(range)?;
        Some(
            div()
                .debug_selector(move || format!("markdown-found-{ix}"))
                .absolute()
                .left_0()
                .top_1()
                .bottom_1()
                .w_0p5()
                .rounded_full()
                .bg(color),
        )
    }

    /// Scrolls `line` to the middle of the view once read; the caret stays.
    pub(super) fn center_line(&mut self, line: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.asks += 1;
        self.middle = Some((line, self.asks, false));
        self.center_waiting(window, cx);
    }

    /// Moves the view to the asked middle once read and laid out.
    pub(super) fn center_waiting(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((line, ask, false)) = self.middle else {
            return;
        };
        let editor = self.editor.read(cx);
        let line = line.min(editor.buffer().lines() - 1);
        let at = editor.buffer().line_range(line).start;
        if matches!(self.mode, MarkdownMode::Source | MarkdownMode::Split) {
            self.middle = None;
            return self
                .editor
                .update(cx, |editor, cx| editor.center_on(at, cx));
        }
        if !self.read_now(cx) {
            return;
        }
        let editor = self.editor.read(cx);
        let item_ix = self.block_at(at);
        let Some(block) = self.parsed.blocks.get(item_ix) else {
            self.middle = None;
            return;
        };
        let first = editor.buffer().line_of(block.range.start);
        let lines = editor.buffer().line_of(block.range.end) + 1 - first;
        // The line's own middle, as a share of its block.
        let into = (line.saturating_sub(first) as f32 + 0.5) / lines as f32;
        self.list.scroll_to_reveal_item(item_ix);
        self.middle = Some((line, ask, true));
        cx.notify();
        // A block's place is known once drawn.
        cx.on_next_frame(window, move |document, _, cx| {
            document.centered(ask, item_ix, into, cx)
        });
    }

    fn centered(&mut self, ask: u64, item_ix: usize, into: f32, cx: &mut Context<Self>) {
        if self.middle.is_none_or(|(_, asked, _)| asked != ask) {
            return;
        }
        self.middle = None;
        let Some(block) = self.list.bounds_for_item(item_ix) else {
            return log::warn!("markdown document: block {item_ix} not drawn to center");
        };
        let y = block.origin.y + block.size.height * into;
        self.list
            .scroll_by(y - self.list.viewport_bounds().center().y);
        cx.notify();
    }
}
