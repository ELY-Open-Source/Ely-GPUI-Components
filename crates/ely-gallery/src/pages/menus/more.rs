use ely_gpui_component::{
    menus::{Menu, MenuBar, MenuItem, PieItem, PieMenu, SearchableMenu},
    primitives::IconName,
    theme::{ActiveTheme, Radius},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, Styled, Window, div, px};

use super::rows::{ran, run};
use crate::{
    probe::probe,
    ui::{section, specimen},
};

pub fn menu_bar(cx: &mut App) -> impl IntoElement + use<> {
    let menus = match cx.get_menus() {
        Some(menus) => menus,
        None if cfg!(target_family = "wasm") => crate::app_menus()
            .into_iter()
            .map(gpui::Menu::owned)
            .collect(),
        None => panic!("the gallery sets its menus in main"),
    };
    section(
        "MenuBar Menu",
        "The app's own menus, the ones cx.set_menus gave the system, drawn in the window. Picking a row dispatches its action.",
        cx,
    )
    .child(specimen(
        "hover moves between menus while one is open",
        probe("menubar", MenuBar::new("app-bar", menus)),
        cx,
    ))
}

pub fn mega_menu(cx: &App) -> impl IntoElement + use<> {
    section(
        "MegaMenu",
        "A menu of links wide enough for a line of detail each.",
        cx,
    )
    .child(Caption::new(
        "It is navigation::NavigationMenu: each entry opens a panel of links. See Navigation, chapter 7.",
    ))
}

const PLACES: [(&str, IconName); 8] = [
    ("Inbox", IconName::Inbox),
    ("Roadmap", IconName::Map),
    ("Design", IconName::PenTool),
    ("Engineering", IconName::Code),
    ("Research", IconName::BookOpen),
    ("Marketing", IconName::Globe),
    ("Archive", IconName::Archive),
    ("Trash", IconName::Trash2),
];

pub fn searchable(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (last, caption) = ran("move-ran", window, cx);
    let menu = PLACES.into_iter().fold(Menu::new(), |menu, (name, icon)| {
        menu.item(MenuItem::new(name).icon(icon).on_click(run(&last, name)))
    });
    section(
        "SearchableMenu",
        "A filter field heads the menu. Typing narrows the rows, best first, and marks the letters that matched.",
        cx,
    )
    .child(specimen(
        caption,
        probe(
            "move",
            SearchableMenu::new("move", "Move to", menu)
                .icon(IconName::FolderOpen)
                .placeholder("Find a place"),
        ),
        cx,
    ))
}

pub fn pie(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (last, caption) = ran("pie-ran", window, cx);
    let items = [
        (IconName::Copy, "Copy"),
        (IconName::Scissors, "Cut"),
        (IconName::Clipboard, "Paste"),
        (IconName::Share2, "Share"),
        (IconName::Pin, "Pin"),
        (IconName::Trash2, "Delete"),
    ]
    .map(|(icon, name)| PieItem::new(icon, name).on_click(run(&last, name)));
    let theme = cx.theme();
    section(
        "PieMenu / RadialMenu",
        "A right click opens a ring at the pointer. The pointer's direction marks a slice and a click runs it; arrows turn the ring, the hub cancels.",
        cx,
    )
    .child(specimen(
        caption,
        probe(
            "pie-area",
            PieMenu::new("pie", items).child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(420.0))
                    .h(px(260.0))
                    .border_1()
                    .border_dashed()
                    .border_color(theme.colors.border_strong)
                    .rounded(theme.radius(Radius::Lg))
                    .text_color(theme.colors.fg_subtle)
                    .child("Right-click for the ring"),
            ),
        ),
        cx,
    ))
}
