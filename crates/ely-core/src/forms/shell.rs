//! Bash's lexer, read for where comments start.

mod brace;
mod delimiter;

use delimiter::{delimiter, line};

/// Byte offsets of every `#` that opens a Bash comment.
pub(crate) fn comments(source: &str) -> Vec<usize> {
    let mut lexer = Lexer::new(source.as_bytes(), 0);
    lexer.list(Until::End);
    lexer.marks
}

/// What closes the list being read.
#[derive(Clone, Copy, PartialEq)]
enum Until {
    End,
    Paren,
    Clause,
}

/// Why a list stopped.
#[derive(PartialEq)]
enum Stop {
    End,
    Paren,
    Clause,
    Esac,
}

enum Token {
    End,
    Newline,
    Operator(&'static [u8]),
    /// A word's bytes; plain when unquoted and unescaped.
    Word {
        start: usize,
        end: usize,
        plain: bool,
    },
}

/// Longest first, as Bash matches them.
const OPERATORS: [&[u8]; 21] = [
    b";;&", b"<<-", b"<<<", b"&>>", b";;", b";&", b"&&", b"||", b"|&", b"<<", b">>", b"<&", b">&",
    b"<>", b">|", b"&>", b";", b"&", b"|", b"<", b">",
];

/// Reserved words after which a command starts.
const OPENERS: [&[u8]; 13] = [
    b"then", b"do", b"done", b"else", b"elif", b"fi", b"if", b"while", b"until", b"esac", b"{",
    b"}", b"!",
];

/// What `time` or `coproc` leaves open.
#[derive(Clone, Copy, PartialEq)]
enum Prefix {
    None,
    Time,
    Coproc,
    Function,
}

/// Where a `$` stands, for how `$'` and `$"` read.
#[derive(Clone, Copy, PartialEq)]
enum Quoting {
    Bare,
    Double,
    Here,
}

/// A here-doc waiting for its body.
struct Doc {
    delimiter: Vec<u8>,
    tabs: bool,
    /// An unquoted delimiter: the body expands.
    expands: bool,
}

struct Lexer<'a> {
    s: &'a [u8],
    at: usize,
    marks: Vec<usize>,
    /// Here-doc delimiters waiting for the next line.
    docs: Vec<Doc>,
}

impl<'a> Lexer<'a> {
    fn new(s: &'a [u8], at: usize) -> Self {
        Lexer {
            s,
            at,
            marks: Vec::new(),
            docs: Vec::new(),
        }
    }

    fn byte(&self, at: usize) -> Option<u8> {
        self.s.get(at).copied()
    }

    /// Reads commands until `until` closes them.
    fn list(&mut self, until: Until) -> Stop {
        let mut command = true;
        let mut prefix = Prefix::None;
        loop {
            match self.token() {
                Token::End => return Stop::End,
                Token::Newline => command = true,
                Token::Operator(b")") if until == Until::Paren => return Stop::Paren,
                Token::Operator(b";;" | b";&" | b";;&") if until == Until::Clause => {
                    return Stop::Clause;
                }
                Token::Operator(b"(") => {
                    self.list(Until::Paren);
                    command = true;
                }
                Token::Operator(op)
                    if op.starts_with(b"<")
                        || op.starts_with(b">")
                        || op == b"&>"
                        || op == b"&>>" =>
                {
                    // The target is a word, not a command.
                    if !matches!(op, b"<<" | b"<<-") {
                        self.token();
                    }
                }
                Token::Operator(_) => {
                    command = true;
                    prefix = Prefix::None;
                }
                Token::Word { start, end, plain } => {
                    let word = self.text(start, end);
                    let word = word.as_slice();
                    let after = std::mem::replace(&mut prefix, Prefix::None);
                    if after == Prefix::Function {
                        // The name; its body starts a command.
                        command = true;
                    } else if !(command && plain) {
                        command = false;
                    } else if word == b"case" {
                        self.case();
                    } else if word == b"esac" && until == Until::Clause {
                        return Stop::Esac;
                    } else if word == b"time"
                        || (after == Prefix::Time && matches!(word, b"-p" | b"--"))
                    {
                        prefix = Prefix::Time;
                    } else if word == b"coproc" {
                        prefix = Prefix::Coproc;
                    } else if word == b"function" {
                        prefix = Prefix::Function;
                    } else {
                        // A coprocess's name leaves command position.
                        command = OPENERS.contains(&word) || after == Prefix::Coproc;
                    }
                }
            }
        }
    }

