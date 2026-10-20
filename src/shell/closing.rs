//! A window's close check, asked by system and drawn closes.

use std::{collections::HashMap, rc::Rc};

use gpui::{App, Global, Window, WindowId};

type Check = Rc<dyn Fn(&mut Window, &mut App) -> bool>;

#[derive(Default)]
struct Checks(HashMap<WindowId, Check>);

impl Global for Checks {}

/// Asks `check` before any Close removes the window.
pub fn guard_close(
    window: &mut Window,
    cx: &mut App,
    check: impl Fn(&mut Window, &mut App) -> bool + 'static,
) {
    if !cx.has_global::<Checks>() {
        cx.set_global(Checks::default());
        cx.on_window_closed(|cx, id| {
            cx.global_mut::<Checks>().0.remove(&id);
        })
        .detach();
    }
    let check: Check = Rc::new(check);
    let id = window.window_handle().window_id();
    cx.global_mut::<Checks>().0.insert(id, check.clone());
    window.on_window_should_close(cx, move |window, cx| check(window, cx));
}

/// Closes the window once its check, if any, lets it.
pub(super) fn close(window: &mut Window, cx: &mut App) {
    let id = window.window_handle().window_id();
    let check = cx
        .try_global::<Checks>()
        .and_then(|checks| checks.0.get(&id).cloned());
    if let Some(check) = check
        && !check(window, cx)
    {
        log::info!("window controls: the close check kept the window");
        return;
    }
    window.remove_window();
}
