use super::{Inline, Opened, Run, Style, html, html::InlineTag};

/// The elements open around inline text.
impl Run {
    pub(super) fn push(&mut self, opened: Opened) {
        *self.named.entry(opened.name.clone()).or_default() += 1;
        self.open.push(opened);
    }

    /// The innermost open link's address.
    pub(super) fn link(&self) -> Option<String> {
        self.open.iter().rev().find_map(|opened| opened.url.clone())
    }

    /// Opens or closes an element for an inline HTML tag.
    pub(super) fn html(&mut self, written: &str) {
        match html::inline_tag(written) {
            InlineTag::Text(shown) => self.push_text(&shown),
            InlineTag::Open { name, style, href } => self.start(&name, style, href),
            InlineTag::Close(name) => {
                self.shortcodes.split();
                self.end(&name);
            }
            InlineTag::Break => {
                self.shortcodes.split();
                self.text.text.push('\n');
            }
            InlineTag::Gone => self.shortcodes.split(),
        }
    }

    /// Opens `name`; a new `a` ends the open one.
    pub(super) fn start(&mut self, name: &str, style: Option<Style>, url: Option<String>) {
        if name == "a" {
            self.end("a");
        }
        self.shortcodes.split();
        self.push(Opened {
            name: name.to_string(),
            start: self.text.text.len(),
            style,
            url,
        });
    }

    /// Ends the innermost `name`, as HTML5 repairs nesting.
    pub(super) fn end(&mut self, name: &str) {
        if self.count(name) == 0 {
            return;
        }
        self.shortcodes.split();
        let at = self
            .open
            .iter()
            .rposition(|opened| opened.name == name)
            .expect("a counted name is open");
        let end = self.text.text.len();
        if let Some((_, from)) = &mut self.owed {
            *from = (*from).min(at);
        }
        // Text written since rebuilt what waited.
        let open = &self.open;
        self.pending.retain(|&ix| ix < at && open[ix].start == end);
        let mut rest = self.open.split_off(at);
        if !html::reopens(name) {
            let skip = if html::scope(name) { rest.len() } else { 1 };
            return self.rebuild(rest, skip, end);
        }
        let ended = rest.remove(0);
        self.finish(&ended, end);
        // HTML5's adoption agency: blocks stay, at most eight.
        let (mut segment, mut blocks, mut rest) = (Vec::new(), 0, rest.into_iter());
        while let Some(opened) = rest.next() {
            if html::inline_style(&opened.name).is_some() {
                segment.push(opened);
                continue;
            }
            self.wrap(std::mem::take(&mut segment), end);
            self.open.push(opened);
            blocks += 1;
            if blocks == 8 {
                return self.open.extend(rest);
            }
        }
        self.rebuild(segment, 0, end);
    }

    /// The three nearest formatting elements of `segment` wrap the block.
    fn wrap(&mut self, segment: Vec<Opened>, end: usize) {
        let wraps = segment.len().saturating_sub(3);
        for (ix, mut opened) in segment.into_iter().enumerate() {
            if ix < wraps || !html::reopens(&opened.name) {
                self.finish(&opened, end);
                continue;
            }
            close(&mut self.text, &opened, end);
            opened.start = end;
            self.open.push(opened);
        }
    }

    /// Closes `closed` at `end`; past `skip`, three alike reopen.
    fn rebuild(&mut self, closed: Vec<Opened>, skip: usize, end: usize) {
        for opened in &closed {
            self.finish(opened, end);
        }
        let keep = rebuilt(&closed[skip.min(closed.len())..]);
        for (opened, keep) in closed.into_iter().skip(skip).zip(keep) {
            if keep {
                self.pending.push(self.open.len());
                self.push(Opened {
                    start: end,
                    ..opened
                });
            }
        }
    }

    /// Records `opened` up to `end`; it is no longer open.
    pub(super) fn finish(&mut self, opened: &Opened, end: usize) {
        close(&mut self.text, opened, end);
        *self.named.get_mut(&opened.name).expect("counted") -= 1;
    }
}

/// Which of `closed` reopen.
///
/// HTML5 rebuilds at most the three latest alike (Noah's Ark).
fn rebuilt(closed: &[Opened]) -> Vec<bool> {
    let mut keep = vec![false; closed.len()];
    let mut alike: Vec<(usize, usize)> = Vec::new();
    for ix in (0..closed.len()).rev() {
        let opened = &closed[ix];
        if !html::reopens(&opened.name) {
            continue;
        }
        let same = |other: &Opened| {
            (&other.name, other.style, &other.url) == (&opened.name, opened.style, &opened.url)
        };
        match alike.iter_mut().find(|(at, _)| same(&closed[*at])) {
            Some((_, seen)) if *seen == 3 => {}
            Some((_, seen)) => {
                *seen += 1;
                keep[ix] = true;
            }
            None => {
                alike.push((ix, 1));
                keep[ix] = true;
            }
        }
    }
    keep
}

/// Records `opened` as styled up to `end`.
pub(super) fn close(text: &mut Inline, opened: &Opened, end: usize) {
    if opened.start >= end {
        return;
    }
    if let Some(style) = opened.style {
        text.styles.push((opened.start..end, style));
    }
    if let Some(url) = &opened.url {
        text.links.push((opened.start..end, url.clone()));
    }
}

/// Inline text's shortcodes as the reader reads them.
///
/// For an owner writing its own HTML from `markdown_tree`: one value
/// per block, `split` at every element, `text` on text and `html` on
/// inline HTML, whose tags open and close elements here too.
#[derive(Default)]
pub struct ShortcodeRun(Run);

impl ShortcodeRun {
    /// A new DOM text node starts.
    pub fn split(&mut self) {
        self.0.shortcodes.split();
    }

    /// Text as shown; an open `code` or `pre` keeps it.
    pub fn text(&mut self, written: &str) -> String {
        self.0.shown(written)
    }

    /// Inline HTML as text: a tag as written, the rest as shown.
    pub fn html(&mut self, written: &str) -> String {
        if matches!(html::inline_tag(written), InlineTag::Text(_)) {
            return self.0.shown(written);
        }
        self.0.html(written);
        // Nothing is laid out from this run.
        self.0.text = Inline::default();
        written.to_string()
    }
}
