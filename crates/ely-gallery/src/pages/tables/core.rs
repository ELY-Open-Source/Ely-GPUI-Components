use std::rc::Rc;

use ely_gpui_component::{
    data_display::Tone,
    forms::TextInput,
    tables::{
        Aggregate, Cell, Column, DataTable, FilterBuilder, FilterRule, HeatmapTable, Row,
        SortBuilder, SortKey, Table, TableToolbar, Test, to_csv,
    },
    theme::{ActiveTheme, Density, Radius},
    typography::Caption,
};
use gpui::{
    App, ClipboardItem, IntoElement, ParentElement, SharedString, Styled, Window, div, px, rems,
};

use crate::{
    probe::probe,
    ui::{keep, noise, section, set},
};

pub fn plain(cx: &mut App) -> impl IntoElement + use<> {
    section(
        "Table",
        "A header and rows, hairlines between. Nothing more.",
        cx,
    )
    .child(
        div().w(px(560.)).child(
            Table::new(["Name", "Role", "City"])
                .row(["Ada Lovelace", "Analyst", "London"])
                .row(["Grace Hopper", "Engineer", "Arlington"])
                .row(["Katherine Johnson", "Mathematician", "Hampton"]),
        ),
    )
}

const PEOPLE: [&str; 10] = [
    "Ada Lovelace",
    "Alan Turing",
    "Grace Hopper",
    "Katherine Johnson",
    "Tim Berners-Lee",
    "Radia Perlman",
    "Sophie Wilson",
    "Barbara Liskov",
    "Ken Thompson",
    "Margaret Hamilton",
];
const REGIONS: [&str; 4] = ["Europe", "Americas", "Asia", "Oceania"];

pub(super) fn order_rows() -> Vec<Row> {
    let mut next = noise(7);
    (0..24)
        .map(|ix| {
            let (status, tone) = match (next() * 3.0) as u32 {
                0 => ("Live", Tone::Success),
                1 => ("Draft", Tone::Neutral),
                _ => ("Review", Tone::Warning),
            };
            let spark: Vec<f32> = (0..8).map(|_| next() as f32).collect();
            Row::new(
                format!("order-{ix}"),
                [
                    Cell::Person(PEOPLE[ix % PEOPLE.len()].into()),
                    Cell::Tag(status.into(), tone),
                    REGIONS[(next() * 4.0) as usize].into(),
                    (next() * 90_000.0 + 8_000.0).round().into(),
                    ((next() - 0.35) * 40.0).into(),
                    Cell::Spark(spark),
                    Cell::Progress(next() as f32),
                ],
            )
        })
        .collect()
}

pub(super) fn order_columns() -> [Column; 7] {
    [
        Column::new("owner", "Owner"),
        Column::new("status", "Status").width(rems(6.)),
        Column::new("region", "Region").width(rems(6.)),
        Column::new("revenue", "Revenue")
            .end()
            .prefix("$")
            .aggregate(Aggregate::Sum)
            .width(rems(8.)),
        Column::new("growth", "Growth")
            .end()
            .decimals(1)
            .suffix("%")
            .scale()
            .aggregate(Aggregate::Average)
            .width(rems(6.)),
        Column::new("trend", "Trend").unsorted().width(rems(6.)),
        Column::new("quota", "Quota")
            .width(rems(9.))
            .aggregate(Aggregate::Average),
    ]
}

