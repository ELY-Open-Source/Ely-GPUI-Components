//! Frames of Ely's heaviest views and a 2 MiB Markdown read, on gpui's test platform.

use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

use criterion::{Criterion, criterion_group, criterion_main};
use ely_gpui_component::{
    documents,
    editor::CodeEditor,
    init_for_tests,
    tables::{Column, DataTable, Row},
    terminal::Terminal,
    theme::Theme,
};
use gpui::{
    AnyView, AppContext, Context, Focusable, IntoElement, Modifiers, ParentElement, Pixels, Render,
    ScrollDelta, ScrollEvent, Styled, TestAppContext, TouchPhase, VisualTestContext, Window, div,
    point, px,
};

struct Rows(Rc<Vec<Row>>);

impl Render for Rows {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(800.0)).h(px(600.0)).child(
            DataTable::new(
                "rows",
                [Column::new("name", "Name"), Column::new("size", "Size")],
            )
            .rows(self.0.clone())
            .virtualized()
            .h_full(),
        )
    }
}

/// Counts the window's draws around the view it holds.
struct Counted {
    view: AnyView,
    draws: Rc<Cell<usize>>,
}

impl Render for Counted {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.draws.set(self.draws.get() + 1);
        div().size_full().child(self.view.clone())
    }
}

fn app() -> TestAppContext {
    let cx = TestAppContext::single();
    cx.update(|cx| {
        init_for_tests(cx);
        Theme::update(cx, |theme| theme.reduced_motion = true);
    });
    cx
}

fn frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

fn counted(
    app: &mut TestAppContext,
    view: impl FnOnce(&mut Window, &mut gpui::App) -> AnyView + 'static,
) -> (&mut VisualTestContext, Rc<Cell<usize>>) {
    let draws = Rc::new(Cell::new(0));
    let held = draws.clone();
    let (_, cx) = app.add_window_view(move |window, cx| Counted {
        view: view(window, cx),
        draws: held,
    });
    frame(cx);
    (cx, draws)
}

fn draws_once(
    cx: &mut VisualTestContext,
    draws: &Cell<usize>,
    input: impl FnOnce(&mut VisualTestContext),
) {
    let before = draws.get();
    input(cx);
    assert_eq!(
        draws.get() - before,
        1,
        "the input must draw the window once"
    );
}

fn wheel(cx: &mut VisualTestContext, at: (f32, f32), down: Pixels) {
    cx.simulate_event(ScrollEvent {
        position: point(px(at.0), px(at.1)),
        delta: ScrollDelta::Pixels(point(px(0.0), down)),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
        source: gpui::ScrollSource::Pointer,
    });
}

fn table_scroll(c: &mut Criterion) {
    let rows: Vec<Row> = (0..100_000)
        .map(|ix| {
            Row::new(
                format!("r{ix}"),
                [format!("file {ix}").into(), format!("{ix} KB").into()],
            )
        })
        .collect();
    let mut app = app();
    let (cx, draws) = counted(&mut app, |_, cx| cx.new(|_| Rows(Rc::new(rows))).into());
    draws_once(cx, &draws, |cx| wheel(cx, (200.0, 300.0), px(-480.0)));
    draws_once(cx, &draws, |cx| wheel(cx, (200.0, 300.0), px(480.0)));
    let mut down = true;
    c.bench_function("table_scroll_100k_rows", |b| {
        b.iter(|| {
            wheel(cx, (200.0, 300.0), px(if down { -480.0 } else { 480.0 }));
            down = !down;
        })
    });
}

fn editor_typing(c: &mut Criterion) {
    let text: String = (0..10_000)
        .map(|ix| format!("let value_{ix} = {ix};\n"))
        .collect();
    let mut app = app();
    let editor = Rc::new(Cell::new(None));
    let made = editor.clone();
    let (cx, draws) = counted(&mut app, move |window, cx| {
        let view = cx.new(|cx| CodeEditor::new(text, window, cx));
        made.set(Some(view.clone()));
        view.into()
    });
    let editor = editor.take().expect("the window builds its editor");
    cx.update(|window, cx| {
        window.activate_window();
        editor.focus_handle(cx).focus(window, cx);
    });
    cx.run_until_parked();
    // The first key turns input to the keyboard, which gpui answers with a refresh.
    cx.simulate_input("x");
    cx.simulate_keystrokes("backspace");
    draws_once(cx, &draws, |cx| cx.simulate_input("x"));
    draws_once(cx, &draws, |cx| cx.simulate_keystrokes("backspace"));
    // Each typed letter is taken back untimed, so the file keeps its size.
    c.bench_function("editor_typing_10k_lines", |b| {
        b.iter_custom(|count| {
            let mut spent = Duration::ZERO;
            for _ in 0..count {
                let at = Instant::now();
                cx.simulate_input("x");
                spent += at.elapsed();
                cx.simulate_keystrokes("backspace");
            }
            spent
        })
    });
}

fn terminal_scrollback(c: &mut Criterion) {
    let output: String = (0..10_000)
        .map(|ix| format!("line {ix}: \x1b[32mok\x1b[0m\r\n"))
        .collect();
    let mut app = app();
    let (cx, draws) = counted(&mut app, move |_, cx| {
        cx.new(|cx| Terminal::replay(output.as_bytes(), 80, 24, cx))
            .into()
    });
    draws_once(cx, &draws, |cx| wheel(cx, (100.0, 100.0), px(4000.0)));
    draws_once(cx, &draws, |cx| wheel(cx, (100.0, 100.0), px(-4000.0)));
    let mut up = true;
    c.bench_function("terminal_scrollback_10k_lines", |b| {
        b.iter(|| {
            wheel(cx, (100.0, 100.0), px(if up { 4000.0 } else { -4000.0 }));
            up = !up;
        })
    });
}

fn markdown_read(c: &mut Criterion) {
    let block = "# Title\n\nA paragraph with **bold**, `code` and a [link](https://example.com).\n\n- one\n- two\n\n```rust\nfn main() {}\n```\n\n";
    let text = block.repeat((2 << 20) / block.len());
    c.bench_function("markdown_read_2mib", |b| {
        b.iter(|| documents::outline(&text))
    });
}

criterion_group! {
    name = budget;
    config = Criterion::default().sample_size(20);
    targets = table_scroll, editor_typing, terminal_scrollback, markdown_read
}
criterion_main!(budget);
