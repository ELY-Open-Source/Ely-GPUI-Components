//! Shell's open parens and `case` clauses, read left to right.

use std::ops::Range;

/// What is open where the lexer stands.
#[derive(Default)]
pub(crate) struct Nesting {
    opens: Vec<Open>,
}

enum Open {
    Paren { expansion: bool },
    Case { keyword_end: usize, at: Clause },
}

/// Where a `case` clause reads.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Clause {
    Subject,
    Patterns,
    Body,
}

impl Nesting {
    /// Reads the word at `range`.
    pub(crate) fn word(&mut self, source: &str, range: Range<usize>) {
        let word = &source[range.clone()];
        let before = &source[..range.start];
        match self.opens.last_mut() {
            Some(Open::Case {
                keyword_end,
                at: at @ Clause::Subject,
            }) if word == "in" && !source[*keyword_end..range.start].trim().is_empty() => {
                *at = Clause::Patterns;
            }
            Some(Open::Case { at, .. })
                if word == "esac" && (*at == Clause::Patterns || command_starts(before)) =>
            {
                self.opens.pop();
            }
            _ if word == "case" && command_starts(before) => self.opens.push(Open::Case {
                keyword_end: range.end,
                at: Clause::Subject,
            }),
            _ => {}
        }
    }

    /// Reads an unescaped `(` at `start`.
    pub(crate) fn open(&mut self, source: &str, start: usize) {
        let before = &source[..start];
        let expansion = before.ends_with(['$', '<', '>']);
        let glob = before.ends_with(['@', '?', '*', '+', '!']);
        // A pattern's optional `(` pairs with its `)`.
        if !expansion && !glob && self.reads(Clause::Patterns) {
            return;
        }
        self.opens.push(Open::Paren { expansion });
    }

    /// Reads an unescaped `)`: whether it closes an expansion.
    pub(crate) fn close(&mut self) -> bool {
        match self.opens.last_mut() {
            Some(Open::Case {
                at: at @ Clause::Patterns,
                ..
            }) => {
                *at = Clause::Body;
                false
            }
            Some(Open::Paren { .. }) => {
                matches!(self.opens.pop(), Some(Open::Paren { expansion: true }))
            }
            _ => false,
        }
    }

    /// Reads an unescaped `;` at `start`.
    pub(crate) fn semicolon(&mut self, source: &str, start: usize) {
        if let Some(Open::Case {
            at: at @ Clause::Body,
            ..
        }) = self.opens.last_mut()
            && source[start + 1..].starts_with([';', '&'])
        {
            *at = Clause::Patterns;
        }
    }

    fn reads(&self, clause: Clause) -> bool {
        matches!(self.opens.last(), Some(Open::Case { at, .. }) if *at == clause)
    }
}

/// Whether a reserved word may start after `before`.
fn command_starts(before: &str) -> bool {
    let line = before.trim_end_matches([' ', '\t']);
    let last = line
        .rsplit(|ch: char| ch.is_whitespace() || ";&|()".contains(ch))
        .next()
        .unwrap_or_default();
    line.is_empty()
        || line.ends_with([';', '&', '|', '(', ')', '\n'])
        || [
            "then", "do", "else", "elif", "if", "while", "until", "{", "!", "time",
        ]
        .contains(&last)
}