    /// Reads a `case` after its keyword, through `esac`.
    fn case(&mut self) {
        self.token();
        loop {
            match self.token() {
                Token::Newline => {}
                Token::Word { start, end, plain } if plain && self.text(start, end) == b"in" => {
                    break;
                }
                _ => return,
            }
        }
        loop {
            // A clause's first word `esac` ends the case.
            let mut first = true;
            loop {
                match self.token() {
                    Token::End => return,
                    Token::Newline => {}
                    Token::Word { start, end, plain }
                        if first && plain && self.text(start, end) == b"esac" =>
                    {
                        return;
                    }
                    Token::Operator(b")") => break,
                    _ => first = false,
                }
            }
            match self.list(Until::Clause) {
                Stop::Clause => {}
                _ => return,
            }
        }
    }

    /// Skips blanks and line continuations.
    fn blanks(&mut self) {
        loop {
            match self.byte(self.at) {
                Some(b' ' | b'\t') => self.at += 1,
                Some(b'\\') if self.byte(self.at + 1) == Some(b'\n') => self.at += 2,
                _ => return,
            }
        }
    }

    fn token(&mut self) -> Token {
        loop {
            self.blanks();
            let Some(byte) = self.byte(self.at) else {
                return Token::End;
            };
            match byte {
                b'\n' => {
                    self.at += 1;
                    self.bodies();
                    return Token::Newline;
                }
                b'#' => {
                    self.marks.push(self.at);
                    while self.byte(self.at).is_some_and(|b| b != b'\n') {
                        self.at += 1;
                    }
                }
                b'<' | b'>' if self.byte(self.at + 1) == Some(b'(') => return self.word(),
                b'(' | b')' => {
                    self.at += 1;
                    return Token::Operator(if byte == b'(' { b"(" } else { b")" });
                }
                _ => {
                    let rest = &self.s[self.at..];
                    let Some(op) = OPERATORS.iter().find(|op| rest.starts_with(op)) else {
                        return self.word();
                    };
                    self.at += op.len();
                    if matches!(*op, b"<<" | b"<<-") {
                        self.doc(*op == b"<<-");
                    }
                    return Token::Operator(op);
                }
            }
        }
    }

    /// Reads a here-doc's delimiter; its body follows the line.
    fn doc(&mut self, tabs: bool) {
        self.blanks();
        let start = self.at;
        let Token::Word { end, .. } = self.word() else {
            return;
        };
        let (delimiter, expands) = delimiter(&self.s[start..end]);
        self.docs.push(Doc {
            delimiter,
            tabs,
            expands,
        });
    }

    /// Skips the bodies of here-docs begun on the last line.
    fn bodies(&mut self) {
        for doc in std::mem::take(&mut self.docs) {
            let start = self.at;
            let mut body = self.s.len();
            while self.at < self.s.len() {
                let (text, next) = line(self.s, self.at, doc.expands);
                let mut text = text.as_slice();
                if doc.tabs {
                    while let [b'\t', rest @ ..] = text {
                        text = rest;
                    }
                }
                if text == doc.delimiter.as_slice() {
                    body = self.at;
                    self.at = next;
                    break;
                }
                self.at = next;
            }
            if doc.expands {
                self.inside(start, body, |inner| inner.quoted(None, Quoting::Here));
            }
        }
    }

    /// A word's bytes, line continuations removed.
    fn text(&self, start: usize, end: usize) -> Vec<u8> {
        let mut text = Vec::with_capacity(end - start);
        let mut at = start;
        while at < end {
            if self.s[at] == b'\\' && self.s.get(at + 1) == Some(&b'\n') {
                at += 2;
            } else {
                text.push(self.s[at]);
                at += 1;
            }
        }
        text
    }

    /// A backquoted command, read as Shell up to its close.
    fn backtick(&mut self) {
        let start = self.at;
        while let Some(byte) = self.byte(self.at) {
            match byte {
                b'\\' => self.at += 2,
                b'`' => break,
                _ => self.at += 1,
            }
        }
        let close = self.at.min(self.s.len());
        self.inside(start, close, |inner| {
            inner.list(Until::End);
        });
        self.at = (close + 1).min(self.s.len());
    }