pub fn orders(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let search = window.use_keyed_state("orders-search", cx, TextInput::new);
    let query = search.read(cx).text().to_string();
    let picked = keep("orders-picked", Vec::<SharedString>::new, window, cx);
    let spacing = keep("orders-density", || Density::Standard, window, cx);
    let hidden = keep("orders-hidden", Vec::<SharedString>::new, window, cx);
    let panel = keep("orders-panel", || false, window, cx);
    let rules = keep(
        "orders-rules",
        || {
            (
                vec![FilterRule {
                    column: "status".into(),
                    test: Test::Contains,
                    value: SharedString::default(),
                }],
                false,
            )
        },
        window,
        cx,
    );
    let sorts = keep(
        "orders-sorts",
        || {
            vec![SortKey {
                column: "region".into(),
                rising: true,
            }]
        },
        window,
        cx,
    );
    let copied = keep("orders-copied", || None::<usize>, window, cx);
    let (chosen, dense, gone, open, (now_rules, any), now_sorts, told) = (
        picked.read(cx).clone(),
        *spacing.read(cx),
        hidden.read(cx).clone(),
        *panel.read(cx),
        rules.read(cx).clone(),
        sorts.read(cx).clone(),
        *copied.read(cx),
    );
    let titles: Vec<(SharedString, SharedString)> = order_columns()
        .iter()
        .map(|column| (column.key().clone(), column.title().clone()))
        .collect();
    let theme = cx.theme();
    let toolbar = TableToolbar::new("orders-tools")
        .search(&search)
        .filter(now_rules.len(), {
            let panel = panel.clone();
            move |_, cx| set(&panel, !open, cx)
        })
        .columns(titles.clone(), gone.clone(), {
            let hidden = hidden.clone();
            move |keys, _, cx| set(&hidden, keys.to_vec(), cx)
        })
        .density(dense, {
            let spacing = spacing.clone();
            move |density, _, cx| set(&spacing, density, cx)
        })
        .export({
            let copied = copied.clone();
            move |_, cx| {
                let rows = order_rows();
                cx.write_to_clipboard(ClipboardItem::new_string(to_csv(&order_columns(), &rows)));
                set(&copied, Some(rows.len()), cx)
            }
        });
    let builders = open.then(|| {
        let (store_rules, store_sorts) = (rules.clone(), sorts.clone());
        div()
            .flex()
            .gap_8()
            .p_4()
            .mb_3()
            .rounded(theme.radius(Radius::Md))
            .bg(theme.colors.sunken)
            .child(
                div().flex_1().child(
                    FilterBuilder::new("orders-filter", titles.clone())
                        .rules(now_rules.clone(), any)
                        .on_change(move |rules, any, _, cx| {
                            set(&store_rules, (rules.to_vec(), any), cx)
                        }),
                ),
            )
            .child(
                div().w(px(380.)).child(
                    SortBuilder::new("orders-sort", titles.clone())
                        .keys(now_sorts.clone())
                        .on_change(move |keys, _, cx| set(&store_sorts, keys.to_vec(), cx)),
                ),
            )
    });
    let store = picked.clone();
    section(
        "DataTable / TableToolbar / FilterBuilder / SortBuilder / ColumnVisibility / CellRenderer / RowSelection / TableDensity / AggregationFooter / ConditionalFormatting",
        "Press a header to sort, Shift to add it to the sort. The toolbar searches, opens the filter and sort builders, hides columns, sets the density and copies the rows as CSV. Boxes select; Shift takes a range. Growth is tinted by where it sits; the footer sums and averages what the filters keep.",
        cx,
    )
    .child(div().pb_3().child(probe("orders-tools", div().w(px(900.)).child(toolbar))))
    .children(builders.map(|panel| div().w(px(900.)).child(panel)))
    .child(probe(
        "orders",
        div().w(px(900.)).child(
            DataTable::new("orders", order_columns())
                .rows(order_rows())
                .query(query)
                .filters(now_rules, any)
                .sorts(now_sorts)
                .hidden(gone)
                .paged(8)
                .density(dense)
                .selected(chosen.clone())
                .on_select(move |keys, _, cx| set(&store, keys.to_vec(), cx)),
        ),
    ))
    .child(Caption::new(match told {
        Some(rows) => format!("{} selected. Copied {rows} rows as CSV.", chosen.len()),
        None => format!("{} selected", chosen.len()),
    }))
}

pub fn long(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let rows = keep(
        "long-rows",
        || {
            let mut next = noise(11);
            Rc::new(
                (0..100_000)
                    .map(|ix| {
                        Row::new(
                            format!("long-{ix}"),
                            [
                                format!("Event {}", ix + 1).into(),
                                REGIONS[ix % 4].into(),
                                (next() * 1000.0).round().into(),
                            ],
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        },
        window,
        cx,
    )
    .read(cx)
    .clone();
    let theme = cx.theme();
    section(
        "VirtualTable",
        "A hundred thousand rows; only the ones in view are drawn, so it scrolls as lightly as ten.",
        cx,
    )
    .child(
        div()
            .w(px(560.))
            .h(px(300.))
            .rounded(theme.radius(Radius::Md))
            .border_1()
            .border_color(theme.colors.border)
            .child(
                DataTable::new(
                    "long",
                    [
                        Column::new("event", "Event"),
                        Column::new("region", "Region"),
                        Column::new("value", "Value").end().aggregate(Aggregate::Sum),
                    ],
                )
                .rows(rows)
                .virtualized()
                .density(Density::Compact)
                .size_full(),
            ),
    )
}

pub fn heatmap(cx: &mut App) -> impl IntoElement + use<> {
    let mut next = noise(3);
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let table = days.iter().enumerate().fold(
        HeatmapTable::new(["6", "9", "12", "15", "18", "21"]),
        |table, (day, name)| {
            let busy = if day >= 5 { 0.5 } else { 1.0 };
            table.row(
                *name,
                (0..6).map(|slot| {
                    (busy
                        * (40.0 + 60.0 * next())
                        * if (1..=4).contains(&slot) { 1.6 } else { 0.7 })
                    .round()
                }),
            )
        },
    );
    section(
        "HeatmapTable",
        "Visits by weekday and hour; each cell is shaded by where it sits between the quietest and the busiest.",
        cx,
    )
    .child(div().w(px(560.)).child(table))
    .child(Caption::new("ComparisonTable is Comparison, in Data Display."))
}
