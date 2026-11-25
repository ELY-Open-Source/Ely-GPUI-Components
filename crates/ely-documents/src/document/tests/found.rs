use gpui::TestAppContext;

use super::{MarkdownMode, frames, open};

#[gpui::test]
fn a_match_late_in_a_long_cell_is_in_view(cx: &mut TestAppContext) {
    let cell = format!("{} needle", "long words ".repeat(80));
    let source = format!("| head |\n| --- |\n| {cell} |\n");
    let (document, cx) = open(&source, MarkdownMode::Visual, cx);
    let elsewhere = cx.update(|window, cx| {
        let handle = cx.focus_handle();
        window.focus(&handle, cx);
        handle
    });
    let at = source.find("needle").expect("the match is in the source");
    document.update_in(cx, |document, window, cx| {
        document.point_match(at..at + 6, window, cx)
    });
    cx.run_until_parked();
    let (selected, field, hit) = document.read_with(cx, |document, cx| {
        let input = document
            .open
            .as_ref()
            .expect("the cell opens")
            .input
            .read(cx);
        let selection = input.selection();
        (
            input.text()[selection.clone()].to_string(),
            input.bounds().expect("the cell is laid out"),
            input
                .bounds_for(selection.end)
                .expect("the match is laid out"),
        )
    });
    assert_eq!(selected, "needle");
    assert!(
        cx.update(|window, _| elsewhere.is_focused(window)),
        "focus stays in the find box"
    );
    assert!(
        field.left() <= hit.left() && hit.left() <= field.right(),
        "the match's end is inside its cell: {hit:?} in {field:?}"
    );
}

/// Where `line` sits in the source's view: 0 at the top, 1 at the bottom.
fn in_source_view(
    document: &gpui::Entity<super::MarkdownDocument>,
    line: usize,
    cx: &mut gpui::VisualTestContext,
) -> f32 {
    document.read_with(cx, |document, cx| {
        let editor = document.editor.read(cx);
        let metrics = editor.metrics();
        let top = editor.top_line().expect("the source is laid out");
        (line as f32 - top as f32 + 0.5) / (metrics.height / metrics.line)
    })
}

#[gpui::test]
fn a_match_in_view_still_moves_to_the_middle_of_the_source(cx: &mut TestAppContext) {
    let source: String = (0..300).map(|n| format!("line {n}\n")).collect();
    let (document, cx) = open(&source, MarkdownMode::Source, cx);
    document.update_in(cx, |document, window, cx| {
        document.set_top_line(100, window, cx)
    });
    frames(cx);
    let rows = document.read_with(cx, |document, cx| {
        let metrics = document.editor.read(cx).metrics();
        (metrics.height / metrics.line) as usize
    });
    let line = 100 + rows - 3;
    assert!(
        in_source_view(&document, line, cx) > 0.8,
        "the match starts near the bottom"
    );
    let at = source
        .find(&format!("line {line}\n"))
        .expect("the line is in the text");
    document.update_in(cx, |document, window, cx| {
        document.point_match(at..at + 4, window, cx)
    });
    frames(cx);
    let place = in_source_view(&document, line, cx);
    assert!(
        (0.4..0.6).contains(&place),
        "the match is mid-view: {place}"
    );
}

#[gpui::test]
fn a_match_in_view_still_moves_to_the_middle_of_the_blocks(cx: &mut TestAppContext) {
    let source: String = (0..120)
        .map(|n| format!("Paragraph {n} of the text.\n\n"))
        .collect();
    let (document, cx) = open(&source, MarkdownMode::Visual, cx);
    document.update_in(cx, |document, window, cx| {
        document.set_top_line(80, window, cx)
    });
    frames(cx);
    let (ix, view) = document.read_with(cx, |document, _| {
        let view = document.list.viewport_bounds();
        let first = document.list.logical_scroll_top().item_ix;
        let last = (first..document.parsed.blocks.len())
            .take_while(|&ix| {
                (document.list.bounds_for_item(ix))
                    .is_some_and(|block| block.bottom() <= view.bottom())
            })
            .last()
            .expect("whole blocks show");
        (last, view)
    });
    let range = document.read_with(cx, |document, _| document.parsed.blocks[ix].range.clone());
    let word = range.start..range.start + 9;
    document.update_in(cx, |document, window, cx| {
        document.point_match(word, window, cx)
    });
    frames(cx);
    let field = cx
        .debug_bounds("markdown-open")
        .expect("the match's block opens");
    let off = (field.center().y - view.center().y).abs();
    assert!(
        off < field.size.height,
        "the match's block is mid-view: {field:?} in {view:?}"
    );
}

