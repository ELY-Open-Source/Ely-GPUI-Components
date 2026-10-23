use ely_gpui_component::{
    forms::{Cascade, Cascader, Choice, Combobox, ListBox, MultiSelect, Select, TransferList},
    primitives::IconName,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::text::field;
use crate::{
    probe::probe,
    ui::{blocked, keep, section, set, specimen, specimens},
};

fn cities() -> Vec<Choice> {
    [
        ("ams", "Amsterdam"),
        ("ber", "Berlin"),
        ("cph", "Copenhagen"),
        ("lis", "Lisbon"),
        ("lon", "London"),
        ("osl", "Oslo"),
        ("par", "Paris"),
        ("vie", "Vienna"),
    ]
    .into_iter()
    .map(|(value, label)| Choice::new(value, label))
    .collect()
}

pub fn select(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let view = keep("select-view", || SharedString::from("list"), window, cx);
    let now = view.read(cx).clone();
    section(
        "Select / NativeSelect",
        "A button that opens its list. Arrows move, Enter picks, Escape closes.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "view",
                probe(
                    "select",
                    div().w(px(240.0)).child(
                        Select::new(
                            "select-view",
                            [
                                Choice::new("list", "List").icon(IconName::List),
                                Choice::new("board", "Board").icon(IconName::Kanban),
                                Choice::new("calendar", "Calendar").icon(IconName::Calendar),
                                Choice::new("gallery", "Gallery")
                                    .icon(IconName::LayoutGrid)
                                    .disabled(),
                            ],
                        )
                        .selected(now)
                        .on_change(move |value, _, cx| set(&view, value.clone(), cx)),
                    ),
                ),
                cx,
            ))
            .child(specimen(
                "disabled",
                div().w(px(200.0)).child(
                    Select::new("select-off", [Choice::new("utc", "UTC")])
                        .selected("utc")
                        .disabled(true),
                ),
                cx,
            )),
    )
    .child(blocked(
        "NativeSelect: gpui has no native popup menu, so Select draws its own.",
        cx,
    ))
}

pub fn list_box(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let picked = keep(
        "listbox-cities",
        || vec![SharedString::from("lis")],
        window,
        cx,
    );
    let now = picked.read(cx).clone();
    section(
        "ListBox",
        "A list chosen from in place. Focus it and use the arrows, Space or Enter.",
        cx,
    )
    .child(
        div().w(px(240.0)).child(
            ListBox::new("listbox-cities", cities())
                .selected(now)
                .on_change(move |next, _, cx| set(&picked, next.to_vec(), cx)),
        ),
    )
}

pub fn combobox(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let city = field("input-city", window, cx, |input| {
        input.placeholder("Search cities")
    });
    let tag = field("input-tag", window, cx, |input| {
        input.placeholder("Any label")
    });
    let chosen = keep("combobox-city", || None::<SharedString>, window, cx);
    let now = chosen.read(cx).clone();
    let labels: Vec<Choice> = ["design", "docs", "infra", "motion", "release"]
        .into_iter()
        .map(|label| Choice::new(label, label))
        .collect();
    let mut combo = Combobox::new("combobox-city", &city, cities())
        .on_change(move |value, _, cx| set(&chosen, Some(value.clone()), cx));
    if let Some(value) = now {
        combo = combo.selected(value);
    }
    section(
        "Combobox / Autocomplete",
        "Type to filter. A combobox keeps a listed value; an autocomplete keeps any text.",
        cx,
    )
    .child(
        specimens()
            .child(specimen(
                "combobox",
                probe("combobox", div().w(px(260.0)).child(combo)),
                cx,
            ))
            .child(specimen(
                "autocomplete",
                div()
                    .w(px(260.0))
                    .child(Combobox::new("combobox-tag", &tag, labels).free()),
                cx,
            )),
    )
}

pub fn multi_select(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let picked = keep(
        "multi-cities",
        || vec![SharedString::from("ber"), SharedString::from("osl")],
        window,
        cx,
    );
    let now = picked.read(cx).clone();
    section(
        "MultiSelect",
        "Several values as chips. The list stays open while you pick; Backspace drops the last.",
        cx,
    )
    .child(probe(
        "multi",
        div().w(px(360.0)).child(
            MultiSelect::new("multi-cities", cities())
                .selected(now)
                .placeholder("Pick cities")
                .on_change(move |next, _, cx| set(&picked, next.to_vec(), cx)),
        ),
    ))
}

pub fn cascader(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let path = keep(
        "cascade-office",
        || {
            vec![
                SharedString::from("eu"),
                SharedString::from("pt"),
                SharedString::from("lis"),
            ]
        },
        window,
        cx,
    );
    let now = path.read(cx).clone();
    let world = [
        Cascade::new("am", "Americas")
            .child(
                Cascade::new("us", "United States")
                    .child(Cascade::new("nyc", "New York"))
                    .child(Cascade::new("sfo", "San Francisco")),
            )
            .child(Cascade::new("br", "Brazil").child(Cascade::new("sao", "São Paulo"))),
        Cascade::new("eu", "Europe")
            .child(
                Cascade::new("pt", "Portugal")
                    .child(Cascade::new("lis", "Lisbon"))
                    .child(Cascade::new("opo", "Porto")),
            )
            .child(Cascade::new("de", "Germany").child(Cascade::new("ber", "Berlin"))),
        Cascade::new("as", "Asia")
            .child(Cascade::new("jp", "Japan").child(Cascade::new("tyo", "Tokyo"))),
    ];
    section(
        "CascadeSelect / Cascader",
        "Each column opens the next. Right and Left walk the levels; a leaf picks the path.",
        cx,
    )
    .child(probe(
        "cascader",
        div().w(px(320.0)).child(
            Cascader::new("cascade-office", world)
                .selected(now)
                .on_change(move |next, _, cx| set(&path, next.to_vec(), cx)),
        ),
    ))
}

pub fn transfer(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let chosen = keep(
        "transfer-cities",
        || vec![SharedString::from("par")],
        window,
        cx,
    );
    let now = chosen.read(cx).clone();
    section(
        "TransferList",
        "Mark rows on one side, then move them across.",
        cx,
    )
    .child(probe(
        "transfer",
        div().w(px(560.0)).child(
            TransferList::new("transfer-cities", cities())
                .chosen(now)
                .titles("Cities", "On the tour")
                .on_change(move |next, _, cx| set(&chosen, next.to_vec(), cx)),
        ),
    ))
}
