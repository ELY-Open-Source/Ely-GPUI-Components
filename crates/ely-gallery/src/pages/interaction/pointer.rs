use ely_gpui_component::{
    interaction::{Resizable, Rotatable, SelectionArea},
    layout::{Transform, Viewport},
    motion::Reorder,
    primitives::{FocusRing, Pressable},
    theme::{ActiveTheme, Elevation, Radius, TextSize},
    typography::format::plural,
};
use gpui::{
    App, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, point, prelude::*, px, size,
};

use crate::{
    probe::probe,
    ui::{blocked, change, code, keep, section, specimen, specimens},
};

/// A chip on its way between lanes: its name and the lane it left.
#[derive(Clone)]
struct Chip {
    name: SharedString,
    from: usize,
}

/// What follows the pointer while a chip is dragged.
struct ChipPreview(SharedString);

impl Render for ChipPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .px_3()
            .py_1p5()
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border_strong)
            .bg(theme.colors.surface)
            .shadow(theme.elevation(Elevation::Floating))
            .text_size(theme.text_size(TextSize::Sm))
            .child(self.0.clone())
    }
}

const LANES: [&str; 2] = ["Today", "Later"];

/// Which lane each chip is in.
struct Board(Vec<(SharedString, usize)>);

pub fn drag_and_drop(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep(
        "interaction-board",
        || {
            Board(
                ["Design", "Build", "Ship"]
                    .map(|name| (name.into(), 0))
                    .to_vec(),
            )
        },
        window,
        cx,
    );
    let chips = state.read(cx).0.clone();
    let theme = cx.theme();
    let lanes = LANES.iter().enumerate().map(|(lane, title)| {
        let dropped = state.clone();
        let held = chips.iter().filter(|(_, at)| *at == lane).map(|(name, _)| {
            let chip = Chip {
                name: name.clone(),
                from: lane,
            };
            div()
                .id(SharedString::from(format!("interaction-chip-{name}")))
                .px_3()
                .py_1p5()
                .rounded(theme.radius(Radius::Md))
                .border_1()
                .border_color(theme.colors.border)
                .bg(theme.colors.surface)
                .cursor(gpui::CursorStyle::OpenHand)
                .text_size(theme.text_size(TextSize::Sm))
                .on_drag(chip, |chip, _, _, cx| {
                    cx.new(|_| ChipPreview(chip.name.clone()))
                })
                .child(name.clone())
        });
        let lit = theme.colors.hover;
        div()
            .id(SharedString::from(format!("interaction-lane-{lane}")))
            .w(px(180.))
            .min_h(px(140.))
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .can_drop(move |value, _, _| {
                value
                    .downcast_ref::<Chip>()
                    .is_some_and(|chip| chip.from != lane)
            })
            .drag_over::<Chip>(move |style, _, _, _| style.bg(lit))
            .on_drop(move |chip: &Chip, _, cx| {
                let name = chip.name.clone();
                change(&dropped, cx, |board| {
                    for (each, at) in board.0.iter_mut() {
                        if *each == name {
                            *at = lane;
                        }
                    }
                })
            })
            .child(
                div()
                    .text_size(theme.text_size(TextSize::Xs))
                    .text_color(theme.colors.fg_subtle)
                    .child(*title),
            )
            .children(held)
    });
    section(
        "Draggable · Droppable / DropTarget · DragOverlay / DragPreview · DragGhost",
        "gpui's own drag and drop: on_drag carries a value and builds the view that follows the pointer, drag_over lights a lane, can_drop refuses the lane a chip left, and on_drop takes it. Ely adds nothing.",
        cx,
    )
    .child(div().flex().flex_wrap().gap_4().children(lanes))
    .child(code("on_drag(value, |value, offset, window, cx| cx.new(…)) · drag_over::<T>(…) · can_drop(…) · on_drop(…)", cx))
}

/// The pointer demos' state.
struct Shapes {
    size: gpui::Size<gpui::Pixels>,
    angle: f32,
    picked: Vec<SharedString>,
    order: Vec<&'static str>,
    presses: usize,
    doubles: usize,
    held: usize,
}

