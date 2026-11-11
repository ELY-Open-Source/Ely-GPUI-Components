//! Shell's open parens and `case` clauses, read left to right.

use std::ops::Range;

/// What is open where the lexer stands.
#[derive(Default)]
pub(crate) struct Nesting {
    opens: Vec<Open>,
}

enum Open {
    Paren {
        expansion: bool,
    },
    /// An extglob's parens: pattern text inside.
    Glob,
    Case {
        keyword_end: usize,
        at: Clause,
    },
}

/// Where a `case` clause reads.
enum Clause {
    Subject,
    /// Patterns, listed from byte `from`.
    Patterns {
        from: usize,
    },
    Body,
}

impl Nesting {
    /// Reads the word at `range`.
    pub(crate) fn word(&mut self, source: &str, range: Range<usize>) {
        if !whole(source, &range) {
            return;
        }
        let word = &source[range.clone()];
        let before = &source[..range.start];
        match self.opens.last_mut() {
            Some(Open::Case {
                keyword_end,
                at: at @ Clause::Subject,
            }) if word == "in" && !source[*keyword_end..range.start].trim().is_empty() => {
                *at = Clause::Patterns { from: range.end };
            }
            Some(Open::Case {
                at: Clause::Patterns { from },
                ..
            }) if word == "esac" && blank(&source[*from..range.start]) => {
                self.opens.pop();
            }
            // Any other pattern is a plain word, `case` and `esac` too.
            Some(Open::Case {
                at: Clause::Patterns { .. },
                ..
            })
            | Some(Open::Glob) => {}
            Some(Open::Case { .. }) if word == "esac" && reserved(before) => {
                self.opens.pop();
            }
            _ if word == "case" && reserved(before) => self.opens.push(Open::Case {
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
        if glob || (!expansion && matches!(self.opens.last(), Some(Open::Glob))) {
            self.opens.push(Open::Glob);
            return;
        }
        // A pattern's optional `(` pairs with its `)`.
        if !expansion
            && let Some(Open::Case {
                at: Clause::Patterns { .. },
                ..
            }) = self.opens.last()
        {
            return;
        }
        self.opens.push(Open::Paren { expansion });
    }

    /// Reads an unescaped `)`: whether it closes an expansion.
    pub(crate) fn close(&mut self) -> bool {
        match self.opens.last_mut() {
            Some(Open::Case {
                at: at @ Clause::Patterns { .. },
                ..
            }) => {
                *at = Clause::Body;
                false
            }
            Some(Open::Glob) => {
                self.opens.pop();
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
            let end = if source[start..].starts_with(";;&") {
                3
            } else {
                2
            };
            *at = Clause::Patterns { from: start + end };
        }
    }
}

/// Whether `gap` holds only blanks and comments.
fn blank(gap: &str) -> bool {
    // A backslash before a newline joins the lines.
    gap.replace("\\\n", "").lines().all(|line| {
        let line = line.trim();
        line.is_empty() || line.starts_with('#')
    })
}

/// Reserved words after which another reserved word may start.
const OPENERS: [&str; 15] = [
    "then", "do", "done", "else", "elif", "fi", "if", "while", "until", "esac", "{", "}", "!",
    "time", "coproc",
];

/// Whether Bash takes a reserved word after `before`.
fn reserved(before: &str) -> bool {
    let line = before.trim_end_matches([' ', '\t']);
    if line.is_empty() || line.ends_with([';', '&', '|', '(', ')', '\n']) {
        return true;
    }
    let last = line.rsplit(bound).next().unwrap_or_default();
    OPENERS.contains(&last) && reserved(&line[..line.len() - last.len()])
}

/// Whether the lexer's word is a whole Shell word.
fn whole(source: &str, range: &Range<usize>) -> bool {
    let edge = |ch: Option<char>| ch.is_none_or(bound);
    edge(source[..range.start].chars().next_back()) && edge(source[range.end..].chars().next())
}

/// Whether `ch` ends a Shell word.
fn bound(ch: char) -> bool {
    ch.is_whitespace() || ";&|()<>".contains(ch)
}
