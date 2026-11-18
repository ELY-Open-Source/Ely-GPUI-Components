//! A here-doc's delimiter and lines, as Bash reads them.

use super::{Lexer, Quoting};

/// The delimiter's text, and whether the body expands.
pub(super) fn delimiter(word: &[u8]) -> (Vec<u8>, bool) {
    let mut text = Vec::with_capacity(word.len());
    if lexed(word, &mut text, false, true) {
        (removed(&text), false)
    } else {
        (text, true)
    }
}

/// The line at `at`, joined across continuations when `joins`; the next's start.
pub(super) fn line(s: &[u8], mut at: usize, joins: bool) -> (Vec<u8>, usize) {
    let mut text = Vec::new();
    while let Some(&byte) = s.get(at) {
        match (byte, s.get(at + 1).copied()) {
            (b'\n', _) => return (text, at + 1),
            (b'\\', Some(b'\n')) if joins => at += 2,
            (b'\\', Some(next)) if joins => {
                text.extend([byte, next]);
                at += 2;
            }
            _ => {
                text.push(byte);
                at += 1;
            }
        }
    }
    (text, at)
}

/// Appends `word` as Bash's lexer keeps it; true when quoted at its top.
fn lexed(word: &[u8], out: &mut Vec<u8>, in_double: bool, quote_ansi: bool) -> bool {
    let mut quoted = false;
    let mut at = 0;
    while let Some(&byte) = word.get(at) {
        let next = word.get(at + 1).copied();
        quoted |= matches!((byte, next), (b'\'' | b'"', _) | (b'$', Some(b'\'' | b'"')))
            || (byte == b'\\' && next != Some(b'\n'));
        at = match (byte, next) {
            (b'$', Some(b'\'')) => translated(word, at + 2, out, quote_ansi),
            (b'$', Some(b'"')) => at + 1,
            (b'"', _) => doubled(word, at, out),
            (b'\'', _) => {
                let end = (closing(word, at + 1) + 1).min(word.len());
                out.extend_from_slice(&word[at..end]);
                end
            }
            _ => step(word, at, out, in_double),
        };
    }
    quoted
}

/// Appends a double-quoted string at `at`; returns its end.
fn doubled(word: &[u8], mut at: usize, out: &mut Vec<u8>) -> usize {
    out.push(b'"');
    at += 1;
    while let Some(&byte) = word.get(at) {
        if byte == b'"' {
            out.push(byte);
            return at + 1;
        }
        at = step(word, at, out, true);
    }
    at
}

/// One escape, expansion or byte at `at`; returns its end.
fn step(word: &[u8], at: usize, out: &mut Vec<u8>, in_double: bool) -> usize {
    match (word[at], word.get(at + 1).copied()) {
        (b'\\', Some(b'\n')) => at + 2,
        (b'\\', Some(next)) => {
            out.extend([b'\\', next]);
            at + 2
        }
        (b'$', Some(b'(')) => {
            let end = span(word, at);
            out.extend_from_slice(&word[at..at + 2]);
            lexed(&word[at + 2..end], out, false, true);
            end
        }
        // Quoted, only a pattern keeps `$'` results quoted.
        (b'$', Some(b'{')) => {
            let end = span(word, at);
            let inner = &word[at + 2..end];
            out.extend_from_slice(&word[at..at + 2]);
            let quote_ansi = !in_double || pattern(inner);
            lexed(inner, out, in_double, quote_ansi);
            end
        }
        // Backquotes keep `$'`; only continuations go.
        (b'`', _) => {
            let end = span(word, at);
            let mut inner = at;
            while inner < end {
                inner = match (word[inner], word.get(inner + 1).copied()) {
                    (b'\\', Some(b'\n')) => inner + 2,
                    (b'\\', Some(next)) => {
                        out.extend([b'\\', next]);
                        inner + 2
                    }
                    (byte, _) => {
                        out.push(byte);
                        inner + 1
                    }
                };
            }
            end
        }
        (byte, _) => {
            out.push(byte);
            at + 1
        }
    }
}

/// Whether Bash's parser reads this `${...}` as a pattern.
fn pattern(inner: &[u8]) -> bool {
    let first = inner.iter().position(|b| b"#%^,~:-=?+/".contains(b));
    first.is_some_and(|at| at > 0 && b"%#/^,".contains(&inner[at]))
}

