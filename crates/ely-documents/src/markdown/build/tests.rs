use super::{
    super::{super::clock::spent, mdast::Node},
    tree,
};

fn blocks(root: &Node) -> &[Node] {
    root.children().expect("the root holds blocks")
}

/// Four times the input in at most eight times the time, the best of five rounds of thread time.
fn linear(source: impl Fn(usize) -> String) {
    let texts = [source(1 << 9), source(1 << 11)];
    let mut best = [std::time::Duration::MAX; 2];
    for _ in 0..5 {
        for (text, best) in texts.iter().zip(&mut best) {
            *best = (*best).min(spent(|| {
                tree(text, false);
            }));
        }
    }
    let [small, large] = best;
    assert!(large < small * 8, "{small:?} then {large:?}");
}

#[test]
fn an_unclosed_fence_before_wide_text_reads() {
    let root = tree("$$\n中文", false);
    assert!(
        matches!(blocks(&root), [Node::Math(math)] if math.value == "中文"),
        "{root:?}"
    );
}

#[test]
fn dollar_fences_inside_open_code_stay_code() {
    let source = |n: usize| format!("{}\n{}", "~".repeat(n), "$$\nx\n$$\n\n".repeat(n));
    let root = tree(&source(64), false);
    assert!(matches!(blocks(&root), [Node::Code(_)]), "{root:?}");
    linear(source);
}

#[test]
fn closing_parentheses_trail_off_in_linear_time() {
    linear(|n| format!("https://a.b/{}", ")".repeat(n * 16)));
}

#[test]
fn many_literal_links_read_in_linear_time() {
    linear(|n| "www.a.b &amp; ".repeat(n * 16));
}

#[test]
fn a_definition_in_an_item_stays_in_the_item() {
    let root = tree("- foo\n\n  [r]: /url\n\n[r]\n", false);
    let [Node::List(list), Node::Paragraph(_)] = blocks(&root) else {
        panic!("a list and a paragraph: {root:?}");
    };
    let [Node::ListItem(item)] = &list.children[..] else {
        panic!("one item: {list:?}");
    };
    assert!(
        matches!(
            &item.children[..],
            [Node::Paragraph(_), Node::Definition(_)]
        ),
        "{item:?}"
    );
}

#[test]
fn cells_past_the_head_keep_their_text_as_written() {
    let root = tree("| a |\n| - |\n| b | [l][r] | x\\|y |\n\n[r]: /url\n", false);
    let Node::Table(table) = &blocks(&root)[0] else {
        panic!("a table: {root:?}");
    };
    let cells = table.children[1].children().expect("cells");
    let text = |ix: usize| match &cells[ix].children().expect("inlines")[..] {
        [Node::Text(text)] => text.value.clone(),
        other => panic!("written text: {other:?}"),
    };
    assert_eq!(cells.len(), 3, "{cells:?}");
    assert_eq!((text(1), text(2)), ("[l][r]".into(), "x\\|y".into()));
    linear(|n| {
        let rows = "| b | [r0] |\n".repeat(n);
        let labels: String = (0..n).map(|ix| format!("[r{ix}]: /u\n")).collect();
        format!("| a |\n| - |\n{rows}\n{labels}")
    });
}

#[test]
fn rejected_domains_read_in_linear_time() {
    linear(|n| "www.a_".repeat(n * 16) + "z");
}

#[test]
fn a_byte_order_mark_starts_no_text() {
    let root = tree("\u{FEFF}---\ntitle: x\n---\n", true);
    assert!(
        matches!(blocks(&root), [Node::Yaml(front)] if front.value == "title: x"),
        "{root:?}"
    );
    let root = tree("\u{FEFF}# Title\n", false);
    let [Node::Heading(heading)] = blocks(&root) else {
        panic!("a heading: {root:?}");
    };
    assert_eq!(heading.position, 3..10);
    let root = tree(&("\u{FEFF}".repeat(1024) + "x"), false);
    assert!(matches!(blocks(&root), [Node::Paragraph(_)]), "{root:?}");
}

#[test]
fn a_list_is_loose_when_its_items_hold_paragraphs() {
    let spread = |source: &str| {
        let root = tree(source, false);
        let spread: Vec<bool> = (blocks(&root).iter())
            .filter_map(|node| match node {
                Node::List(list) => Some(list.spread),
                _ => None,
            })
            .collect();
        spread
    };
    assert_eq!(spread("- a\n- b\n"), [false]);
    assert_eq!(spread("- a\n\n- b\n"), [true]);
    assert_eq!(spread("1. a\n\n   more\n2. b\n"), [true]);
    let root = tree("- a\n\n- b\n  - c\n  - d\n", false);
    let Some(Node::List(outer)) = blocks(&root).first() else {
        panic!("{root:?}")
    };
    let inner = outer.children[1].children().expect("an item").last();
    assert!(outer.spread, "{root:?}");
    assert!(
        matches!(inner, Some(Node::List(list)) if !list.spread),
        "{root:?}"
    );
}
