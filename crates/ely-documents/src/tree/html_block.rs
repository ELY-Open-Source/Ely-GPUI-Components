use std::ops::Range;

use super::{
    Escapes, Inline,
    drawn::{Drawn, Mode, draw},
    html,
    sources::LineMap,
    written::{self, Written},
};

/// HTML text with reserved characters escaped, references too.
fn guarded(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(ch) = rest.chars().next() {
        let decoded = (ch == '&').then(|| html::reference(rest, false)).flatten();
        match decoded {
            Some((value, used)) if value.chars().any(written::reserved) => {
                for ch in value.chars() {
                    out.push_str(&written::escaped(&ch.to_string()));
                }
                rest = &rest[used..];
            }
            _ => {
                match written::reserved(ch) {
                    true => out.push_str(&written::escaped(&ch.to_string())),
                    false => out.push(ch),
                }
                rest = &rest[ch.len_utf8()..];
            }
        }
    }
    out
}

/// An HTML block's text, as a browser shows it.
pub(super) fn html_block_text(written: &str, source: &str, span: Range<usize>) -> Inline {
    let (mut out, map) = (
        Written::default(),
        LineMap::new(source, span.start, written),
    );
    let mut rest = written;
    // Each text run keeps where it was written.
    let words = |out: &mut Written, rest: &str, text: &str| {
        let from = written.len() - rest.len();
        let at = |value| map.as_ref()?.at(value);
        // Blank text draws nothing where it stands.
        if !text.chars().all(html::spacing) {
            match at(from).zip(at(from + text.len())) {
                Some((start, end)) => out.chunk(start..end, Escapes::Html),
                None => log::warn!("markdown: an HTML block's text lost its source"),
            }
        }
        out.html.push_str(&guarded(text));
    };
    while let Some(at) = rest.find('<') {
        words(&mut out, rest, &rest[..at]);
        rest = &rest[at..];
        let opens =
            rest[1..].starts_with(|ch: char| ch.is_ascii_alphabetic() || "/!?".contains(ch));
        if !opens {
            words(&mut out, rest, "<");
            rest = &rest[1..];
            continue;
        }
        // Markup goes to the parser as written.
        let end = match rest.strip_prefix("<!--") {
            Some(comment) => comment.find("-->").map(|close| close + 7),
            None => html::tag_end(rest),
        };
        let (markup, after) = rest.split_at(end.unwrap_or(rest.len()));
        out.raw(markup);
        rest = after;
    }
    words(&mut out, rest, rest);
    let mut text = Inline::default();
    for piece in draw(&out, ("", ""), Mode::Block, false, false) {
        if let Drawn::Text(drawn) = piece {
            text = drawn;
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use crate::tree::{Inline, Node, Style, parse};

    fn html(source: &str) -> Option<Inline> {
        match parse(source).into_iter().next() {
            Some(Node::Paragraph(line)) => Some(line),
            None => None,
            other => panic!("a paragraph, not {other:?}"),
        }
    }

    #[test]
    fn an_html_block_shows_its_text_as_a_browser_does() {
        let centered = html(
            "<div align=\"center\">\n  <b>Bold</b>   and\n  :smile: &amp; <code>:smile:</code>\n</div>",
        )
        .expect("text");
        assert_eq!(centered.text, "Bold and 😄 & :smile:");
        let (bold, code) = (0..4, 16..23);
        assert_eq!(
            centered.styles,
            [(bold, Style::Strong), (code, Style::Code)]
        );
        let parts = html("<p>one <b>two </b></p>\n<p>three &lt;b&gt;</p>").expect("text");
        assert_eq!(parts.text, "one two\n\nthree <b>");
        assert_eq!(
            html("<div>\na <br> b</br>c</div>").expect("text").text,
            "a\nb\nc"
        );
        let filtered = "<div>\n<script>x</script>\n<!-- note -->\nshown<style>p{}</style>\n</div>";
        assert_eq!(
            html(filtered).expect("text").text,
            "<script>x</script> shown<style>p{}</style>"
        );
        let crossed = "<div>before<script>a</scriptx>b</script>after</div>";
        assert_eq!(
            html(crossed).expect("text").text,
            "before<script>ab</script>after"
        );
        let split = parse("<div>\n<script>\nfirst\n\nsecond\n</script>\n</div>");
        let lines: Vec<_> = (split.into_iter())
            .map(|node| match node {
                Node::Paragraph(line) => line.text,
                other => panic!("a paragraph, not {other:?}"),
            })
            .collect();
        assert_eq!(lines, ["<script> first", "second\n</script>"]);
        let inline = html("a <script>b</script> c").expect("text");
        assert_eq!(inline.text, "a <script>b</script> c");
        assert_eq!(html("<div>\na < b <i").expect("text").text, "a < b");
    }

    #[test]
    fn a_closing_block_ends_what_opened_inside_it() {
        let marked = html("<div><mark>x</div>y").expect("text");
        assert_eq!(
            (marked.text.as_str(), &marked.styles[..]),
            ("x\ny", &[(0..1, Style::Mark)][..])
        );
        let bold = html("<div><b>x</div>y").expect("text");
        assert_eq!(bold.styles, [(0..1, Style::Strong), (2..3, Style::Strong)]);
        let kept = html("<div><mark><p>x</p>y</mark></div>").expect("text");
        assert_eq!(kept.styles, [(0..4, Style::Mark)]);
    }

    #[test]
    fn a_block_start_ends_what_html5_ends() {
        let mark = [(0..1, Style::Mark)];
        // A `p` ends with a blank line, as innerText.
        for (source, shown) in [
            ("<ul><li><mark>x<li>y</ul>z", "x\ny\nz"),
            ("<div><p><mark>x<div>y</div>z</div>", "x\n\ny\nz"),
            ("<dl><dt><mark>x<dd>y</dl>z", "x\ny\nz"),
        ] {
            let line = html(source).expect("text");
            assert_eq!((line.text.as_str(), &line.styles[..]), (shown, &mark[..]));
        }
        let kbd = html("<div><p><kbd>x<p>y</div>z").expect("text");
        assert_eq!(kbd.styles, [(0..1, Style::Code)]);
        let nested = html("<ul><li><mark>a<ul><li>b</li></ul></li></ul>").expect("text");
        assert_eq!(
            nested.styles,
            [(0..3, Style::Mark)],
            "a list stops the search"
        );
    }

    #[test]
    fn filtered_tags_are_text_and_names_end_as_html5_ends_them() {
        // GitHub's answers, `.work/R4.3-fix3/github-*.log`.
        for (source, shown) in [
            (
                "<div><script>:smile:</script> :smile:</div>",
                "<script>:smile:</script> :smile:",
            ),
            (
                "<div><script><b>:smile:</b></script> :smile:</div>",
                "<script>😄</script> 😄",
            ),
            (
                "<div><script> :smile:</script> :smile:</div>",
                "<script> 😄</script> 😄",
            ),
            (
                "<div><script title=\" :smile:\"> x:smile:</div>",
                "<script title=\" 😄\"> x😄",
            ),
            (
                "<div><iframe title=\":smile:\"> :smile:</div>",
                "<iframe title=\":smile:\"> :smile:",
            ),
            ("<div>\n:smile: x:smile:\n</div>", "😄 x😄"),
            ("<div>\t:smile: x:smile:</div>", "😄 x😄"),
            ("<div>a\n :smile: x:smile:</div>", "a 😄 x😄"),
            ("<div>y\n:smile: x:smile:</div>", "y :smile: x:smile:"),
            ("<div>:smile:</div>", "😄"),
        ] {
            assert_eq!(html(source).expect("text").text, shown, "{source:?}");
        }
        let named = "<div>a<script:custom>x</script:custom>b<script!>y</script!>c</div>";
        assert_eq!(html(named).expect("text").text, "axbyc");
    }

    #[test]
    fn every_p_closer_ends_a_p_as_github_does() {
        // Start tags ending a `p`, as GitHub and browsers read.
        const P_CLOSERS: [&str; 39] = [
            "address",
            "article",
            "aside",
            "blockquote",
            "center",
            "details",
            "dialog",
            "dir",
            "div",
            "dl",
            "dd",
            "dt",
            "fieldset",
            "figcaption",
            "figure",
            "footer",
            "form",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "header",
            "hgroup",
            "hr",
            "li",
            "listing",
            "main",
            "menu",
            "nav",
            "ol",
            "p",
            "pre",
            "search",
            "section",
            "summary",
            "table",
            "ul",
        ];
        for name in P_CLOSERS {
            let line = html(&format!("<div><p><mark>x<{name}>y</{name}>z</div>")).expect("text");
            assert_eq!(line.styles, [(0..1, Style::Mark)], "{name}: {line:?}");
            assert!(line.text.starts_with("x\n"), "{name}: {line:?}");
        }
    }

    #[test]
    fn a_table_cell_holds_what_opens_in_it() {
        // GitHub's answers, `.work/R19/r43/gh-gfm2.log`.
        for tag in ["b", "mark", "i"] {
            let cells = html(&format!(
                "<div><table><tr><td><{tag}>x</td><td>y</td></tr></table>z</div>"
            ))
            .expect("text");
            assert_eq!(cells.styles.len(), 1, "{tag}: {cells:?}");
            assert!(
                cells.styles[0].0.contains(&0) && cells.styles[0].0.len() == 1,
                "{tag}: {cells:?}"
            );
        }
        let around = html("<div><p>a<b>x<table><tr><td>y</td></tr></table>z</div>").expect("text");
        let bold: Vec<&str> = around
            .styles
            .iter()
            .map(|(range, _)| &around.text[range.clone()])
            .collect();
        assert_eq!(bold, ["x", "z"], "{around:?}");
    }

    #[test]
    fn open_formatting_reaches_the_cells_and_a_cell_ends_at_the_next() {
        // GitHub's answers, `.work/R4.3-r5/github-oracles.log`.
        let bold = |source: &str| {
            let line = html(source).expect("text");
            let words: Vec<String> = line
                .styles
                .iter()
                .map(|(range, _)| line.text[range.clone()].to_string())
                .collect();
            words.concat()
        };
        assert_eq!(
            bold("<div><b>x<table><tr><td>y</td></tr></table>z</b></div>").replace('\n', ""),
            "xyz"
        );
        assert_eq!(
            bold("<div><table><tr><td><section><b>x<td>y</tr></table>z</div>").trim(),
            "x"
        );
        assert_eq!(
            bold("<div><table><tr><td><section><b>x<tr><td>y</tr></table>z</div>").trim(),
            "x"
        );
    }

    #[test]
    fn an_inline_start_rebuilds_before_a_table() {
        // GitHub's answers, `.work/R4.3-r6/github-oracles.log`.
        for tag in ["i", "span", "u", "mark"] {
            let source =
                format!("<div><p><b>x</p><{tag}><table><tr><td>y</td></tr></table>z</{tag}></div>");
            let line = html(&source).expect("text");
            let at = line.text.find('y').expect("the cell");
            let bold = line
                .styles
                .iter()
                .any(|(range, style)| *style == Style::Strong && range.contains(&at));
            assert!(bold, "{source}: {line:?}");
        }
    }

    #[test]
    fn head_tags_rebuild_nothing_and_an_end_br_drops_its_space() {
        // GitHub's answers, `.work/R4.3-r7/github-oracles.log`.
        let line =
            html("<div><p><b>x</p><meta name=\"k\"><table><tr><td>y</td></tr></table>z</div>")
                .expect("text");
        let at = line.text.find('y').expect("the cell");
        assert!(
            !line.styles.iter().any(|(range, _)| range.contains(&at)),
            "{line:?}"
        );
        assert_eq!(html("<div>a </br>b</div>").expect("text").text, "a\nb");
    }

    #[test]
    fn cells_part_with_tabs_and_rows_with_lines() {
        // Chromium's innerText: a tab between cells.
        for (source, shown) in [
            (
                "<div><table><tr><td>hello</td><td>world</td></tr></table></div>",
                "hello\tworld",
            ),
            (
                "<div><table>\n<tr><th>a</th> <th>b</th></tr>\n<tr><td>c</td><td></td><td>e</td></tr>\n</table>z</div>",
                "a\tb\nc\t\te\nz",
            ),
            (
                "<div><table><tr><td><p>a</p></td><td>b</td></tr></table></div>",
                "a\n\n\tb",
            ),
            (
                "<div><table><tr><td></td><td>x</td></tr></table></div>",
                "\tx",
            ),
            (
                "<div><table><caption>t</caption><tr><td>a</td><td>b</td></tr><tr><td>c</td></tr></table>z</div>",
                "t\na\tb\nc\nz",
            ),
            ("<div><p>one</p><p>two</p></div>", "one\n\ntwo"),
            (
                "<div><fieldset><legend>k</legend>v</fieldset></div>",
                "k\nv",
            ),
        ] {
            assert_eq!(html(source).expect("text").text, shown, "{source:?}");
        }
    }

    #[test]
    fn nested_tables_read_in_linear_time() {
        let source = |n: usize| {
            format!(
                "<div>{}x{}</div>",
                "<table><tr><td>".repeat(n),
                "</td></tr></table>".repeat(n)
            )
        };
        let time = |n| crate::clock::spent(|| drop(parse(&source(n))));
        let (mut small, mut large) = (std::time::Duration::MAX, std::time::Duration::MAX);
        for _ in 0..5 {
            small = small.min(time(1 << 8));
            large = large.min(time(1 << 10));
        }
        assert!(large < small * 8, "{small:?} then {large:?}");
    }

    #[test]
    fn hostile_html_reads_in_linear_time() {
        let sources = |n: usize| {
            [
                "<".repeat(2 * n),
                "<a".repeat(n),
                "<!--".repeat(n),
                "<b>".repeat(n),
                "<b>".repeat(n / 2) + &"</i>".repeat(n / 2),
                "<script></script>".repeat(n / 8),
                "<div>a ".repeat(n),
                "<b> ".repeat(n),
                "<li><b>x ".repeat(n / 2),
                "<p><i>x ".repeat(n / 2),
                "<div/>".repeat(n),
            ]
        };
        let time = |n| {
            crate::clock::spent(|| {
                for source in sources(n) {
                    parse(&format!(
                        "<div>
{source}"
                    ));
                }
            })
        };
        let (small, large) = (time(1 << 14), time(1 << 16));
        // Four times the input, at most eight the time.
        assert!(large < small * 8, "{small:?} then {large:?}");
    }

    #[test]
    fn references_and_long_runs_read_as_a_browser_reads_them() {
        let line = html("<div>&notit; &amp &ampx &lt3 &nope;</div>").expect("text");
        assert_eq!(line.text, "¬it; & &x <3 &nope;");
        let amps = "&".repeat(1 << 18);
        let started = web_time::Instant::now();
        let line = html(&format!("<div>{amps}</div>")).expect("text");
        assert_eq!(line.text.len(), amps.len());
        let spent = started.elapsed();
        assert!(spent.as_secs() < 2, "256 KiB took {spent:?}");
    }

    #[test]
    fn the_source_s_own_noncharacters_draw_as_written() {
        let line = html("<div>a&#xFDD0;b\u{FDE5}c</div>").expect("text");
        assert_eq!(line.text, "a\u{FDD0}b\u{FDE5}c");
    }

    #[test]
    fn nesting_past_the_bound_reads_as_text() {
        let line = html(&format!("<div>{}x</div>", "<b>".repeat(1100))).expect("text");
        assert!(
            line.text.starts_with("<b>") && line.text.ends_with("<b>x"),
            "{:?}",
            &line.text[..9]
        );
        assert_eq!(line.text.matches("<b>").count(), 1100 - 1024 + 1);
    }

    #[test]
    fn pre_keeps_its_white_space() {
        let line = html("<pre>a  b\nc\t d</pre>").expect("text");
        assert_eq!(line.text, "a  b\nc\t d");
        let nested = html("<div>x  y<pre>a  b\n <i>c  d</i></pre>z  w</div>").expect("text");
        assert_eq!(nested.text, "x y\na  b\n c  d\nz w");
        let lines = html("<pre>\nline one\n\n  two</pre>").expect("text");
        assert_eq!(lines.text, "line one\n\n  two");
    }

    #[test]
    fn a_slash_closes_only_what_html_closes() {
        let divs = html(&format!("<div>{}x</div>", "<div/>".repeat(1100))).expect("text");
        assert_eq!(divs.text.matches("<div/>").count(), 1100 - 1024 + 1);
        let paths = "<path/>".repeat(1100);
        let svg = html(&format!("<div><svg>{paths}</svg>x</div>")).expect("text");
        assert_eq!(svg.text, "x");
        let left = html(&format!("<div><svg><p>{}x</div>", "<g/>".repeat(1100)));
        assert_eq!(
            left.expect("text").text.matches("<g/>").count(),
            1100 - 1022
        );
    }

    #[test]
    fn source_html_cannot_name_the_writer_s_controls() {
        for name in ["math", "note", "auto", "image=\"0\""] {
            for lead in ["\u{FDD0}", "&#xFDD0;", "data-ely-"] {
                let block = html(&format!("<div><span {lead}{name}>:smile:</span></div>"));
                let block = block.expect("text");
                assert_eq!(
                    (block.text.as_str(), block.styles.len()),
                    ("😄", 0),
                    "{lead}{name}"
                );
            }
        }
    }
}
