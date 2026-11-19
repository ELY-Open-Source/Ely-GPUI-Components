use super::*;

#[test]
fn blocks_nest_as_written() {
    let nodes = parse(
        "# Lift\n\nBlend **toward** [white](https://ely.dev).\n\n- [x] clamp\n- [ ] gamma\n\n> quoted\n\n```rust\nfn main() {}\n```\n\n---\n",
    );
    assert!(matches!(&nodes[0], Node::Heading(1, text) if text.text == "Lift"));
    let Node::Paragraph(line) = &nodes[1] else {
        panic!("a paragraph, not {:?}", nodes[1])
    };
    assert_eq!(line.text, "Blend toward white.");
    assert_eq!(line.styles, [(6..12, Style::Strong), (13..18, Style::Link)]);
    assert_eq!(line.links, [(13..18, "https://ely.dev".to_string())]);
    let Node::List { start: None, items } = &nodes[2] else {
        panic!("a list, not {:?}", nodes[2])
    };
    assert_eq!(
        items.iter().map(|(task, _)| *task).collect::<Vec<_>>(),
        [Some(true), Some(false)]
    );
    assert!(matches!(&items[0].1[0], Node::Paragraph(text) if text.text == "clamp"));
    assert!(matches!(&nodes[3], Node::Quote(inner) if inner.len() == 1));
    assert_eq!(
        nodes[4],
        Node::Code {
            language: Some("rust".into()),
            text: "fn main() {}".into(),
            from: 95
        }
    );
    assert_eq!(nodes[5], Node::Rule);
}

#[test]
fn an_image_inside_an_image_is_the_outer_one() {
    // markdown-rs drops the inner image from the alt.
    assert_eq!(
        parse("![outer ![inner](inner.png)](outer.png)"),
        [Node::Image {
            alt: "outer ".into(),
            url: "outer.png".into(),
            link: None,
            at: 0
        }]
    );
}

#[test]
fn an_image_splits_its_paragraph_and_references_resolve() {
    let nodes = parse("Before ![a][pic] after [site].\n\n[pic]: a.png\n[site]: https://ely.dev\n");
    assert!(matches!(&nodes[0], Node::Paragraph(text) if text.text == "Before "));
    assert_eq!(
        nodes[1],
        Node::Image {
            alt: "a".into(),
            url: "a.png".into(),
            link: None,
            at: 7
        }
    );
    let Node::Paragraph(after) = &nodes[2] else {
        panic!("a paragraph, not {:?}", nodes[2])
    };
    assert_eq!(after.text, " after site.");
    assert_eq!(after.links, [(7..11, "https://ely.dev".to_string())]);
    assert_eq!(nodes.len(), 3, "definitions draw nothing: {nodes:?}");
}

#[test]
fn a_soft_break_draws_as_a_line() {
    for source in [
        "one\ntwo  \nthree",
        "one\r\ntwo  \r\nthree",
        "one\rtwo  \rthree",
    ] {
        let nodes = parse(source);
        assert!(
            matches!(&nodes[0], Node::Paragraph(text) if text.text == "one\ntwo\nthree"),
            "{source:?} draws {nodes:?}"
        );
    }
}

#[test]
fn a_joined_reader_reads_a_soft_break_as_a_space() {
    let source = "one\ntwo  \nthree";
    let root = super::super::markdown::tree(source);
    let mut out = Vec::new();
    let paragraph = &root.children().expect("blocks")[0];
    Reader::joined(source, &root).block(paragraph, &mut out);
    assert!(
        matches!(&out[0], Node::Paragraph(text) if text.text == "one two\nthree"),
        "{out:?}"
    );
}

#[test]
fn a_heading_title_reads_on_one_line() {
    let source = "one\ntwo\n===\n";
    let root = super::super::markdown::tree(source);
    assert_eq!(Reader::new(source, &root).headings[0].title, "one two");
}

#[test]
fn definitions_nested_in_quotes_and_lists_resolve() {
    for source in [
        "[site] ![pic]\n\n> [site]: https://ely.dev\n> [pic]: a.png\n",
        "[site] ![pic]\n\n- [site]: https://ely.dev\n- [pic]: a.png\n",
    ] {
        let nodes = parse(source);
        let Node::Paragraph(text) = &nodes[0] else {
            panic!("a paragraph, not {:?}", nodes[0])
        };
        assert_eq!(text.links, [(0..4, "https://ely.dev".to_string())]);
        assert_eq!(
            nodes[1],
            Node::Image {
                alt: "pic".into(),
                url: "a.png".into(),
                link: None,
                at: 7
            }
        );
    }
}

