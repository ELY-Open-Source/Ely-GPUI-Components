//! A here-doc delimiter, quotes removed as Bash does.

/// A here-doc delimiter after Bash's quote removal.
pub(super) fn unquoted(word: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(word.len());
    let mut at = 0;
    while let Some(&byte) = word.get(at) {
        let next = word.get(at + 1).copied();
        match byte {
            b'\\' if next == Some(b'\n') => at += 2,
            b'\\' => {
                out.extend(next);
                at += 2;
            }
            b'\'' => {
                let close = word[at + 1..]
                    .iter()
                    .position(|b| *b == b'\'')
                    .map_or(word.len(), |n| at + 1 + n);
                out.extend_from_slice(&word[at + 1..close]);
                at = close + 1;
            }
            b'"' => {
                at += 1;
                while let Some(&inner) = word.get(at).filter(|b| **b != b'"') {
                    let escaped = word
                        .get(at + 1)
                        .copied()
                        .filter(|b| matches!(b, b'$' | b'`' | b'"' | b'\\' | b'\n'));
                    match (inner, escaped) {
                        (b'\\', Some(b'\n')) => at += 2,
                        (b'\\', Some(kept)) => {
                            out.push(kept);
                            at += 2;
                        }
                        _ => {
                            out.push(inner);
                            at += 1;
                        }
                    }
                }
                at += 1;
            }
            b'$' if next == Some(b'\'') => {
                at += 2;
                while let Some(&inner) = word.get(at).filter(|b| **b != b'\'') {
                    at += 1;
                    if inner == b'\\' {
                        at = ansi(word, at, &mut out);
                    } else {
                        out.push(inner);
                    }
                }
                at += 1;
            }
            b'$' if next == Some(b'"') => at += 1,
            _ => {
                out.push(byte);
                at += 1;
            }
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
