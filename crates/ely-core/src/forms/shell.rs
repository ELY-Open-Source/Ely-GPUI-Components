//! Bash's lexer, read for where comments start.

/// Byte offsets of every `#` that opens a Bash comment.
pub(crate) fn comments(source: &str) -> Vec<usize> {
    let mut lexer = Lexer {
        s: source.as_bytes(),
        at: 0,
        marks: Vec::new(),
        docs: Vec::new(),
    };
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
const OPENERS: [&[u8]; 15] = [
    b"then", b"do", b"done", b"else", b"elif", b"fi", b"if", b"while", b"until", b"esac", b"{",
    b"}", b"!", b"time", b"coproc",
];

struct Lexer<'a> {
    s: &'a [u8],
    at: usize,
    marks: Vec<usize>,
    /// Here-doc delimiters waiting for the next line.
    docs: Vec<(Vec<u8>, bool)>,
}

impl Lexer<'_> {
    fn byte(&self, at: usize) -> Option<u8> {
        self.s.get(at).copied()
    }

    /// Reads commands until `until` closes them.
    fn list(&mut self, until: Until) -> Stop {
        let mut command = true;
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
                Token::Operator(_) => command = true,
                Token::Word { start, end, plain } => {
                    let word = &self.s[start..end];
                    if !(command && plain) {
                        command = false;
                    } else if word == b"case" {
                        self.case();
                    } else if word == b"esac" && until == Until::Clause {
                        return Stop::Esac;
                    } else {
                        command = OPENERS.contains(&word);
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
                Token::Word { start, end, plain } if plain && &self.s[start..end] == b"in" => break,
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
                        if first && plain && &self.s[start..end] == b"esac" =>
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
        let delimiter = self.s[start..end]
            .iter()
            .copied()
            .filter(|b| !matches!(b, b'\'' | b'"' | b'\\'))
            .collect();
        self.docs.push((delimiter, tabs));
    }

    /// Skips the bodies of here-docs begun on the last line.
    fn bodies(&mut self) {
        for (delimiter, tabs) in std::mem::take(&mut self.docs) {
            while self.at < self.s.len() {
                let end = self.s[self.at..]
                    .iter()
                    .position(|b| *b == b'\n')
                    .map_or(self.s.len(), |n| self.at + n);
                let mut line = &self.s[self.at..end];
                if tabs {
                    while let [b'\t', rest @ ..] = line {
                        line = rest;
                    }
                }
                self.at = (end + 1).min(self.s.len());
                if line == delimiter.as_slice() {
                    break;
                }
            }
        }
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
                b'\\' => {
                    plain = false;
                    self.at = (self.at + 2).min(self.s.len());
                }
                b'\'' => {
                    plain = false;
                    self.at += 1;
                    self.until(b'\'', false);
                }
                b'"' => {
                    plain = false;
                    self.at += 1;
                    self.quoted();
                }
                b'`' => {
                    plain = false;
                    self.at += 1;
                    self.until(b'`', true);
                }
                b'$' => {
                    self.at += 1;
                    self.dollar();
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

    /// Inside double quotes, through the closing one.
    fn quoted(&mut self) {
        while let Some(byte) = self.byte(self.at) {
            match byte {
                b'"' => {
                    self.at += 1;
                    return;
                }
                b'\\' => self.at += 2,
                b'`' => {
                    self.at += 1;
                    self.until(b'`', true);
                }
                b'$' => {
                    self.at += 1;
                    self.dollar();
                }
                _ => self.at += 1,
            }
        }
        self.at = self.at.min(self.s.len());
    }

    /// After a `$`: an expansion, or a plain dollar.
    fn dollar(&mut self) {
        let rest = &self.s[self.at..];
        if rest.starts_with(b"((") {
            self.at += 2;
            self.balanced(2);
        } else if rest.starts_with(b"(") {
            self.at += 1;
            self.list(Until::Paren);
        } else if rest.starts_with(b"{") {
            self.at += 1;
            self.brace();
        } else if rest.starts_with(b"'") {
            self.at += 1;
            self.until(b'\'', true);
        } else if rest.starts_with(b"\"") {
            self.at += 1;
            self.quoted();
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
                b'\\' => self.at += 1,
                b'$' => {
                    self.at += 1;
                    self.dollar();
                    continue;
                }
                _ => {}
            }
            self.at += 1;
        }
        self.at = self.at.min(self.s.len());
    }

    /// `${...}`, nested expansions and quotes inside.
    fn brace(&mut self) {
        while let Some(byte) = self.byte(self.at) {
            match byte {
                b'}' => {
                    self.at += 1;
                    return;
                }
                b'\\' => self.at += 2,
                b'\'' => {
                    self.at += 1;
                    self.until(b'\'', false);
                }
                b'"' => {
                    self.at += 1;
                    self.quoted();
                }
                b'$' => {
                    self.at += 1;
                    self.dollar();
                }
                _ => self.at += 1,
            }
        }
        self.at = self.at.min(self.s.len());
    }

    /// An extglob's pattern, through its closing paren.
    fn glob(&mut self) {
        let mut depth = 1;
        while let Some(byte) = self.byte(self.at) {
            self.at += 1;
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return;
                    }
                }
                b'\\' => self.at += 1,
                b'\'' => self.until(b'\'', false),
                b'"' => self.quoted(),
                _ => {}
            }
        }
        self.at = self.at.min(self.s.len());
    }
}

#[cfg(test)]
mod tests;
