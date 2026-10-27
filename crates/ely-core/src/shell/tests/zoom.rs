use std::{cell::RefCell, rc::Rc};

use gpui::{
    Context, FocusHandle, IntoElement, KeyUpEvent, Keystroke, ParentElement as _, Render,
    TestAppContext, VisualTestContext, Window,
};

use crate::{
    primitives::FocusScope,
    shell::{ZoomControl, ZoomStep},
};

/// An owned zoom control and the steps it handed back.
struct Zoomed {
    root: FocusHandle,
    steps: Rc<RefCell<Vec<ZoomStep>>>,
}

impl Render for Zoomed {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let steps = self.steps.clone();
        FocusScope::new(&self.root).root().child(
            ZoomControl::new("zoom").owned(125, move |step, _, _| steps.borrow_mut().push(step)),
        )
    }
}

/// Moves to the next Tab stop and presses Enter there.
fn enter_on_next(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("enter");
    // A key click fires on the release.
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").expect("a key"),
    });
}

#[gpui::test]
fn the_percentage_is_a_tab_stop_that_resets(cx: &mut TestAppContext) {
    cx.update(crate::init_for_tests);
    let steps = Rc::new(RefCell::new(Vec::new()));
    let shared = steps.clone();
    let (_, cx) = cx.add_window_view(|_, cx| Zoomed {
        root: cx.focus_handle(),
        steps: shared,
    });
    enter_on_next(cx);
    assert_eq!(*steps.borrow(), vec![ZoomStep::Out], "minus first");
    enter_on_next(cx);
    assert_eq!(
        *steps.borrow(),
        vec![ZoomStep::Out, ZoomStep::Reset],
        "then the percentage"
    );
    enter_on_next(cx);
    assert_eq!(
        *steps.borrow(),
        vec![ZoomStep::Out, ZoomStep::Reset, ZoomStep::In],
        "then plus"
    );
}
