use std::rc::Rc;

use gpui::{
    App, AppContext as _, ClipboardItem, Div, ElementId, Entity, InteractiveElement, ParentElement,
    SharedString, Styled, Window, div,
};

use crate::{
    buttons::{Button, ButtonVariant},
    forms::{Input, TextInput},
    i18n,
    menus::{Menu, MenuItem, OverflowMenu},
    primitives::IconName,
    theme::ControlSize,
};

/// Gets a comment's index.
pub(crate) type OnComment = Rc<dyn Fn(usize, &mut Window, &mut App)>;
/// Gets a comment's index and its new text.
pub(crate) type OnEdit = Rc<dyn Fn(usize, &SharedString, &mut Window, &mut App)>;
/// The comment being edited, by index, and its field.
pub(crate) type Open = Option<(usize, Entity<TextInput>)>;

/// The ⋯ menu of comment `ix`: edit while `open` is given, copy always, delete while `on_delete` is; on the first comment delete takes the thread.
pub(crate) fn menu(
    id: ElementId,
    ix: usize,
    body: &SharedString,
    open: Option<&Entity<Open>>,
    on_delete: Option<OnComment>,
    cx: &App,
) -> OverflowMenu {
    let mut menu = Menu::new();
    if let Some(open) = open {
        let (open, body) = (open.clone(), body.clone());
        menu = menu.item(
            MenuItem::new(i18n::text(cx, "comments.edit", &[]))
                .icon(IconName::Pencil)
                .on_click(move |window, cx| begin(&open, ix, &body, window, cx)),
        );
    }
    let copied = body.clone();
    menu = menu.item(
        MenuItem::new(i18n::text(cx, "comments.copy", &[]))
            .icon(IconName::Copy)
            .on_click(move |_, cx| {
                log::info!("comment: copied {ix}");
                cx.write_to_clipboard(ClipboardItem::new_string(copied.to_string()));
            }),
    );
    if let Some(delete) = on_delete {
        let label = if ix == 0 {
            "comments.delete_thread"
        } else {
            "comments.delete"
        };
        menu = menu.separator().item(
            MenuItem::new(i18n::text(cx, label, &[]))
                .icon(IconName::Trash2)
                .on_click(move |window, cx| {
                    log::info!("comment: delete {ix}");
                    delete(ix, window, cx)
                }),
        );
    }
    OverflowMenu::new(id, menu).tooltip(i18n::text(cx, "menu.more", &[]))
}

/// Opens comment `ix` in a field holding `body`, focused.
fn begin(open: &Entity<Open>, ix: usize, body: &SharedString, window: &mut Window, cx: &mut App) {
    let field = cx.new(|cx| {
        let mut input = TextInput::new(window, cx).multi_line(1, 8);
        input.set_text(body.to_string(), cx);
        input
    });
    let all = field.read(cx).text().len();
    field.update(cx, |input, cx| input.select(0..all, cx));
    window.focus(&field.read(cx).focus().clone(), cx);
    log::info!("comment: editing {ix}");
    open.update(cx, |open, cx| {
        *open = Some((ix, field));
        cx.notify();
    });
}

fn close(open: &Entity<Open>, cx: &mut App) {
    open.update(cx, |open, cx| {
        *open = None;
        cx.notify();
    });
}

/// Comment `ix` in `field` with Save and Cancel; Escape cancels, an unchanged text saves nothing.
pub(crate) fn field(
    id: ElementId,
    ix: usize,
    field: Entity<TextInput>,
    body: SharedString,
    open: Entity<Open>,
    on_edit: OnEdit,
    cx: &App,
) -> Div {
    let empty = field.read(cx).text().trim().is_empty();
    let (escape, cancel, save) = (open.clone(), open.clone(), open);
    let kept = field.clone();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .on_key_down(move |event, _, cx| {
            if event.keystroke.key == "escape" {
                cx.stop_propagation();
                log::info!("comment: edit cancelled");
                close(&escape, cx);
            }
        })
        .child(Input::new(&field).size(ControlSize::Sm))
        .child(
            div()
                .flex()
                .justify_end()
                .gap_2()
                .child(
                    Button::new((id.clone(), "cancel"), i18n::text(cx, "dialog.cancel", &[]))
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm)
                        .on_click(move |_, _, cx| {
                            cx.stop_propagation();
                            log::info!("comment: edit cancelled");
                            close(&cancel, cx);
                        }),
                )
                .child(
                    div()
                        .debug_selector(move || format!("comment-save-{ix}"))
                        .child(
                            Button::new((id, "save"), i18n::text(cx, "comments.save", &[]))
                                .variant(ButtonVariant::Primary)
                                .size(ControlSize::Sm)
                                .disabled(empty)
                                .on_click(move |_, window, cx| {
                                    cx.stop_propagation();
                                    let text = SharedString::from(kept.read(cx).text().to_string());
                                    close(&save, cx);
                                    if text == body {
                                        log::info!("comment: edit unchanged");
                                    } else {
                                        log::info!("comment: edited {ix}");
                                        on_edit(ix, &text, window, cx);
                                    }
                                }),
                        ),
                ),
        )
}
