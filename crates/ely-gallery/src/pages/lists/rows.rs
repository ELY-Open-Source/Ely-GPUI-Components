use ely_gpui_component::{
    data_display::{Avatar, Badge, CountBadge, Tone},
    lists::{List, ListItem, SelectableList, SortableList, SwipeAction, SwipeableListItem},
    primitives::{Icon, IconName, Severity},
    theme::{ActiveTheme, AvatarSize, IconSize, Radius},
    typography::Caption,
};
use gpui::{App, ElementId, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

pub fn items(cx: &mut App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let folder = |icon| {
        Icon::new(icon)
            .size(IconSize::Md)
            .color(theme.colors.fg_muted)
    };
    section(
        "List / ListItem",
        "Rows with something to lead with, a title and a quieter line, and something at the end. Rows that can be pressed light on hover.",
        cx,
    )
    .child(
        div().w(px(480.)).child(
            List::new()
                .divided()
                .child(
                    ListItem::new("item-grace", "Grace Hopper")
                        .description("Design review: the cards finally breathe.")
                        .leading(Avatar::new("item-grace-face", "Grace Hopper").size(AvatarSize::Md))
                        .trailing("09:41")
                        .on_click(|_, _, _| log::info!("gallery: opened Grace's note")),
                )
                .child(
                    ListItem::new("item-inbox", "Inbox")
                        .leading(folder(IconName::Inbox))
                        .trailing(CountBadge::new("item-inbox-count", 12))
                        .on_click(|_, _, _| log::info!("gallery: opened the inbox")),
                )
                .child(
                    ListItem::new("item-drafts", "Drafts")
                        .description("Two unsent")
                        .leading(folder(IconName::FileText))
                        .trailing(Badge::new("Synced").tone(Severity::Success).dot()),
                )
                .child(
                    ListItem::new("item-archive", "Archive")
                        .description("Moving to cold storage")
                        .leading(folder(IconName::Archive))
                        .disabled(true),
                ),
        ),
    )
}

const FILES: [(&str, &str, &str); 5] = [
    ("brief", "Brief.pdf", "2.4 MB"),
    ("plan", "Q4 plan.key", "18 MB"),
    ("notes", "Interview notes.md", "12 KB"),
    ("budget", "Budget.numbers", "640 KB"),
    ("logo", "Logo.svg", "8 KB"),
];

pub fn selectable(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let picked = keep(
        "pick-selected",
        || vec![SharedString::from("plan")],
        window,
        cx,
    );
    let (now, store) = (picked.read(cx).clone(), picked.clone());
    let theme = cx.theme();
    let list = FILES.iter().enumerate().fold(
        SelectableList::new("pick").multiple().selected(now.clone()),
        |list, (ix, (key, name, size))| {
            let row = ListItem::new(("pick-item", ix), *name)
                .leading(
                    Icon::new(IconName::File)
                        .size(IconSize::Md)
                        .color(theme.colors.fg_muted),
                )
                .trailing(*size);
            list.row(*key, row)
        },
    );
    section(
        "SelectableList",
        "A press picks one; Cmd-press adds or drops one, Shift-press takes a range. With focus, Up and Down move, Shift extends, Space toggles and Cmd-A takes all.",
        cx,
    )
    .child(
        probe(
            "pick",
            div()
                .w(px(420.))
                .child(list.on_change(move |keys, _, cx| set(&store, keys.to_vec(), cx))),
        ),
    )
    .child(Caption::new(format!("{} selected: {}", now.len(), now.join(", "))))
}

pub fn sortable(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    const TRACKS: [&str; 5] = ["Morning light", "Harbour", "Low tide", "Dunes", "Evening"];
    let order = keep("sort-order", || TRACKS.to_vec(), window, cx);
    let (now, store) = (order.read(cx).clone(), order.clone());
    let list = now
        .iter()
        .enumerate()
        .fold(SortableList::new("sort"), |list, (ix, name)| {
            list.row(
                *name,
                ListItem::new(SharedString::from(format!("sort-{name}")), *name)
                    .trailing(format!("{}", ix + 1)),
            )
        });
    section(
        "SortableList / ReorderableList",
        "Drag a row to its new place, or focus the list and press Alt with Up or Down. The others glide aside.",
        cx,
    )
    .child(div().w(px(360.)).child(list.on_reorder(move |from, to, _, cx| {
        let mut next = store.read(cx).clone();
        let track = next.remove(from);
        next.insert(to, track);
        set(&store, next, cx)
    })))
}

pub fn swipeable(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let archived = keep("swipe-archived", || None::<&'static str>, window, cx);
    let (last, store) = (*archived.read(cx), archived.clone());
    let theme = cx.theme();
    let row = |id: &'static str, who: &'static str, what: &'static str| {
        let (archive, delete) = (store.clone(), store.clone());
        SwipeableListItem::new(
            id,
            ListItem::new((ElementId::from(id), "item"), who)
                .description(what)
                .leading(Avatar::new((ElementId::from(id), "face"), who).size(AvatarSize::Md)),
        )
        .action(SwipeAction::new(
            "Archive",
            IconName::Archive,
            move |_, cx| set(&archive, Some("Archived"), cx),
        ))
        .action(
            SwipeAction::new("Delete", IconName::Trash2, move |_, cx| {
                set(&delete, Some("Deleted"), cx)
            })
            .tone(Tone::Danger),
        )
    };
    section(
        "SwipeableListItem",
        "Swipe a row left with two fingers to show what can be done to it; it snaps open or shut as the fingers lift.",
        cx,
    )
    .child(
        div()
            .w(px(420.))
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .overflow_hidden()
            .child(probe(
                "swipe-row",
                div().w(px(418.)).child(row("swipe-ada", "Ada Lovelace", "The engine notes, annotated")),
            ))
            .child(div().border_t_1().border_color(theme.colors.border).child(row("swipe-alan", "Alan Turing", "On computable numbers"))),
    )
    .child(Caption::new(last.unwrap_or("Nothing done yet.")))
}