#[gpui::test]
fn matches_are_marked_beside_their_blocks_and_in_the_open_one(cx: &mut TestAppContext) {
    let source = "A needle here.\n\nNothing.\n\nThe needle again.\n";
    let (document, cx) = open(source, MarkdownMode::Visual, cx);
    let hits: Vec<_> = source
        .match_indices("needle")
        .map(|(at, word)| at..at + word.len())
        .collect();
    let second = hits[1].clone();
    document.update(cx, |document, cx| document.set_found(hits, Some(1), cx));
    frames(cx);
    let marked: Vec<bool> = (0..3)
        .map(|ix| {
            cx.debug_bounds(Box::leak(format!("markdown-found-{ix}").into_boxed_str()))
                .is_some()
        })
        .collect();
    assert_eq!(
        marked,
        [true, false, true],
        "a bar beside each block with a match"
    );
    document.update_in(cx, |document, window, cx| {
        document.point_match(second.clone(), window, cx)
    });
    frames(cx);
    let (spans, start, wash) = document.read_with(cx, |document, cx| {
        let open = document.open.as_ref().expect("the match's block opens");
        let tone = crate::theme::ActiveTheme::theme(cx)
            .colors
            .warning
            .opacity(0.6);
        (document.found_open.borrow().clone(), open.range.start, tone)
    });
    let inside = second.start - start..second.end - start;
    assert!(
        spans
            .iter()
            .any(|(range, mark)| *range == inside && mark.background == Some(wash)),
        "the current match is washed in its field, not left to the selection: {spans:?}"
    );
    document.update(cx, |document, cx| document.set_found(Vec::new(), None, cx));
    frames(cx);
    assert!(
        cx.debug_bounds("markdown-found-0").is_none(),
        "no matches, no bars"
    );
}

#[gpui::test]
fn preview_steps_leave_focus_in_the_find_box(cx: &mut TestAppContext) {
    let source: String = (0..80).map(|n| format!("Paragraph {n}.\n\n")).collect();
    let (document, cx) = open(&source, MarkdownMode::Preview, cx);
    let elsewhere = cx.update(|window, cx| {
        let handle = cx.focus_handle();
        window.focus(&handle, cx);
        handle
    });
    let at = source
        .find("Paragraph 60.")
        .expect("the paragraph is in the text");
    document.update_in(cx, |document, window, cx| {
        document.point_match(at..at + 9, window, cx)
    });
    frames(cx);
    assert!(
        cx.update(|window, _| elsewhere.is_focused(window)),
        "focus stays in the find box"
    );
}

#[gpui::test]
fn closed_and_previewed_blocks_wash_their_matches(cx: &mut TestAppContext) {
    let source = "Nothing here.\n\nA **bold** needle.\n\n```\nlet needle = 1;\n```\n";
    for (mode, word) in [
        (MarkdownMode::Visual, "needle."),
        (MarkdownMode::Preview, "needle."),
        (MarkdownMode::Preview, "needle ="),
    ] {
        let (document, cx) = open(source, mode, cx);
        let at = source.find(word).expect("the match is in the source");
        let hit = at..at + 6;
        document.update(cx, |document, cx| {
            document.set_found(vec![hit.clone()], None, cx)
        });
        frames(cx);
        let open = document.read_with(cx, |document, _| document.open.is_some());
        assert!(!open, "{mode:?}: no block opened, the match is drawn");
        assert!(
            cx.debug_bounds("markdown-washed").is_some(),
            "{mode:?}: the drawn match is washed"
        );
        let washes = document.read_with(cx, |document, _| {
            let block = &document.parsed.blocks[document.block_at(hit.start)];
            document.washes_in(&block.range)
        });
        assert_eq!(washes.len(), 1, "{mode:?}: one wash in the match's block");
        document.update(cx, |document, cx| document.set_found(Vec::new(), None, cx));
        frames(cx);
        assert!(
            cx.debug_bounds("markdown-washed").is_none(),
            "{mode:?}: no matches, no washes"
        );
    }
}

#[gpui::test]
fn a_match_no_drawn_text_shows_washes_its_block(cx: &mut TestAppContext) {
    // Each: source, match, washed as text (else as a block).
    let cases = [
        ("Intro needle.\n\nMore.\n", "needle", true),
        ("---\ntitle: needle\n---\n\nBody.\n", "needle", true),
        ("Intro.\n\n![a needle](n.png)\n", "needle", false),
        ("Intro.\n\n$$\nx_{needle}\n$$\n", "needle", false),
        ("Intro.\n\n[TOC]\n\n# Head\n", "TOC", false),
    ];
    for mode in [MarkdownMode::Visual, MarkdownMode::Preview] {
        for (source, word, as_text) in cases {
            let (document, cx) = open(source, mode, cx);
            let at = source.find(word).expect("the match is in the source");
            let hit = at..at + word.len();
            document.update(cx, |document, cx| document.set_found(vec![hit], None, cx));
            frames(cx);
            let (text, block) = (
                cx.debug_bounds("markdown-washed").is_some(),
                cx.debug_bounds("markdown-washed-block").is_some(),
            );
            assert_eq!(
                (text, block),
                (as_text, !as_text),
                "{mode:?} {source:?}: washed as text {as_text}"
            );
            document.update(cx, |document, cx| document.set_found(Vec::new(), None, cx));
            frames(cx);
            assert!(
                cx.debug_bounds("markdown-washed-block").is_none(),
                "{mode:?} {source:?}: no matches, no block wash"
            );
        }
    }
}
