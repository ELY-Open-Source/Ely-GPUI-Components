use std::{collections::HashMap, sync::OnceLock};

use html5ever::{Attribute, LocalName, local_name};

use super::{Style, dom::Dom};

/// One HTML tag as written: its lowercase name.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tag {
    pub name: String,
    pub closing: bool,
}

/// How Typora shows an inline tag; others show text.
pub(crate) fn inline_style(name: &str) -> Option<Style> {
    match name {
        "u" => Some(Style::Underline),
        "b" | "strong" => Some(Style::Strong),
        "i" | "em" => Some(Style::Emphasis),
        "s" | "del" | "strike" => Some(Style::Strike),
        "code" | "kbd" => Some(Style::Code),
        "mark" => Some(Style::Mark),
        "a" => Some(Style::Link),
        _ => None,
    }
}

/// HTML5's formatting elements reopen past a crossed close.
pub(crate) fn reopens(name: &str) -> bool {
    matches!(
        name,
        "a" | "b" | "code" | "em" | "i" | "s" | "strike" | "strong" | "u"
    )
}

/// GFM's tag filter: these tags show as text.
pub(crate) fn filtered(name: &str) -> bool {
    matches!(
        name,
        "title"
            | "textarea"
            | "style"
            | "xmp"
            | "iframe"
            | "noembed"
            | "noframes"
            | "script"
            | "plaintext"
    )
}

/// Elements a browser lays as blocks: a line each side.
pub(crate) fn breaks(name: &str) -> bool {
    matches!(
        name,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "center"
            | "caption"
            | "details"
            | "dialog"
            | "dir"
            | "div"
            | "dl"
            | "dd"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hgroup"
            | "hr"
            | "legend"
            | "li"
            | "listing"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "optgroup"
            | "option"
            | "p"
            | "pre"
            | "search"
            | "section"
            | "summary"
            | "table"
            | "tr"
            | "ul"
    )
}

/// Replaced elements: a box, their content only a fallback.
pub(crate) fn replaced(name: &str) -> bool {
    matches!(
        name,
        "audio" | "canvas" | "meter" | "object" | "progress" | "video"
    )
}

/// Whether an element may hide or replace its content.
pub(crate) fn veils(name: &LocalName, attributes: &[Attribute]) -> bool {
    [
        local_name!("audio"),
        local_name!("canvas"),
        local_name!("datalist"),
        local_name!("details"),
        local_name!("dialog"),
        local_name!("meter"),
        local_name!("noscript"),
        local_name!("object"),
        local_name!("progress"),
        local_name!("rp"),
        local_name!("template"),
        local_name!("video"),
    ]
    .contains(name)
        || attributes
            .iter()
            .any(|it| it.name.local == local_name!("hidden"))
}

/// Whether a browser draws `node` at all.
pub(crate) fn shown(dom: &Dom, node: usize) -> bool {
    let name = &dom.names[node].local;
    let has = |attribute| dom.has(node, &attribute);
    if has(local_name!("hidden")) {
        return false;
    }
    if [
        local_name!("datalist"),
        local_name!("noscript"),
        local_name!("rp"),
        local_name!("template"),
    ]
    .contains(name)
    {
        return false;
    } else if *name == local_name!("dialog") {
        return has(local_name!("open"));
    } else if *name == local_name!("audio") {
        return has(local_name!("controls"));
    }
    // A closed `details` shows its first `summary` alone.
    let Some(parent) = dom.nodes[node].parent else {
        return true;
    };
    if dom.names[parent].local != local_name!("details") || dom.has(parent, &local_name!("open")) {
        return true;
    }
    let children = &dom.nodes[parent].children;
    let summary = local_name!("summary");
    children
        .iter()
        .find(|&&child| dom.names[child].local == summary)
        == Some(&node)
}

/// Bytes through the `>` ending the tag at `html`'s start.
pub(crate) fn tag_end(html: &str) -> Option<usize> {
    let (mut quote, mut last) = (None, '<');
    for (at, ch) in html.char_indices().skip(1) {
        match quote {
            Some(open) if ch == open => quote = None,
            Some(_) => {}
            None if ch == '>' => return Some(at + 1),
            None if matches!(ch, '"' | '\'') && last == '=' => quote = Some(ch),
            None => {}
        }
        if !spacing(ch) {
            last = ch;
        }
    }
    None
}

/// `html` as one tag; comments and the rest are none.
pub(crate) fn tag(html: &str) -> Option<Tag> {
    let inner = html.trim().strip_prefix('<')?.strip_suffix('>')?;
    let (closing, inner) = match inner.strip_prefix('/') {
        Some(rest) => (true, rest),
        None => (false, inner),
    };
    if !inner.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
        return None;
    }
    let named = inner
        .find(|ch: char| spacing(ch) || ch == '/')
        .unwrap_or(inner.len());
    Some(Tag {
        name: inner[..named].to_ascii_lowercase(),
        closing,
    })
}