    /// Reads `start..end` alone with `read`, keeping its comments.
    fn inside(&mut self, start: usize, end: usize, read: impl FnOnce(&mut Lexer)) {
        let mut inner = Lexer::new(&self.s[..end], start);
        read(&mut inner);
        self.marks.extend(inner.marks);
    }

    /// Steps over an escape, quote or expansion at the cursor.
    fn nested(&mut self, quoting: Quoting) -> bool {
        match self.byte(self.at) {
            Some(b'\\') => self.at = (self.at + 2).min(self.s.len()),
            Some(b'\'') if quoting == Quoting::Bare => {
                self.at += 1;
                self.until(b'\'', false);
            }
            Some(b'"') if quoting == Quoting::Bare => {
                self.at += 1;
                self.quoted(Some(b'"'), Quoting::Double);
            }
            Some(b'`') => {
                self.at += 1;
                self.backtick();
            }
            Some(b'$') => {
                self.at += 1;
                self.dollar(quoting);
            }
            _ => return false,
        }
        true
    }

    /// Reads one word, quotes and expansions inside it.
    fn word(&mut self) -> Token {
        let start = self.at;
        let mut plain = true;
        while let Some(byte) = self.byte(self.at) {
            let next = self.byte(self.at + 1);
            match byte {
                b' ' | b'\t' | b'\n' | b';' | b'&' | b'|' | b')' => break,
                b'(' if self.at > start => break,
                b'<' | b'>' if next == Some(b'(') => {
                    self.at += 2;
                    self.list(Until::Paren);
                }
                b'<' | b'>' | b'(' => break,
                b'\\' | b'\'' | b'"' | b'`' | b'$' => {
                    plain &= byte == b'$' || (byte == b'\\' && next == Some(b'\n'));
                    self.nested(Quoting::Bare);
                }
                b'@' | b'?' | b'*' | b'+' | b'!' if next == Some(b'(') => {
                    self.at += 2;
                    self.glob();
                }
                _ => self.at += 1,
            }
        }
        Token::Word {
            start,
            end: self.at,
            plain,
        }
    }

    /// Skips past `close`; backslashes escape when `escapes`.
    fn until(&mut self, close: u8, escapes: bool) {
        while let Some(byte) = self.byte(self.at) {
            self.at += if escapes && byte == b'\\' { 2 } else { 1 };
            if byte == close {
                return;
            }
        }
        self.at = self.at.min(self.s.len());
    }

    /// Through `close`, or the end; only expansions inside.
    fn quoted(&mut self, close: Option<u8>, quoting: Quoting) {
        while let Some(byte) = self.byte(self.at) {
            if Some(byte) == close {
                self.at += 1;
                return;
            }
            if !self.nested(quoting) {
                self.at += 1;
            }
        }
    }

    /// After a `$`; `$'` and `$"` quote only when bare.
    fn dollar(&mut self, quoting: Quoting) {
        let quotes = quoting == Quoting::Bare;
        let rest = &self.s[self.at..];
        if rest.starts_with(b"((") {
            self.at += 2;
            self.balanced(2);
        } else if rest.starts_with(b"(") {
            self.at += 1;
            self.list(Until::Paren);
        } else if rest.starts_with(b"{") {
            self.at += 1;
            self.brace(quoting);
        } else if quotes && rest.starts_with(b"'") {
            self.at += 1;
            self.until(b'\'', true);
        } else if quotes && rest.starts_with(b"\"") {
            self.at += 1;
            self.quoted(Some(b'"'), Quoting::Double);
        }
    }

    /// Arithmetic: `depth` parens open; no comments inside.
    fn balanced(&mut self, mut depth: usize) {
        while let Some(byte) = self.byte(self.at) {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        self.at += 1;
                        return;
                    }
                }
                _ if self.nested(Quoting::Bare) => continue,
                _ => {}
            }
            self.at += 1;
        }
    }

    /// An extglob's pattern, through its closing paren.
    fn glob(&mut self) {
        let mut depth = 1;
        while let Some(byte) = self.byte(self.at) {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        self.at += 1;
                        return;
                    }
                }
                _ if self.nested(Quoting::Bare) => continue,
                _ => {}
            }
            self.at += 1;
        }
    }
}

#[cfg(test)]
mod tests;