pub fn shapes(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = keep(
        "interaction-shapes",
        || Shapes {
            size: size(px(260.), px(140.)),
            angle: 45.0,
            picked: vec!["tile-2".into()],
            order: vec!["Inbox", "Today", "Upcoming", "Someday"],
            presses: 0,
            doubles: 0,
            held: 0,
        },
        window,
        cx,
    );
    let now = state.read(cx);
    let (size_now, angle, picked, order) =
        (now.size, now.angle, now.picked.clone(), now.order.clone());
    let (presses, doubles, held) = (now.presses, now.doubles, now.held);
    let theme = cx.theme();
    let [resize, turn, pick, sort, double, press, long] = [(); 7].map(|_| state.clone());
    let tiles = (0..12).fold(
        SelectionArea::new("interaction-tiles", move |keys, _, cx| {
            let keys = keys.to_vec();
            change(&pick, cx, |shapes| shapes.picked = keys)
        })
        .w(px(300.))
        .flex()
        .flex_wrap()
        .gap(px(8.))
        .p_3()
        .rounded_lg()
        .border_1()
        .border_color(theme.colors.border),
        |area, ix| {
            area.item(
                SharedString::from(format!("tile-{ix}")),
                div().size(px(56.)).rounded_md().bg(theme.colors.sunken),
            )
        },
    );
    let list = order
        .iter()
        .fold(Reorder::new("interaction-sortable"), |list, name| {
            list.row(
                *name,
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.colors.border)
                    .bg(theme.colors.surface)
                    .child(*name),
            )
        });
    let moves = section(
        "Resizable · Rotatable · Selectable / SelectionArea · RubberBandSelection · Sortable",
        "A box resized from its edges or its grip; a picture turned by its knob, as gpui turns svgs alone; tiles taken by a band drawn on empty space; rows sorted by drag through motion::Reorder.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "Resizable",
                probe(
                    "interaction-resizable",
                    Resizable::new("interaction-resizable", size_now, size(px(160.), px(96.)), size(px(360.), px(220.)), move |next, _, cx| change(&resize, cx, |shapes| shapes.size = next))
                        .child(
                            div()
                                .size_full()
                                .p_4()
                                .rounded_lg()
                                .border_1()
                                .border_color(theme.colors.border)
                                .bg(theme.colors.surface)
                                .text_size(theme.text_size(TextSize::Sm))
                                .text_color(theme.colors.fg_muted)
                                .child(format!("{:.0} × {:.0}", f32::from(size_now.width), f32::from(size_now.height))),
                        ),
                ),
                cx,
            ))
            .child(specimen(
                "Rotatable",
                probe(
                    "interaction-rotatable",
                    Rotatable::new("interaction-rotatable", "icons/arrow-up.svg", px(120.), angle, move |next, _, cx| change(&turn, cx, |shapes| shapes.angle = next)),
                ),
                cx,
            )),
    )
    .child(
        specimens()
            .child(specimen(
                "SelectionArea",
                probe(
                    "interaction-tiles",
                    tiles.selected(picked),
                ),
                cx,
            ))
            .child(specimen(
                "Sortable → motion::Reorder",
                div().w(px(220.)).child(list.on_reorder(move |from, to, _, cx| {
                    change(&sort, cx, |shapes| {
                        let row = shapes.order.remove(from);
                        shapes.order.insert(to, row);
                    })
                })),
                cx,
            )),
    );
    let presses = section(
        "LongPress · DoubleClick · Gesture · PanZoom",
        "A long press is primitives::Pressable; a double press is gpui's click_count. The wheel pans and, with Command, zooms layout::Viewport; two fingers swipe lists::SwipeableListItem.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "LongPress → primitives::Pressable",
                Pressable::new("interaction-press", move |down, _, cx| {
                    let theme = cx.theme();
                    div()
                        .px_4()
                        .py_3()
                        .rounded_lg()
                        .border_1()
                        .border_color(theme.colors.border)
                        .bg(if down { theme.colors.active } else { theme.colors.surface })
                        .text_size(theme.text_size(TextSize::Sm))
                        .child(format!("Pressed {presses} · held {held}"))
                        .into_any_element()
                })
                .on_press(move |_, cx| change(&press, cx, |shapes| shapes.presses += 1))
                .on_long_press(move |_, cx| change(&long, cx, |shapes| shapes.held += 1)),
                cx,
            ))
            .child(specimen(
                "DoubleClick",
                div()
                    .id("interaction-double")
                    .px_4()
                    .py_3()
                    .rounded_lg()
                    .border_1()
                    .border_color(theme.colors.border)
                    .tab_index(0)
                    .focus_ring(cx)
                    .text_size(theme.text_size(TextSize::Sm))
                    .on_click(move |event, _, cx| {
                        if event.click_count() == 2 {
                            change(&double, cx, |shapes| shapes.doubles += 1)
                        }
                    })
                    .child(format!("Opened {}", plural(doubles as u64, "time", "times"))),
                cx,
            ))
            .child(specimen(
                "PanZoom → layout::Viewport",
                div().w(px(240.)).h(px(140.)).rounded_lg().border_1().border_color(theme.colors.border).overflow_hidden().child(
                    Viewport::new("interaction-viewport", |view: Transform, _, cx| {
                        let theme = cx.theme();
                        let corner = view.apply(point(px(80.), px(30.)));
                        div()
                            .absolute()
                            .left(corner.x)
                            .top(corner.y)
                            .size(px(80.) * view.scale)
                            .rounded_lg()
                            .bg(theme.colors.sunken)
                            .into_any_element()
                    }),
                ),
                cx,
            )),
    )
    .child(blocked("Pinch: gpui hands a trackpad pinch to on_pinch, and the viewport does not zoom on it yet.", cx));
    div().child(moves).child(presses)
}
