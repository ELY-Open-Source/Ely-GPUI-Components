//! Matches shown in any mode; replacements in one step.

use std::ops::Range;

use gpui::{Context, Window};

use super::{
    MarkdownDocument,
    cells::{self, Aim},
    edit::Landing,
    parse::{Block, parsed},
};
use crate::{MarkdownMode, render::CellAt};

impl MarkdownDocument {
    /// `range` in view: selected, or in its opened block.
    pub fn show_match(&mut self, range: Range<usize>, window: &mut Window, cx: &mut Context<Self>) {
        self.shown(range, true, window, cx);
    }

    /// As `show_match`, focus left where it is, as a find box needs.
    pub fn point_match(
        &mut self,
        range: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.shown(range, false, window, cx);
    }

    fn shown(
        &mut self,
        range: Range<usize>,
        focus: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let line = |document: &Self, cx: &Context<Self>| {
            document.editor.read(cx).text()[..range.start]
                .matches('\n')
                .count()
        };
        // A newer request takes the place of one waiting.
        self.showing = None;
        match self.mode {
            MarkdownMode::Source | MarkdownMode::Split => {
                self.editor
                    .update(cx, |editor, cx| editor.select(std::iter::once(range), cx));
            }
            MarkdownMode::Preview => self.reveal(line(self, cx), window, cx),
            MarkdownMode::Visual => {
                if self.commit(cx).is_some() || !self.read_now(cx) {
                    self.reparse(true, cx);
                }
                if !self.read_now(cx) {
                    // Shown once read, if nothing else changed it.
                    let expected = self.edits + self.writes as u64;
                    self.showing = Some((range, focus, expected));
                    return cx.notify();
                }
                let ix = self.block_at(range.start);
                let held = self.parsed.blocks.get(ix).map(|block| block.range.clone());
                let Some(block) =
                    held.filter(|block| block.start <= range.start && range.end <= block.end)
                else {
                    // Outside one block: its line.
                    return self.reveal(line(self, cx), window, cx);
                };
                // A match in a table opens its cell.
                let text = self.editor.read(cx).text();
                let landing = match cells::find(text, block.clone(), Aim::Byte(range.start)) {
                    Some(cell)
                        if cell.range.start <= range.start && range.end <= cell.range.end =>
                    {
                        Landing::Cell(Aim::At(cell.at))
                    }
                    Some(_) => return self.reveal(line(self, cx), window, cx),
                    None => Landing::End,
                };
                self.open(Some(ix), landing, focus, window, cx);
                self.list.scroll_to_reveal_item(ix);
                if let Some(open) = &self.open {
                    let input = open.input.clone();
                    let inside = range.start - open.range.start..range.end - open.range.start;
                    input.update(cx, |input, cx| input.select(inside, cx));
                }
            }
        }
    }

    /// A match asked before its text was read, now shown.
    pub(super) fn show_waiting(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.showing.is_none() || !self.read_now(cx) {
            return;
        }
        let (range, focus, expected) = self.showing.take().expect("checked above");
        if self.edits != expected {
            return log::info!("markdown document: a waiting match lapsed with its text");
        }
        self.shown(range, focus, window, cx);
    }

    /// `edits` as one undo step; false in preview.
    pub fn replace(&mut self, edits: Vec<(Range<usize>, String)>, cx: &mut Context<Self>) -> bool {
        if self.mode == MarkdownMode::Preview {
            return false;
        }
        let edits = self.beside_open(edits, cx);
        let text = self.editor.read(cx).text();
        if edits
            .iter()
            .all(|(range, with)| text[range.clone()] == *with)
        {
            return true;
        }
        log::info!("markdown document: {} replaced", edits.len());
        // Its own change: an open block outlives it.
        self.writes += 1;
        self.editor.update(cx, |editor, cx| editor.edit(edits, cx));
        true
    }

    /// `edits` of the text as read, in the editor's bytes.
    ///
    /// The open block stays and moves with the edits before it; an edit
    /// reaching into it writes it back first.
    fn beside_open(
        &mut self,
        edits: Vec<(Range<usize>, String)>,
        cx: &mut Context<Self>,
    ) -> Vec<(Range<usize>, String)> {
        let Some(open) = &self.open else {
            return edits;
        };
        let held = open.range.clone();
        let written = Self::typed(open, self.editor.read(cx).text(), cx).len();
        let shown = held.start..held.start + written;
        if edits
            .iter()
            .any(|(edit, _)| edit.start < shown.end && edit.end > shown.start)
        {
            log::info!("markdown document: a replacement reaches into the open block");
            self.commit(cx);
            return edits;
        }
        let before: isize = (edits.iter())
            .filter(|(edit, _)| edit.end <= shown.start)
            .map(|(edit, with)| with.len() as isize - edit.len() as isize)
            .sum();
        let moved = |byte: usize| byte.wrapping_add_signed(before);
        let after = edited(&self.text(cx), &edits);
        if !open.append && !stands(&after, moved(shown.start)..moved(shown.end), open.cell) {
            // Written first: undo takes the replacement alone.
            log::info!("markdown document: a replacement moves the open block's bounds");
            self.commit(cx);
            return edits;
        }
        // Typing not yet written shifts every byte after it.
        let unwritten = written as isize - held.len() as isize;
        let edits = edits
            .into_iter()
            .map(|(edit, with)| match edit.end <= shown.start {
                true => (edit, with),
                false => {
                    let at = |byte: usize| byte.wrapping_add_signed(-unwritten);
                    (at(edit.start)..at(edit.end), with)
                }
            })
            .collect();
        if let Some(open) = self.open.as_mut() {
            open.range = moved(held.start)..moved(held.end);
        }
        edits
    }
}

/// `text` with `edits` made, however they are ordered.
fn edited(text: &str, edits: &[(Range<usize>, String)]) -> String {
    let mut sorted: Vec<_> = edits.iter().collect();
    sorted.sort_by_key(|(range, _)| range.start);
    let (mut out, mut at) = (String::new(), 0);
    for (range, with) in sorted {
        out.push_str(&text[at..range.start]);
        out.push_str(with);
        at = range.end;
    }
    out + &text[at..]
}

/// Whether `range` of `text` is still one block, or that cell.
fn stands(text: &str, range: Range<usize>, cell: Option<CellAt>) -> bool {
    let read = parsed(text);
    let holds = |block: &&Block| block.range.start <= range.start && range.end <= block.range.end;
    match (read.blocks.iter().find(holds), cell) {
        (Some(block), Some(at)) => cells::find(text, block.range.clone(), Aim::At(at))
            .is_some_and(|cell| cell.range == range),
        (Some(block), None) => block.range == range,
        (None, _) => false,
    }
}
