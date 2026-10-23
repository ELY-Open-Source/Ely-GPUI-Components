use ely_gpui_component::{
    buttons::{Button, ButtonVariant, IconButton},
    motion::{AnimatePresence, Entrance, Flip, Reorder, Stagger, Transition},
    primitives::{Icon, IconName},
    theme::{ActiveTheme, ControlSize, IconSize, Radius},
    typography::Caption,
};
use gpui::{App, Div, IntoElement, ParentElement, SharedString, Styled, Window, div};

use crate::{
    probe::probe,
    ui::{keep, row, section, set, specimen, specimens},
};

fn card(text: impl Into<SharedString>, cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .px_4()
        .py_3()
        .rounded(theme.radius(Radius::Lg))
        .border_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.surface)
        .child(text.into())
}

/// A row in a list demo: a hairline under it.
fn line(text: impl Into<SharedString>, cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .flex()
        .items_center()
        .gap_3()
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(theme.colors.border)
        .bg(theme.colors.surface)
        .child(text.into())
}

pub fn transitions(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let shown = keep("transition-shown", || true, window, cx);
    let (now, toggle) = (*shown.read(cx), shown.clone());
    let entrances = [
        (Entrance::Fade, "fade"),
        (Entrance::Rise, "rise"),
        (Entrance::Slide, "slide"),
    ];
    section(
        "Transition",
        "Content that fades, rises or slides in, and leaves the way it came.",
        cx,
    )
    .child(specimens().children(entrances.map(|(entrance, name)| {
        specimen(
            name,
            div().w_48().h_12().child(
                Transition::new(SharedString::from(format!("transition-{name}")), now)
                    .entrance(entrance)
                    .child(card("Hello", cx)),
            ),
            cx,
        )
    })))
    .child(
        row().child(probe(
            "transition-toggle",
            Button::new("transition-toggle", if now { "Hide" } else { "Show" })
                .variant(ButtonVariant::Ghost)
                .on_click(move |_, _, cx| set(&toggle, !now, cx)),
        )),
    )
}

pub fn stagger(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let turn = keep("stagger-turn", || 0usize, window, cx);
    let (now, replay) = (*turn.read(cx), turn.clone());
    section(
        "Stagger",
        "Children arrive one after another, a few frames apart.",
        cx,
    )
    .child(
        div().w_96().child(
            Stagger::new(SharedString::from(format!("stagger-{now}"))).children(
                ["Inbox", "Drafts", "Sent", "Archive", "Trash"].map(|name| line(name, cx)),
            ),
        ),
    )
    .child(
        row().child(probe(
            "stagger-replay",
            Button::new("stagger-replay", "Replay")
                .variant(ButtonVariant::Ghost)
                .on_click(move |_, _, cx| set(&replay, now + 1, cx)),
        )),
    )
}

/// A tag in the presence demo: a stable number, its name, and whether it shows.
type Tag = (usize, &'static str, bool);

pub fn presence(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    const MORE: [&str; 4] = ["Typography", "Color", "Layout", "Sound"];
    let tags = keep(
        "presence-tags",
        || vec![(0, "Design", true), (1, "Motion", true), (2, "Rust", true)],
        window,
        cx,
    );
    let list: Vec<Tag> = tags.read(cx).clone();
    let (add, gone) = (tags.clone(), tags.clone());
    let rows = list.iter().fold(
        AnimatePresence::new("presence").on_gone(move |key, _, cx| {
            let left: Vec<Tag> = gone
                .read(cx)
                .iter()
                .filter(|(number, _, _)| format!("tag-{number}") != key.as_ref())
                .copied()
                .collect();
            set(&gone, left, cx)
        }),
        |rows, (number, name, shown)| {
            let (hide, number) = (tags.clone(), *number);
            rows.row(
                SharedString::from(format!("tag-{number}")),
                *shown,
                line(*name, cx).justify_between().child(
                    IconButton::new(
                        SharedString::from(format!("presence-remove-{number}")),
                        IconName::X,
                    )
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Sm)
                    .on_click(move |_, _, cx| {
                        let then: Vec<Tag> = hide
                            .read(cx)
                            .iter()
                            .map(|tag| {
                                if tag.0 == number {
                                    (tag.0, tag.1, false)
                                } else {
                                    *tag
                                }
                            })
                            .collect();
                        set(&hide, then, cx)
                    }),
                ),
            )
        },
    );
    section(
        "AnimatePresence",
        "Rows fold open as they arrive and fold away as they go; the rest glide to close the gap.",
        cx,
    )
    .child(div().w_96().child(rows))
    .child(
        row().child(probe(
            "presence-add",
            Button::new("presence-add", "Add a tag")
                .variant(ButtonVariant::Ghost)
                .on_click(move |_, _, cx| {
                    let mut then = add.read(cx).clone();
                    let number = then.iter().map(|tag| tag.0 + 1).max().unwrap_or(0);
                    then.push((number, MORE[number % MORE.len()], true));
                    set(&add, then, cx);
                }),
        )),
    )
}

pub fn flip(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let order = keep(
        "flip-order",
        || vec!["Aurora", "Basalt", "Cinder", "Dune", "Ember"],
        window,
        cx,
    );
    let (now, shuffle) = (order.read(cx).clone(), order.clone());
    let list = now.iter().fold(Flip::new("flip"), |list, name| {
        list.row(*name, line(*name, cx))
    });
    section(
        "Flip",
        "When the order changes, each row glides from where it was to where it goes.",
        cx,
    )
    .child(div().w_96().child(list))
    .child(
        row().child(probe(
            "flip-shuffle",
            Button::new("flip-shuffle", "Shuffle")
                .variant(ButtonVariant::Ghost)
                .on_click(move |_, _, cx| {
                    let mut then = shuffle.read(cx).clone();
                    then.rotate_left(2);
                    then.swap(0, 3);
                    set(&shuffle, then, cx)
                }),
        )),
    )
}

pub fn reorder(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let tasks = keep(
        "reorder-tasks",
        || {
            vec![
                "Write the brief",
                "Sketch the flows",
                "Pick the type",
                "Build the prototype",
                "Test with five people",
            ]
        },
        window,
        cx,
    );
    let (now, moved) = (tasks.read(cx).clone(), tasks.clone());
    let muted = cx.theme().colors.fg_subtle;
    let list = now.iter().fold(
        Reorder::new("reorder").on_reorder(move |from, to, _, cx| {
            let mut then = moved.read(cx).clone();
            let task = then.remove(from);
            then.insert(to, task);
            set(&moved, then, cx)
        }),
        |list, task| {
            list.row(
                *task,
                line(*task, cx).child(div().flex_1()).child(
                    Icon::new(IconName::GripVertical)
                        .size(IconSize::Sm)
                        .color(muted),
                ),
            )
        },
    );
    section(
        "Reorder",
        "Drag a row: it follows the pointer and the others glide aside to make room.",
        cx,
    )
    .child(div().w_96().child(list))
    .child(Caption::new(
        "Motion is gpui's with_animation, timed by motion::duration and eased by the curves in motion. Spring is motion::spring. CountUp is typography::AnimatedNumber::count_up. A collapse is layout::Collapsible. A scale entrance is blocked: gpui transforms only svgs.",
    ))
}
