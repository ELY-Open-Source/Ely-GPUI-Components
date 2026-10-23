use std::time::Duration;

use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    motion::{
        Shimmer, Skeleton, SkeletonAvatar, SkeletonCard, SkeletonTable, SkeletonText, Spinner,
        SpinnerStyle,
    },
    navigation::LoadMore,
    theme::{ActiveTheme, IconSize, Radius, TextSize},
    typography::Caption,
};
use gpui::{
    App, Entity, FontWeight, IntoElement, ParentElement, SharedString, Styled, Window, div,
};

use crate::{
    probe::probe,
    ui::{keep, row, section, set, specimen, specimens},
};

/// Sets `state` to `value` after `wait`, as a slow server would answer.
pub fn later<T: 'static>(state: &Entity<T>, value: T, wait: Duration, cx: &mut App) {
    let state = state.clone();
    cx.spawn(async move |cx| {
        cx.background_executor().timer(wait).await;
        cx.update(|cx| set(&state, value, cx));
    })
    .detach();
}

pub fn spinners(cx: &mut App) -> impl IntoElement + use<> {
    const STYLES: [(SpinnerStyle, &str); 6] = [
        (SpinnerStyle::Ring, "ring"),
        (SpinnerStyle::Dots, "dots"),
        (SpinnerStyle::Bars, "bars"),
        (SpinnerStyle::Pulse, "pulse"),
        (SpinnerStyle::Orbit, "orbit"),
        (SpinnerStyle::Wave, "wave"),
    ];
    const SIZES: [(IconSize, &str); 5] = [
        (IconSize::Xs, "xs"),
        (IconSize::Sm, "sm"),
        (IconSize::Md, "md"),
        (IconSize::Lg, "lg"),
        (IconSize::Xl, "xl"),
    ];
    section(
        "Spinner / DotsLoader / BarsLoader / PulseLoader / RingLoader / OrbitLoader / WaveLoader / InlineLoader",
        "Six small motions for waiting, one component. Under reduced motion each rests on one frame.",
        cx,
    )
    .child(specimens().children(STYLES.map(|(style, name)| {
        specimen(
            name,
            Spinner::new(SharedString::from(format!("spinner-{name}"))).style(style).size(IconSize::Xl),
            cx,
        )
    })))
    .child(specimens().children(SIZES.map(|(size, name)| {
        specimen(name, Spinner::new(SharedString::from(format!("spinner-size-{name}"))).size(size), cx)
    })))
}

pub fn skeletons(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "Skeleton / SkeletonText / SkeletonAvatar / SkeletonCard / SkeletonTable",
        "Placeholders in the shape of what is coming. They breathe together, slowly.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_start()
            .gap_6()
            .child(div().w_80().child(SkeletonCard::new("skeleton-card")))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(SkeletonAvatar::new("skeleton-avatar"))
                            .child(div().flex_1().child(SkeletonText::new("skeleton-text", 3))),
                    )
                    .child(SkeletonTable::new("skeleton-table", 3, 4))
                    .child(Skeleton::new("skeleton-block").w_full().h_16()),
            ),
    )
}

pub fn shimmer(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let loaded = keep("shimmer-loaded", || false, window, cx);
    let (now, toggle) = (*loaded.read(cx), loaded.clone());
    let theme = cx.theme();
    let card = if now {
        div()
            .w_80()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Quarterly review"),
            )
            .child(
                div()
                    .text_size(theme.text_size(TextSize::Sm))
                    .text_color(theme.colors.fg_muted)
                    .child("Revenue rose 12% on the quarter. Churn held at 2.1%; three new markets opened."),
            )
            .into_any_element()
    } else {
        Shimmer::new("shimmer")
            .w_80()
            .rounded(theme.radius(Radius::Lg))
            .child(SkeletonCard::new("shimmer-card"))
            .into_any_element()
    };
    section(
        "Shimmer",
        "A soft band of light sweeps what is loading. Load swaps in the content.",
        cx,
    )
    .child(card)
    .child(
        row().child(probe(
            "shimmer-load",
            Button::new("shimmer-load", if now { "Reload" } else { "Load" })
                .variant(ButtonVariant::Ghost)
                .on_click(move |_, _, cx| set(&toggle, !now, cx)),
        )),
    )
}

pub fn buttons(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let saving = keep("button-saving", || false, window, cx);
    let exporting = keep("button-exporting", || false, window, cx);
    let (save_now, export_now) = (*saving.read(cx), *exporting.read(cx));
    section(
        "LoadingButton",
        "A button spins in place at the same width and ignores presses until the work is done.",
        cx,
    )
    .child(
        row()
            .child(probe(
                "save",
                Button::new("save", "Save changes")
                    .primary()
                    .loading(save_now)
                    .on_click(move |_, _, cx| {
                        set(&saving, true, cx);
                        later(&saving, false, Duration::from_millis(1600), cx);
                    }),
            ))
            .child(
                Button::new("export", "Export")
                    .loading(export_now)
                    .on_click(move |_, _, cx| {
                        set(&exporting, true, cx);
                        later(&exporting, false, Duration::from_millis(2400), cx);
                    }),
            ),
    )
}

pub fn more(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    const PEOPLE: [&str; 18] = [
        "Ada Lovelace",
        "Alan Turing",
        "Grace Hopper",
        "Katherine Johnson",
        "Tim Berners-Lee",
        "Margaret Hamilton",
        "Donald Knuth",
        "Barbara Liskov",
        "Ken Thompson",
        "Radia Perlman",
        "Edsger Dijkstra",
        "Frances Allen",
        "Dennis Ritchie",
        "Hedy Lamarr",
        "John McCarthy",
        "Sophie Wilson",
        "Yukihiro Matsumoto",
        "Fei-Fei Li",
    ];
    let shown = keep("more-shown", || 5usize, window, cx);
    let busy = keep("more-busy", || false, window, cx);
    let (count, loading) = (*shown.read(cx), *busy.read(cx));
    let theme = cx.theme();
    section(
        "LoadMore",
        "A list that loads in pages ends in one button. It spins while the next page comes, and counts what shows.",
        cx,
    )
    .child(
        div()
            .w_96()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .children(PEOPLE[..count].iter().map(|name| {
                div()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .child(*name)
            }))
            .child(probe(
                "load-more",
                div().w_96().child(
                    LoadMore::new("more", loading)
                        .shown(count, PEOPLE.len())
                        .on_load(move |_, cx| {
                            set(&busy, true, cx);
                            later(&busy, false, Duration::from_millis(900), cx);
                            later(&shown, (count + 5).min(PEOPLE.len()), Duration::from_millis(900), cx);
                        }),
                ),
            )),
    )
    .child(Caption::new("Five at a time, eighteen in all."))
}