#[test]
fn an_image_inside_styles_and_links_stays_an_image() {
    let nodes = parse("*a ![x](x.png) b* [![y](y.png)](https://ely.dev)");
    let Node::Paragraph(before) = &nodes[0] else {
        panic!("a paragraph, not {:?}", nodes[0])
    };
    assert_eq!(
        (before.text.as_str(), &before.styles[..]),
        ("a ", &[(0..2, Style::Emphasis)][..])
    );
    assert_eq!(
        nodes[1],
        Node::Image {
            alt: "x".into(),
            url: "x.png".into(),
            link: None,
            at: 3
        }
    );
    let Node::Paragraph(after) = &nodes[2] else {
        panic!("a paragraph, not {:?}", nodes[2])
    };
    assert_eq!(
        (after.text.as_str(), &after.styles[..]),
        (" b ", &[(0..2, Style::Emphasis)][..])
    );
    assert_eq!(
        nodes[3],
        Node::Image {
            alt: "y".into(),
            url: "y.png".into(),
            link: Some("https://ely.dev".into()),
            at: 19
        }
    );
    assert_eq!(nodes.len(), 4, "{nodes:?}");
}

#[test]
fn a_heading_keeps_its_images_after_it() {
    let nodes = parse("# Lift ![x](x.png)");
    assert!(matches!(&nodes[0], Node::Heading(1, text) if text.text == "Lift "));
    assert!(matches!(&nodes[1], Node::Image { url, .. } if url == "x.png"));
}

#[test]
fn display_math_stands_alone() {
    assert_eq!(parse("$$c = 1$$"), [Node::Math("c = 1".into())]);
    assert_eq!(parse("$$\nc = 1\n$$"), [Node::Math("c = 1".into())]);
    // GitHub closes at the first `$$`, even mid-text.
    let both = parse("$$\na.$$\n\n$$\nb\\tag{8}$$\n");
    assert_eq!(
        both,
        [Node::Math("a.".into()), Node::Math("b\\tag{8}".into())]
    );
    assert_eq!(parse("$$a = 1\n$$"), [Node::Math("a = 1".into())]);
    let three = parse("$$\na.$$\n\n$$\nb\n$$\n\n$$\nc\n$$\n");
    let math = |tex: &str| Node::Math(tex.into());
    assert_eq!(three, [math("a."), math("b"), math("c")]);
    let packed = parse("$$\na.$$\n$$\nb\n$$");
    assert_eq!(packed, [math("a."), math("b")], "GitHub shows both");
    let opened = parse("$$a\n$$\n\n$$\nb\n$$\n");
    assert_eq!(
        opened,
        [math("a"), math("b")],
        "a text-opened one closes first"
    );
    let Node::Paragraph(inline) = &parse("so $c$ holds")[0] else {
        panic!("inline math stays in its paragraph")
    };
    assert_eq!(inline.styles, [(3..4, Style::Math)]);
}

#[test]
fn tables_keep_their_cells_and_alignment() {
    let nodes = parse("| Name | Lift |\n|:--|--:|\n| accent | 0.5 |\n");
    let Node::Table { aligns, head, rows } = &nodes[0] else {
        panic!("a table, not {:?}", nodes[0])
    };
    assert_eq!(aligns, &[AlignKind::Left, AlignKind::Right]);
    let text = |cell: &[Node]| match cell {
        [Node::Paragraph(line)] => line.text.clone(),
        other => panic!("one line, not {other:?}"),
    };
    assert_eq!(
        head.iter().map(|cell| text(cell)).collect::<Vec<_>>(),
        ["Name", "Lift"]
    );
    assert_eq!(text(&rows[0][1]), "0.5");
}

#[test]
fn a_picture_in_a_cell_stays_a_picture() {
    let nodes = parse("| Picture |\n| --- |\n| ![shot](assets/shot.png) |\n");
    let Node::Table { rows, .. } = &nodes[0] else {
        panic!("a table, not {:?}", nodes[0])
    };
    assert!(
        matches!(&rows[0][0][..], [Node::Image { alt, url, .. }] if alt == "shot" && url == "assets/shot.png")
    );
}

