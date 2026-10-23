use std::time::Duration;

use ely_gpui_component::{
    data_display::Avatar,
    lists::{GroupedList, InfiniteList, ListItem, VirtualList},
    theme::{ActiveTheme, AvatarSize, Radius},
};
use gpui::{
    App, InteractiveElement, IntoElement, ParentElement, SharedString, StatefulInteractiveElement,
    Styled, Window, div, px,
};

use crate::ui::{keep, section, set};

fn framed(cx: &App) -> gpui::Div {
    let theme = cx.theme();
    div()
        .w(px(420.))
        .h(px(280.))
        .rounded(theme.radius(Radius::Md))
        .border_1()
        .border_color(theme.colors.border)
}

pub fn virtual_list(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "VirtualList",
        "Ten thousand rows, each as tall as it needs; only those near the view are built.",
        cx,
    )
    .child(
        framed(cx).child(
            VirtualList::new("long", 10_000, |ix, _, _| {
                let row = ListItem::new(("long-row", ix), format!("Row {}", ix + 1));
                let row = if ix % 7 == 0 {
                    row.description("Every seventh row carries a second line, so heights differ.")
                } else {
                    row
                };
                div().px_1().child(row).into_any_element()
            })
            .size_full(),
        ),
    )
}

/// Pages of twelve, five pages in all.
const PAGE: usize = 12;

pub fn infinite(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep("infinite", || (PAGE, false), window, cx);
    let ((rows, loading), more) = (*state.read(cx), state.clone());
    section(
        "InfiniteList",
        "Scroll to the end and the next page comes, a spinner holding its place while it loads. After five pages, the list ends quietly.",
        cx,
    )
    .child(
        framed(cx).child(
            div().id("infinite-scroll").size_full().overflow_y_scroll().child(
                InfiniteList::new("infinite")
                    .children((0..rows).map(|ix| ListItem::new(("infinite-row", ix), format!("Message {}", ix + 1))))
                    .loading(loading)
                    .done(rows >= PAGE * 5)
                    .on_more(move |_, cx| {
                        set(&more, (rows, true), cx);
                        let more = more.clone();
                        cx.spawn(async move |cx| {
                            cx.background_executor().timer(Duration::from_millis(700)).await;
                            cx.update(|cx| set(&more, (rows + PAGE, false), cx));
                        })
                        .detach();
                    }),
            ),
        ),
    )
}

const PEOPLE: [&str; 14] = [
    "Ada Lovelace",
    "Alan Kay",
    "Alan Turing",
    "Barbara Liskov",
    "Brian Kernighan",
    "Claude Shannon",
    "Dennis Ritchie",
    "Donald Knuth",
    "Edsger Dijkstra",
    "Frances Allen",
    "Grace Hopper",
    "Katherine Johnson",
    "Ken Thompson",
    "Margaret Hamilton",
];

pub fn grouped(cx: &mut App) -> impl IntoElement + use<> {
    let mut letters: Vec<char> = PEOPLE
        .iter()
        .filter_map(|name| name.chars().next())
        .collect();
    letters.dedup();
    let list = letters
        .into_iter()
        .fold(GroupedList::new("people"), |list, letter| {
            let rows = PEOPLE
                .iter()
                .filter(move |name| name.starts_with(letter))
                .map(|name| {
                    div().px_1().child(
                        ListItem::new(SharedString::from(format!("person-{name}")), *name).leading(
                            Avatar::new(SharedString::from(format!("face-{name}")), *name)
                                .size(AvatarSize::Sm),
                        ),
                    )
                });
            list.group(letter.to_string(), rows)
        });
    section(
        "GroupedList / SectionList",
        "Rows under their letter; each letter pins to the top while its group scrolls past.",
        cx,
    )
    .child(framed(cx).child(list.size_full()))
}
