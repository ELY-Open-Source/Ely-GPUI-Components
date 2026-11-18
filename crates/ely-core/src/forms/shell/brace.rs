//! `${...}`, read as Bash reads its operand.

use super::{Lexer, Quoting};

/// Whether the `${...}` at `s` is `-`, `=`, `?` or `+`, maybe after `:`.
fn defaulted(s: &[u8]) -> bool {
    let mut at = 0;
    while let Some(&byte) = s.get(at) {
        match byte {
            b'[' => {
                at += s[at..]
                    .iter()
                    .position(|b| *b == b']')
                    .unwrap_or(s.len() - at)
            }
            b'-' | b'=' | b'?' | b'+' => return true,
            b':' => return matches!(s.get(at + 1), Some(b'-' | b'=' | b'?' | b'+')),
            b'#' | b'%' | b'/' | b'^' | b',' if at > 0 => return false,
            b'~' | b'}' => return false,
            _ => {}
        }
        at += 1;
    }
    false
}

/// The quote group after `quote`, for finding the close.
fn grouped(group: Option<u8>, quote: u8) -> Option<u8> {
    match group {
        None => Some(quote),
        Some(open) if open == quote => None,
        other => other,
    }
}

impl Lexer<'_> {
    /// `${...}` after its `${`.
    pub(super) fn brace(&mut self, quoting: Quoting) {
        if quoting != Quoting::Bare && defaulted(&self.s[self.at..]) {
            return self.operand(quoting);
        }
        while let Some(byte) = self.byte(self.at) {
            if byte == b'}' {
                self.at += 1;
                return;
            }
            if !self.nested(Quoting::Bare) {
                self.at += 1;
            }
        }
    }

    /// A quoted `${x:-word}`; Bash drops its double quotes first.
    fn operand(&mut self, quoting: Quoting) {
        let mut group = None;
        while let Some(byte) = self.byte(self.at) {
            let next = self.byte(self.at + 1);
            match byte {
                b'}' if group.is_none() => {
                    self.at += 1;
                    return;
                }
                b'"' | b'\'' => {
                    group = grouped(group, byte);
                    self.at += 1;
                }
                // Double quotes translate `$'` and `$"` first.
                b'$' if quoting == Quoting::Double
                    && group.is_none()
                    && matches!(next, Some(b'\'' | b'"')) =>
                {
                    self.at += 1;
                }
                b'$' => {
                    self.at += 1;
                    // Dropped quotes join the `$` to what follows.
                    while self.byte(self.at) == Some(b'"') {
                        group = grouped(group, b'"');
                        self.at += 1;
                    }
                    match self.byte(self.at) {
                        Some(b'$') => self.at += 1,
                        _ => self.dollar(quoting),
                    }
                }
                _ if self.nested(quoting) => {}
                _ => self.at += 1,
            }
        }
    }
}
