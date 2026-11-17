/// `text` with each known `:shortcode:` as its emoji.
fn emoji(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(':') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let len = after
            .find(|ch: char| !shortcode_char(ch))
            .unwrap_or(after.len());
        let found = after[len..]
            .starts_with(':')
            .then(|| emojis::get_by_shortcode(&after[..len]))
            .flatten();
        match found {
            Some(found) => {
                out.push_str(found.as_str());
                rest = &after[len + 1..];
            }
            None => {
                out.push(':');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

fn shortcode_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '+' | '-')
}

/// Where the first `:name:` starts.
fn candidate(text: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(at) = text[from..].find(':').map(|at| from + at) {
        let after = &text[at + 1..];
        let len = after
            .find(|ch: char| !shortcode_char(ch))
            .unwrap_or(after.len());
        if len > 0 && after[len..].starts_with(':') {
            return Some(at);
        }
        from = at + 1 + len;
    }
    None
}

/// One DOM text node's shortcodes, as GitHub reads them.
#[derive(Default)]
pub struct Shortcodes {
    /// Settled by the node's first `:name:`.
    reads: Option<bool>,
    /// Text before it held more than white space.
    inked: bool,
    last: Option<char>,
}

impl Shortcodes {
    /// A new DOM text node starts.
    pub fn split(&mut self) {
        *self = Self::default();
    }

    /// Blank or spaced text before the first reads them all.
    pub fn read(&mut self, text: &str) -> String {
        if self.reads.is_none() {
            let at = candidate(text);
            let lead = &text[..at.unwrap_or(text.len())];
            self.inked |= !lead.chars().all(|ch| ch.is_ascii_whitespace());
            self.last = lead.chars().next_back().or(self.last);
            self.reads = at.map(|_| !self.inked || self.last == Some(' '));
        }
        match self.reads {
            Some(true) => emoji(text),
            _ => text.to_string(),
        }
    }
}

/// Shortcodes for an owner writing its own HTML.
///
/// One value per block: `split` at every element, `text` on text,
/// `html` on inline HTML. Text inside `code` or `pre` stays as written.
#[derive(Default)]
pub struct ShortcodeRun {
    shortcodes: Shortcodes,
    /// How many `code` are open.
    code: usize,
    /// How many `pre` are open.
    pre: usize,
}

impl ShortcodeRun {
    /// A new DOM text node starts.
    pub fn split(&mut self) {
        self.shortcodes.split();
    }

    /// Text as shown; an open `code` or `pre` keeps it.
    pub fn text(&mut self, written: &str) -> String {
        match self.code + self.pre {
            0 => self.shortcodes.read(written),
            _ => written.to_string(),
        }
    }

    /// Inline HTML as written; a filtered tag shows as text.
    pub fn html(&mut self, written: &str) -> String {
        self.shortcodes.split();
        let Some(tag) = super::html::tag(written) else {
            return written.to_string();
        };
        if super::html::filtered(&tag.name) {
            return self.text(written);
        }
        let open = match tag.name.as_str() {
            "code" => &mut self.code,
            "pre" => &mut self.pre,
            _ => return written.to_string(),
        };
        *open = match tag.closing {
            true => open.saturating_sub(1),
            false => *open + 1,
        };
        written.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_shortcodes_become_emoji() {
        assert_eq!(emoji("hi :smile: :+1:"), "hi 😄 👍");
        assert_eq!(emoji(":nope: at 10:30 and ::"), ":nope: at 10:30 and ::");
        assert_eq!(emoji("a:smile:b :x:smile:"), "a😄b ❌smile:");
        assert_eq!(emoji("é :smile: 中"), "é 😄 中");
    }

    #[test]
    fn long_runs_scan_in_linear_time() {
        let colons = ":zq".repeat(1 << 16);
        let started = web_time::Instant::now();
        assert_eq!(emoji(&colons).len(), colons.len());
        assert_eq!(Shortcodes::default().read(&colons).len(), colons.len());
        let spent = started.elapsed();
        assert!(spent.as_secs() < 2, "192 KiB took {spent:?}");
    }

    #[test]
    fn a_close_ends_only_its_own_kind() {
        let mut run = ShortcodeRun::default();
        let mut read = |parts: &[&str]| -> String {
            parts
                .iter()
                .map(|part| match part.starts_with('<') {
                    true => run.html(part),
                    false => run.text(part),
                })
                .collect()
        };
        let crossed = read(&[
            "a ", "<pre>", "x", "</code>", " :smile:", "</pre>", " :smile:",
        ]);
        assert_eq!(crossed, "a <pre>x</code> :smile:</pre> 😄");
        let unmatched = read(&["<code>", "</pre>", " :smile:", "</code>", " :smile:"]);
        assert_eq!(unmatched, "<code></pre> :smile:</code> 😄");
    }
}
