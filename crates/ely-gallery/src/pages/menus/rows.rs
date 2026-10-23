use ely_gpui_component::{
    buttons::ButtonVariant,
    menus::{ContextMenu, DropdownMenu, Menu, MenuItem, OverflowMenu, SplitButton},
    primitives::IconName,
    theme::{ActiveTheme, Radius},
};
use gpui::{App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use crate::{
    probe::probe,
    ui::{keep, row, section, set, specimen},
};

/// Records the last row run, for the captions.
pub(super) fn ran(
    key: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> (Entity<Option<SharedString>>, String) {
    let last = keep(key, || None::<SharedString>, window, cx);
    let caption = last
        .read(cx)
        .clone()
        .map_or("nothing run yet".to_string(), |name| format!("ran {name}"));
    (last, caption)
}

pub(super) fn run(
    last: &Entity<Option<SharedString>>,
    name: &'static str,
) -> impl Fn(&mut Window, &mut App) + 'static {
    let last = last.clone();
    move |_, cx| set(&last, Some(name.into()), cx)
}

pub fn file_menu(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (last, caption) = ran("file-ran", window, cx);
    let recent = Menu::new()
        .item(
            MenuItem::new("roadmap.md")
                .icon(IconName::FileText)
                .on_click(run(&last, "roadmap.md")),
        )
        .item(
            MenuItem::new("tokens.rs")
                .icon(IconName::FileText)
                .on_click(run(&last, "tokens.rs")),
        )
        .separator()
        .item(MenuItem::new("Clear recent").on_click(run(&last, "clear recent")));
    let menu = Menu::new()
        .item(
            MenuItem::new("New file")
                .icon(IconName::FilePlus)
                .keys("secondary-n")
                .on_click(run(&last, "new file")),
        )
        .item(
            MenuItem::new("Open…")
                .icon(IconName::FolderOpen)
                .keys("secondary-o")
                .on_click(run(&last, "open")),
        )
        .item(MenuItem::submenu("Open recent", recent).icon(IconName::History))
        .separator()
        .item(
            MenuItem::new("Save")
                .icon(IconName::Save)
                .keys("secondary-s")
                .on_click(run(&last, "save")),
        )
        .item(
            MenuItem::new("Save as…")
                .keys("secondary-shift-s")
                .on_click(run(&last, "save as")),
        )
        .item(MenuItem::new("Revert").disabled(true))
        .group(
            "Share",
            [
                MenuItem::new("Copy link")
                    .icon(IconName::Link)
                    .on_click(run(&last, "copy link")),
                MenuItem::new("Export PDF")
                    .icon(IconName::Download)
                    .on_click(run(&last, "export")),
            ],
        );
    section(
        "Menu / MenuItem / MenuGroup / MenuSeparator / SubMenu / MenuItemShortcut / MenuItemIcon",
        "Rows with an icon and a key hint, split by rules and titled groups. The hint only shows a chord; the app binds it. A row with an arrow opens the next menu beside it; Right and Left walk in and out.",
        cx,
    )
    .child(specimen(
        caption,
        probe("file-menu", DropdownMenu::new("file", "File", menu).icon(IconName::FileText)),
        cx,
    ))
}

pub fn view_menu(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let shown = keep("view-shown", || [true, false, true], window, cx);
    let density = keep("view-density", || 1usize, window, cx);
    let (now, dense) = (*shown.read(cx), *density.read(cx));
    let flip = |ix: usize| {
        let shown = shown.clone();
        move |_: &mut Window, cx: &mut App| {
            let mut next = *shown.read(cx);
            next[ix] = !next[ix];
            set(&shown, next, cx);
        }
    };
    let pick = |ix: usize| {
        let density = density.clone();
        move |_: &mut Window, cx: &mut App| set(&density, ix, cx)
    };
    let menu = Menu::new()
        .item(
            MenuItem::check("Sidebar", now[0])
                .keys("secondary-b")
                .on_click(flip(0)),
        )
        .item(MenuItem::check("Minimap", now[1]).on_click(flip(1)))
        .item(MenuItem::check("Status bar", now[2]).on_click(flip(2)))
        .group(
            "Density",
            ["Compact", "Standard", "Comfortable"]
                .into_iter()
                .enumerate()
                .map(|(ix, name)| MenuItem::radio(name, dense == ix).on_click(pick(ix))),
        );
    let names = ["sidebar", "minimap", "status bar"];
    let on: Vec<&str> = names
        .iter()
        .zip(now)
        .filter(|(_, on)| *on)
        .map(|(name, _)| *name)
        .collect();
    section(
        "CheckboxMenuItem / RadioMenuItem",
        "A check flips on its own; a radio is one of its group. The owner holds both and redraws the menu.",
        cx,
    )
    .child(specimen(
        format!("{}; {}", on.join(", "), ["compact", "standard", "comfortable"][dense]),
        probe("view-menu", DropdownMenu::new("view", "View", menu).variant(ButtonVariant::Ghost)),
        cx,
    ))
}

pub fn hosts(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (last, caption) = ran("hosts-ran", window, cx);
    let merge = Menu::new()
        .item(MenuItem::new("Squash and merge").on_click(run(&last, "squash and merge")))
        .item(MenuItem::new("Rebase and merge").on_click(run(&last, "rebase and merge")));
    let more = Menu::new()
        .item(
            MenuItem::new("Rename")
                .icon(IconName::Pencil)
                .keys("f2")
                .on_click(run(&last, "rename")),
        )
        .item(
            MenuItem::new("Duplicate")
                .icon(IconName::Copy)
                .on_click(run(&last, "duplicate")),
        )
        .item(
            MenuItem::new("Archive")
                .icon(IconName::Archive)
                .on_click(run(&last, "archive")),
        )
        .separator()
        .item(
            MenuItem::new("Delete")
                .icon(IconName::Trash2)
                .on_click(run(&last, "delete")),
        );
    let share = Menu::new()
        .item(
            MenuItem::new("Copy link")
                .icon(IconName::Link)
                .on_click(run(&last, "copy link")),
        )
        .item(
            MenuItem::new("Email")
                .icon(IconName::Mail)
                .on_click(run(&last, "email")),
        );
    let main = last.clone();
    section(
        "DropdownMenu / SplitButton / OverflowMenu",
        "A button that opens a menu; a main action with related ones under its arrow; and More, for what did not fit.",
        cx,
    )
    .child(specimen(
        caption,
        row()
            .child(probe("share", DropdownMenu::new("share", "Share", share).icon(IconName::Share2)))
            .child(probe(
                "merge",
                SplitButton::new("merge", "Merge", merge)
                    .variant(ButtonVariant::Primary)
                    .on_click(move |_, _, cx| set(&main, Some("merge".into()), cx)),
            ))
            .child(probe("more", OverflowMenu::new("more", more))),
        cx,
    ))
}

pub fn context(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let (last, caption) = ran("context-ran", window, cx);
    let menu = Menu::new()
        .buttons([
            MenuItem::new("Cut")
                .icon(IconName::Scissors)
                .keys("secondary-x")
                .on_click(run(&last, "cut")),
            MenuItem::new("Copy")
                .icon(IconName::Copy)
                .keys("secondary-c")
                .on_click(run(&last, "copy")),
            MenuItem::new("Paste")
                .icon(IconName::Clipboard)
                .keys("secondary-v")
                .on_click(run(&last, "paste")),
            MenuItem::new("Delete")
                .icon(IconName::Trash2)
                .disabled(true),
        ])
        .separator()
        .item(
            MenuItem::new("Back")
                .icon(IconName::ArrowLeft)
                .keys("secondary-[")
                .on_click(run(&last, "back")),
        )
        .item(
            MenuItem::new("Forward")
                .icon(IconName::ArrowRight)
                .disabled(true),
        )
        .item(
            MenuItem::new("Reload")
                .icon(IconName::RotateCw)
                .keys("secondary-r")
                .on_click(run(&last, "reload")),
        )
        .separator()
        .item(
            MenuItem::new("Copy link")
                .icon(IconName::Link)
                .on_click(run(&last, "copy link")),
        )
        .item(
            MenuItem::new("Inspect")
                .icon(IconName::Code)
                .on_click(run(&last, "inspect")),
        );
    let theme = cx.theme();
    section(
        "ContextMenu",
        "A right click opens the menu where it lands; it stays inside the window. A row of icon buttons heads it; arrows cross the row.",
        cx,
    )
    .child(specimen(
        caption,
        probe(
            "canvas",
            ContextMenu::new("canvas", menu).child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(420.0))
                    .h(px(140.0))
                    .border_1()
                    .border_dashed()
                    .border_color(theme.colors.border_strong)
                    .rounded(theme.radius(Radius::Lg))
                    .text_color(theme.colors.fg_subtle)
                    .child("Right-click anywhere here"),
            ),
        ),
        cx,
    ))
}