#[test]
fn cells_past_the_head_stay_undrawn() {
    let nodes = parse("| a |\n| - |\n| b | c |\n");
    let Node::Table { head, rows, .. } = &nodes[0] else {
        panic!("a table, not {:?}", nodes[0])
    };
    assert_eq!((head.len(), rows[0].len()), (1, 1));
}

#[test]
fn footnotes_number_their_references_and_hold_their_body() {
    let nodes = parse("Lift[^a].\n\n[^a]: Toward white.\n");
    let Node::Paragraph(line) = &nodes[0] else {
        panic!("a paragraph, not {:?}", nodes[0])
    };
    assert_eq!(line.text, "Lift¹.");
    assert_eq!(line.styles, [(4..6, Style::Footnote)]);
    assert!(matches!(&nodes[1], Node::Footnote { label, body } if label == "1" && body.len() == 1));
    let alone = parse(
        "[^z]: Unread.
",
    );
    assert!(matches!(&alone[0], Node::Footnote { label, .. } if label == "z"));
}

mod inline;

#[test]
fn what_a_browser_hides_draws_nothing() {
    // Chromium's innerText, `.work/R20.35/oracle/sweep.html`.
    let read = |source: &str| -> String {
        let texts: Vec<String> = (parse(source).into_iter())
            .map(|node| match node {
                Node::Paragraph(line) => line.text,
                other => panic!("a paragraph, not {other:?}"),
            })
            .collect();
        texts.join("\n\n")
    };
    for (inner, inline, block) in [
        ("<dialog open>b</dialog>", "a\n\nb\nc", "a\nb\nc"),
        ("<dialog>b</dialog>", "a\n\nc", "a c"),
        ("<span hidden><b>x</b></span>", "a  c", "a c"),
        ("<span hidden>x</span>", "a  c", "a c"),
        ("<div hidden>b</div>", "a\n\nc", "a c"),
        ("<datalist>b</datalist>", "a  c", "a c"),
        ("<audio>b</audio>", "a  c", "a c"),
        ("<audio controls>b</audio>", "a  c", "a  c"),
        ("<video><b>x</b> y</video>", "a  c", "a  c"),
        ("<canvas>b</canvas>", "a  c", "a  c"),
        ("<details><p>b</p></details>", "a\n\nc", "a\nc"),
        (
            "<details><summary>s</summary><summary>t</summary>b</details>",
            "a\n\ns\nc",
            "a\ns\nc",
        ),
        (
            "<details open><summary>s</summary>b</details>",
            "a\n\ns\nb\nc",
            "a\ns\nb\nc",
        ),
        (
            "<option>b</option><option>d</option>",
            "a\nb\nd\nc",
            "a\nb\nd\nc",
        ),
        ("<ul><li>b<li hidden>x</ul>", "a\n\nb\nc", "a\nb\nc"),
    ] {
        let source = format!("a {inner} c");
        // Inline text keeps its spaces as written.
        assert_eq!(read(&source), inline, "{source:?}");
        assert_eq!(read(&format!("<div>{source}</div>")), block, "{source:?}");
    }
}

#[test]
fn hidden_cells_owe_no_tab() {
    // Chromium's innerText, `.work/R20.56/oracle/cells.html`.
    let read = |source: &str| match parse(source).into_iter().next() {
        Some(Node::Paragraph(line)) => line.text,
        other => panic!("a paragraph, not {other:?}"),
    };
    for (row, shown) in [
        ("<td>a</td><td hidden>b</td>", "a"),
        ("<td>a</td><td>b</td><td hidden>c</td>", "a\tb"),
        ("<td>a</td><td hidden>b</td><td hidden>c</td>", "a"),
        ("<td>a</td><td hidden>b</td><td>c</td>", "a\tc"),
        ("<td hidden>a</td><td>b</td>", "b"),
        ("<td>a</td><td hidden>b</td> ", "a"),
    ] {
        let source = format!("<div><table><tr>{row}</tr></table></div>");
        assert_eq!(read(&source), shown, "{source:?}");
    }
    let rows = "<div><table><tr><th>a</th><th hidden>b</th></tr><tr><td>c</td><td hidden>d</td><td hidden>e</td></tr></table>z</div>";
    assert_eq!(read(rows), "a\nc\nz");
}
