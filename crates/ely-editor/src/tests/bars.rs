use gpui::TestAppContext;

use super::editor;

#[gpui::test]
fn a_gutter_bar_marks_its_lines_and_rides_edits(cx: &mut TestAppContext) {
    let (editor, cx) = editor("one\ntwo\nthree\nfour", cx);
    editor.update(cx, |editor, cx| {
        let (first, second) = (gpui::red(), gpui::blue());
        let (two, three) = (4..7, 8..13);
        editor.set_gutter_bars(vec![(two, first), (three, second), (2..99, first)], cx);
        assert_eq!(
            editor.marks.bars.len(),
            2,
            "a span past the text is dropped"
        );
        let bars: Vec<_> = (0..4).map(|line| editor.bar(line)).collect();
        assert_eq!(bars, [None, Some(first), Some(second), None]);
        editor.edit([(0..0, "zero\n".to_string())], cx);
        let moved: Vec<_> = (0..5).map(|line| editor.bar(line)).collect();
        assert_eq!(
            moved,
            [None, None, Some(first), Some(second), None],
            "bars move with the text above them"
        );
        editor.edit([(8..10, String::new())], cx);
        assert_eq!(
            editor.marks.bars.len(),
            1,
            "an edit cutting into a bar drops it"
        );
        editor.set_text("new", cx);
        assert!(editor.marks.bars.is_empty(), "a new text drops every bar");
    });
}
