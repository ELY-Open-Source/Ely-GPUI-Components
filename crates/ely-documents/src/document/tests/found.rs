use gpui::TestAppContext;

use super::{MarkdownMode, open};

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
            input.bounds_for(selection.end).expect("the match is laid out"),
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
