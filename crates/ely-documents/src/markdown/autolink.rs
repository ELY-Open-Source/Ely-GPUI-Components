//! GitHub's literal autolinks: `www.`, `http(s)://`, and email addresses.

use std::ops::Range;

/// Literal links inside `stretch` of `source`, with their addresses.
pub(super) fn find(source: &str, stretch: Range<usize>) -> Vec<(Range<usize>, String)> {
    let (mut out, mut at, mut quiet) = (Vec::new(), stretch.start, stretch.start);
    while at < stretch.end {
        let found = www(source, at, stretch.end, &mut quiet)
            .or_else(|| url(source, at, stretch.end))
            .or_else(|| email(source, at, stretch.end));
        match found {
            Some((range, address)) => {
                at = range.end;
                out.push((range, address));
            }
            None => at += source[at..].chars().next().map_or(1, char::len_utf8),
        }
    }
    out
}

fn before(source: &str, at: usize) -> Option<char> {
    source[..at].chars().next_back()
}

/// `name` at `at`, in any case.
fn starts(source: &str, at: usize, end: usize, name: &str) -> bool {
    (source.get(at..end))
        .and_then(|rest| rest.get(..name.len()))
        .is_some_and(|head| head.eq_ignore_ascii_case(name))
}

/// `quiet`: before it, a `www.` shares a domain already refused.
fn www(source: &str, at: usize, end: usize, quiet: &mut usize) -> Option<(Range<usize>, String)> {
    let opens = before(source, at)
        .is_none_or(|ch| matches!(ch, '\t' | '\n' | ' ' | '(' | '*' | '_' | '[' | ']' | '~'));
    if at < *quiet || !starts(source, at, end, "www.") || !opens {
        return None;
    }
    let stop = match domain(source, at + 4, end) {
        Ok(stop) => stop,
        Err(shared) => {
            *quiet = shared.saturating_sub(3);
            return None;
        }
    };
    let range = at..linked(source, at, stop, end);
    let address = format!("http://{}", &source[range.clone()]);
    Some((range, address))
}

fn url(source: &str, at: usize, end: usize) -> Option<(Range<usize>, String)> {
    let scheme = ["https://", "http://"]
        .into_iter()
        .find(|scheme| starts(source, at, end, scheme))?;
    if before(source, at).is_some_and(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    let range = at..linked(
        source,
        at,
        domain(source, at + scheme.len(), end).ok()?,
        end,
    );
    Some((range.clone(), source[range].to_string()))
}

/// The end of a domain from `at`, or where a later start shares its refusal.
fn domain(source: &str, at: usize, end: usize) -> Result<usize, usize> {
    let stop = source[at..end]
        .find(|ch: char| !(ch.is_alphanumeric() || matches!(ch, '-' | '_' | '.')))
        .map_or(end, |ix| at + ix);
    let name = source[at..stop].trim_end_matches(['.', '_']);
    let last = name.rfind('.');
    let second = last.and_then(|ix| name[..ix].rfind('.'));
    // Starts up to the second-to-last part share the verdict.
    let shared = at + second.map_or(0, |ix| ix + 1);
    // An underscore in the last two parts is no domain.
    let clean = !name[shared - at..].contains('_');
    match clean && name.contains(char::is_alphanumeric) {
        true => Ok(stop),
        false => Err(shared),
    }
}

/// The link's end: its path to white space or `<`, less what trails.
fn linked(source: &str, start: usize, from: usize, end: usize) -> usize {
    let path = source[from..end]
        .find(|ch: char| ch.is_whitespace() || ch == '<')
        .map_or(end, |ix| from + ix);
    let mut stop = path;
    // Only `)` leaves while trimming; `(` stays counted.
    let opened = source[start..path].matches('(').count();
    let mut closed = source[start..path].matches(')').count();
    loop {
        let link = &source[start..stop];
        let Some(last) = link.chars().next_back() else {
            return stop;
        };
        let trails = matches!(
            last,
            '!' | '"' | '\'' | '*' | ',' | '.' | ':' | '?' | '_' | '~' | ']'
        );
        if trails {
            stop -= 1;
        } else if last == ')' && closed > opened {
            stop -= 1;
            closed -= 1;
        } else if last == ';' {
            stop -= entity(link).unwrap_or(1);
        } else {
            return stop;
        }
    }
}

/// The length of an `&name;` ending `link`.
fn entity(link: &str) -> Option<usize> {
    let body = &link[..link.len() - 1];
    let name = body.len()
        - body
            .trim_end_matches(|ch: char| ch.is_ascii_alphabetic())
            .len();
    (name > 0 && body[..body.len() - name].ends_with('&')).then_some(name + 2)
}

fn email(source: &str, at: usize, end: usize) -> Option<(Range<usize>, String)> {
    let bytes = source.as_bytes();
    let protocol = ["mailto:", "xmpp:"]
        .into_iter()
        .find(|name| starts(source, at, end, name));
    let local =
        |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'+' | b'_' | b'-');
    // An address starts a word, not an escape; each word is read once.
    let word =
        before(source, at).is_some_and(|ch| ch == '\\' || (ch.is_ascii() && local(ch as u8)));
    if word || (protocol.is_some() && before(source, at).is_some_and(|ch| ch.is_ascii_alphabetic()))
    {
        return None;
    }
    let local_from = at + protocol.map_or(0, str::len);
    let mut stop = local_from;
    while stop < end && local(bytes[stop]) {
        stop += 1;
    }
    if stop == local_from || bytes.get(stop) != Some(&b'@') {
        return None;
    }
    let mut host = stop + 1;
    let mut dots = 0;
    while host < end
        && (bytes[host].is_ascii_alphanumeric() || matches!(bytes[host], b'-' | b'_' | b'.'))
    {
        let next = bytes.get(host + 1).filter(|_| host + 1 < end);
        dots += usize::from(bytes[host] == b'.' && next.is_some_and(u8::is_ascii_alphanumeric));
        host += 1;
    }
    while host > stop + 1 && bytes[host - 1] == b'.' {
        host -= 1;
    }
    if dots == 0 || matches!(bytes[host - 1], b'-' | b'_') {
        return None;
    }
    if protocol == Some("xmpp:") && bytes.get(host) == Some(&b'/') {
        host += 1;
        while host < end
            && (bytes[host].is_ascii_alphanumeric() || matches!(bytes[host], b'@' | b'.'))
        {
            host += 1;
        }
    }
    let range = at..host;
    let written = &source[range.clone()];
    let address = match protocol {
        Some(_) => written.to_string(),
        None => format!("mailto:{written}"),
    };
    Some((range, address))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn links(text: &str) -> Vec<(&str, String)> {
        find(text, 0..text.len())
            .into_iter()
            .map(|(range, address)| (&text[range], address))
            .collect()
    }

    #[test]
    fn literal_links_lose_what_trails() {
        assert_eq!(
            links("see www.x.dev, https://y.dev/a_(b)). and me@z.io."),
            [
                ("www.x.dev", "http://www.x.dev".to_string()),
                ("https://y.dev/a_(b)", "https://y.dev/a_(b)".to_string()),
                ("me@z.io", "mailto:me@z.io".to_string()),
            ]
        );
        assert_eq!(links("awww.x.dev x@y www._a.b"), []);
        assert_eq!(
            links("www.a.b/c&amp; 'www.d.e'"),
            [("www.a.b/c", "http://www.a.b/c".to_string())]
        );
        assert_eq!(
            links("<https://a.b c"),
            [("https://a.b", "https://a.b".to_string())]
        );
    }
}