/// HTML's whitespace, which is ASCII alone.
pub(crate) fn spacing(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r' | '\x0C')
}

/// Longest reference name HTML knows, in bytes.
const LONGEST: usize = 32;

/// A reference at `text`'s start, as HTML reads it.
pub(super) fn reference(text: &str, attribute: bool) -> Option<(String, usize)> {
    let body = text.strip_prefix('&')?;
    if let Some(number) = body.strip_prefix('#') {
        let (digits, radix) = match number.strip_prefix(['x', 'X']) {
            Some(hex) => (hex, 16),
            None => (number, 10),
        };
        let len = digits
            .find(|ch: char| !ch.is_digit(radix))
            .unwrap_or(digits.len());
        if len == 0 {
            return None;
        }
        let code = digits[..len].chars().try_fold(0u32, |code, ch| {
            code.checked_mul(radix)?.checked_add(ch.to_digit(radix)?)
        });
        let ended = usize::from(digits[len..].starts_with(';'));
        let used = 2 + (number.len() - digits.len()) + len + ended;
        return Some((numeric(code).to_string(), used));
    }
    let len = body
        .find(|ch: char| !ch.is_ascii_alphanumeric())
        .unwrap_or(body.len());
    let name = &body[..len];
    if (1..=LONGEST).contains(&len)
        && body[len..].starts_with(';')
        && let Some(value) = named(name, true)
    {
        return Some((value.to_string(), len + 2));
    }
    // Text takes a legacy prefix; a value, the whole name.
    let shortest = match attribute {
        true if body[len..].starts_with(['=', ';']) => return None,
        true => len,
        false => 1,
    };
    let name = (shortest..=len.min(LONGEST))
        .rev()
        .map(|end| &name[..end])
        .find(|name| legacy(name))?;
    let value = named(name, false).expect("a legacy name");
    Some((value.to_string(), name.len() + 1))
}

/// A name HTML decodes without its semicolon.
fn legacy(name: &str) -> bool {
    named(name, false).is_some()
}

/// A named reference's text, with or without its `;`.
pub(super) fn named(name: &str, semicolon: bool) -> Option<&'static str> {
    static NAMES: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let names = NAMES.get_or_init(|| {
        (entities::ENTITIES.iter())
            .map(|entity| (&entity.entity[1..], entity.characters))
            .collect()
    });
    match semicolon {
        true => names.get(format!("{name};").as_str()).copied(),
        false => names.get(name).copied(),
    }
}

/// The character a numeric reference names, as HTML maps it.
fn numeric(code: Option<u32>) -> char {
    // Windows-1252 for the C1 block, as browsers do.
    const C1: [u32; 32] = [
        0x20AC, 0x81, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160,
        0x2039, 0x0152, 0x8D, 0x017D, 0x8F, 0x90, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013,
        0x2014, 0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0x9D, 0x017E, 0x0178,
    ];
    match code {
        Some(code @ 0x80..=0x9F) => char::from_u32(C1[(code - 0x80) as usize]),
        Some(0) | None => None,
        Some(code) => char::from_u32(code),
    }
    .unwrap_or(char::REPLACEMENT_CHARACTER)
}

/// `n` in superscript digits.
pub(crate) fn superscript(n: usize) -> String {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    n.to_string()
        .bytes()
        .map(|digit| DIGITS[usize::from(digit - b'0')])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tag_reads_its_name() {
        let link = tag("<A HREF=\"https://ely.dev\" title='x' hidden>").expect("a tag");
        assert_eq!((link.name.as_str(), link.closing), ("a", false));
        assert_eq!(
            tag("</u>").map(|t| (t.name, t.closing)),
            Some(("u".into(), true))
        );
        assert_eq!(tag("<br/>").map(|t| t.name), Some("br".into()));
        assert_eq!(tag("<u-box>").map(|t| t.name), Some("u-box".into()));
        assert_eq!(tag("<!-- note -->"), None);
        assert_eq!(tag("<1>"), None);
    }

    #[test]
    fn a_tag_ends_outside_its_quotes() {
        assert_eq!(tag_end("<a href=\"x>y\">z"), Some(14));
        assert_eq!(tag_end("<b it's>r"), Some(8));
        assert_eq!(tag_end("<b"), None);
    }

    #[test]
    fn numbers_go_up() {
        assert_eq!(superscript(1), "¹");
        assert_eq!(superscript(120), "¹²⁰");
    }
}
