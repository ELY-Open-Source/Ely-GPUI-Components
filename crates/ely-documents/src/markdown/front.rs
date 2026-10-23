//! Front matter: `---` or `+++` fences at a file's start, empty or not.

use super::mdast::{Literal, Node};

/// The front matter's node, and `source` with it blanked to spaces.
pub(super) fn split(source: &str) -> Option<(Node, String)> {
    let marker = match source.get(..3)? {
        "---" => b'-',
        "+++" => b'+',
        _ => return None,
    };
    let fence = |line: &str| {
        let bytes = line.as_bytes();
        bytes.len() >= 3
            && bytes[..3].iter().all(|byte| *byte == marker)
            && bytes[3..].iter().all(|byte| matches!(byte, b' ' | b'\t'))
    };
    let mut lines = lines(source);
    let (_, first_end, body) = lines.next()?;
    if !fence(&source[..first_end]) {
        return None;
    }
    let (close, end, _) = lines.find(|(start, end, _)| fence(&source[*start..*end]))?;
    let value = source[body..close]
        .strip_suffix('\n')
        .unwrap_or(&source[body..close]);
    let value = value.strip_suffix('\r').unwrap_or(value);
    let literal = Literal {
        value: value.to_string(),
        position: 0..end,
    };
    let node = match marker {
        b'-' => Node::Yaml(literal),
        _ => Node::Toml(literal),
    };
    let mut blanked = source.as_bytes().to_vec();
    for byte in &mut blanked[..end] {
        if !matches!(byte, b'\n' | b'\r') {
            *byte = b' ';
        }
    }
    let blanked = String::from_utf8(blanked).expect("spaces and line ends are UTF-8");
    Some((node, blanked))
}

/// Each line's start, its end before the line ending, and the next line's start.
fn lines(text: &str) -> impl Iterator<Item = (usize, usize, usize)> + '_ {
    let mut at = Some(0);
    std::iter::from_fn(move || {
        let start = at?;
        let Some(ix) = text[start..].find(['\n', '\r']) else {
            at = None;
            return Some((start, text.len(), text.len()));
        };
        let end = start + ix;
        let next = end + 1 + usize::from(text[end..].starts_with("\r\n"));
        at = (next < text.len()).then_some(next);
        Some((start, end, next))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(source: &str) -> Option<(String, std::ops::Range<usize>)> {
        split(source).map(|(node, _)| match node {
            Node::Yaml(front) | Node::Toml(front) => (front.value, front.position),
            other => panic!("front matter, not {other:?}"),
        })
    }

    #[test]
    fn fences_open_and_close_at_the_start() {
        assert_eq!(value("---\nt: 1\n---\n\na"), Some(("t: 1".into(), 0..12)));
        assert_eq!(value("---\n---"), Some((String::new(), 0..7)));
        assert_eq!(value("+++ \r\nx\r\n+++\r\n"), Some(("x".into(), 0..12)));
        assert_eq!(value("----\n---"), None);
        assert_eq!(value("---\nx"), None);
        assert_eq!(value("a\n---\n---"), None);
    }

    #[test]
    fn the_blanked_text_keeps_its_lines() {
        let (_, blanked) = split("---\nt: é\n---\na").expect("front matter");
        assert_eq!(blanked, "   \n     \n   \na");
    }
}
