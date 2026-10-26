//! Folders in a list beside a table of files, built from Ely's public API alone.

use ely_gpui_component::{
    init_for_tests,
    lists::{ListItem, SelectableList},
    primitives::FocusScope,
    tables::{Column, DataTable, Row},
};
use gpui::{
    Context, Entity, FocusHandle, IntoElement, KeyUpEvent, Keystroke, Modifiers, ParentElement,
    Render, SharedString, Styled, TestAppContext, VisualTestContext, Window, div, point, px,
};

struct Browser {
    root: FocusHandle,
    folder: SharedString,
    picked: Vec<SharedString>,
}

impl Render for Browser {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (pick, open) = (cx.entity(), cx.entity());
        let files = DataTable::new(
            "files",
            [Column::new("name", "Name"), Column::new("size", "Size")],
        )
        .rows(vec![
            Row::new("a", ["notes.md".into(), "2 KB".into()]),
            Row::new("b", ["plan.pdf".into(), "1 MB".into()]),
            Row::new("c", ["logo.png".into(), "40 KB".into()]),
        ])
        .selected(self.picked.clone())
        .on_select(move |keys, _, cx| {
            pick.update(cx, |view, cx| {
                view.picked = keys.to_vec();
                cx.notify();
            })
        })
        .w(px(400.0));
        let folders = SelectableList::new("folders")
            .label("Folders")
            .row("docs", ListItem::new("docs", "Documents"))
            .row("pics", ListItem::new("pics", "Pictures"))
            .selected([self.folder.clone()])
            .on_activate(move |key, _, cx| {
                open.update(cx, |view, cx| {
                    view.folder = key.clone();
                    cx.notify();
                })
            });
        FocusScope::new(&self.root)
            .root()
            .size_full()
            .child(files)
            .child(div().w(px(200.0)).child(folders))
    }
}

fn browser(cx: &mut TestAppContext) -> (Entity<Browser>, &mut VisualTestContext) {
    cx.update(init_for_tests);
    let (view, cx) = cx.add_window_view(|_, cx| Browser {
        root: cx.focus_handle(),
        folder: "docs".into(),
        picked: Vec::new(),
    });
    let root = view.read_with(cx, |view, _| view.root.clone());
    cx.update(|window, cx| {
        window.activate_window();
        window.focus(&root, cx);
    });
    settle(cx);
    (view, cx)
}

fn settle(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

fn press(key: &str, cx: &mut VisualTestContext) {
    cx.simulate_keystrokes(key);
    settle(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke:
            Keystroke::parse_with_os(
                key,
                ely_gpui_component::typography::keys::os(
                    ely_gpui_component::theme::Platform::current(),
                ),
            )
            .expect("a key"),
    });
    settle(cx);
}

#[gpui::test]
fn a_row_box_selects_its_file(cx: &mut TestAppContext) {
    let (view, cx) = browser(cx);
    cx.simulate_click(point(px(14.0), px(90.0)), Modifiers::none());
    settle(cx);
    let picked = view.read_with(cx, |view, _| view.picked.clone());
    assert_eq!(picked, ["b"]);
}

#[gpui::test]
fn the_keyboard_opens_another_folder(cx: &mut TestAppContext) {
    let (view, cx) = browser(cx);
    cx.update(|window, cx| window.focus_prev(cx));
    press("down", cx);
    press("enter", cx);
    let folder = view.read_with(cx, |view, _| view.folder.clone());
    assert_eq!(folder, "pics");
}
