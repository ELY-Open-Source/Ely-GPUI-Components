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
                    if inner == b'\\' {
                        out.extend(word.get(at + 1).map(|c| match c {
                            b'n' => b'\n',
                            b't' => b'\t',
                            other => *other,
                        }));
                        at += 2;
                    } else {
                        out.push(inner);
                        at += 1;
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