/// Where the expansion at `at` ends, as the lexer reads it.
fn span(word: &[u8], at: usize) -> usize {
    let mut lexer = Lexer::new(word, at);
    lexer.nested(Quoting::Bare);
    lexer.at.min(word.len())
}

/// The next `'` from `at`, or the end.
fn closing(word: &[u8], at: usize) -> usize {
    word[at.min(word.len())..]
        .iter()
        .position(|b| *b == b'\'')
        .map_or(word.len(), |n| at + n)
}

/// Appends `$'...'` from `at`, translated, single-quoted; returns its end.
fn translated(word: &[u8], mut at: usize, out: &mut Vec<u8>, quoted: bool) -> usize {
    let mut text = Vec::new();
    while let Some(&byte) = word.get(at).filter(|b| **b != b'\'') {
        at += 1;
        if byte == b'\\' {
            at = ansi(word, at, &mut text);
        } else {
            text.push(byte);
        }
    }
    if !quoted {
        out.extend_from_slice(&text);
        return at + 1;
    }
    // Bash quotes a lone quote as `\'`.
    if text == b"'" {
        out.extend_from_slice(b"\\'");
        return at + 1;
    }
    out.push(b'\'');
    for byte in text {
        if byte == b'\'' {
            out.extend_from_slice(b"'\\''");
        } else {
            out.push(byte);
        }
    }
    out.push(b'\'');
    at + 1
}

/// Bash's quote removal of a lexed word.
fn removed(word: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(word.len());
    let mut double = false;
    let mut at = 0;
    while let Some(&byte) = word.get(at) {
        at += 1;
        match byte {
            b'\\' => {
                let Some(&next) = word.get(at) else {
                    out.push(byte);
                    break;
                };
                if double && !matches!(next, b'$' | b'`' | b'"' | b'\\' | b'\n') {
                    out.push(byte);
                }
                out.push(next);
                at += 1;
            }
            b'\'' if !double => {
                let end = closing(word, at);
                out.extend_from_slice(&word[at..end]);
                at = end + 1;
            }
            b'"' => double = !double,
            _ => out.push(byte),
        }
    }
    out
}

/// One ANSI-C escape after its backslash; returns where it ends.
fn ansi(word: &[u8], at: usize, out: &mut Vec<u8>) -> usize {
    let Some(&code) = word.get(at) else {
        out.push(b'\\');
        return at;
    };
    // Up to `most` digits in `radix`, read as one number.
    let digits = |from: usize, radix: u32, most: usize| {
        let run = word[from..]
            .iter()
            .take(most)
            .take_while(|b| char::from(**b).is_digit(radix))
            .count();
        let text = std::str::from_utf8(&word[from..from + run]).unwrap_or_default();
        (u32::from_str_radix(text, radix).ok(), from + run)
    };
    let simple = match code {
        b'a' => Some(0x07),
        b'b' => Some(0x08),
        b'e' | b'E' => Some(0x1b),
        b'f' => Some(0x0c),
        b'n' => Some(b'\n'),
        b'r' => Some(b'\r'),
        b't' => Some(b'\t'),
        b'v' => Some(0x0b),
        b'\\' | b'\'' | b'"' | b'?' => Some(code),
        _ => None,
    };
    if let Some(byte) = simple {
        out.push(byte);
        return at + 1;
    }
    let (value, end) = match code {
        b'0'..=b'7' => digits(at, 8, 3),
        b'x' => digits(at + 1, 16, 2),
        b'u' => digits(at + 1, 16, 4),
        b'U' => digits(at + 1, 16, 8),
        b'c' if word.get(at + 1).is_some() => {
            out.push(word[at + 1] & 0x1f);
            return at + 2;
        }
        _ => (None, at),
    };
    match (code, value) {
        (b'u' | b'U', Some(value)) => {
            let mut buffer = [0; 4];
            let ch = char::from_u32(value).unwrap_or(char::REPLACEMENT_CHARACTER);
            out.extend_from_slice(ch.encode_utf8(&mut buffer).as_bytes());
            end
        }
        (_, Some(value)) => {
            out.push(value as u8);
            end
        }
        // Bash keeps an unknown escape's backslash.
        _ => {
            out.extend_from_slice(&[b'\\', code]);
            at + 1
        }
    }
}
