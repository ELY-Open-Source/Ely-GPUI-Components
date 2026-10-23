use std::{collections::HashMap, rc::Rc};

use ely_gpui_component::{
    tables::{Aggregate, Cell, Column, DataGrid, PivotTable, Spreadsheet, TreeRow, TreeTable},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px, rems};

use crate::{
    probe::probe,
    ui::{keep, section, set},
};

type Cells = HashMap<(usize, usize), SharedString>;

pub fn data_grid(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let kept = keep(
        "grid-rows",
        || {
            [
                ["Rent", "2400", "2400", "2400"],
                ["Salaries", "18000", "18500", "19000"],
                ["Software", "1200", "1250", "1300"],
                ["Travel", "900", "400", "1600"],
            ]
            .map(|row| row.map(SharedString::from).to_vec())
            .to_vec()
        },
        window,
        cx,
    );
    let (rows, store) = (kept.read(cx).clone(), kept.clone());
    section(
        "DataGrid",
        "Cells to edit like a spreadsheet's. Arrows walk, Shift stretches a range, typing or a double press edits, Delete clears, and Cmd-C and Cmd-V copy and paste.",
        cx,
    )
    .child(probe(
        "grid",
        div().child(
            DataGrid::new("grid", ["Item", "Q1", "Q2", "Q3"], rows)
                .on_change(move |edits, _, cx| {
                    let mut next = store.read(cx).clone();
                    for (row, col, text) in edits {
                        next[*row][*col] = text.clone();
                    }
                    set(&store, next, cx)
                })
                .w(px(386.))
                .h(px(150.)),
        ),
    ))
}

fn budget() -> Cells {
    let mut cells = Cells::new();
    let mut put = |row: usize, col: usize, text: &str| {
        cells.insert((row, col), text.to_string().into());
    };
    put(0, 0, "Quarterly budget");
    for (col, title) in ["Item", "Q1", "Q2", "Q3", "Q4", "Total"].iter().enumerate() {
        put(1, col, title);
    }
    let items = [
        ("Rent", 2400.0),
        ("Salaries", 18000.0),
        ("Software", 1200.0),
        ("Travel", 900.0),
        ("Marketing", 3000.0),
    ];
    for (ix, (item, base)) in items.iter().enumerate() {
        let row = ix + 2;
        put(row, 0, item);
        for quarter in 0..4 {
            put(
                row,
                quarter + 1,
                &format!("{:.0}", base * (1.0 + 0.04 * quarter as f64)),
            );
        }
        put(row, 5, &format!("=SUM(B{0}:E{0})", row + 1));
    }
    put(7, 0, "Total");
    for col in 1..6 {
        let letter = (b'A' + col as u8) as char;
        put(7, col, &format!("=SUM({letter}3:{letter}7)"));
    }
    put(8, 0, "Share of rent");
    put(8, 5, "=F3/F8*100");
    cells
}

pub fn spreadsheet(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let kept = keep("sheet-cells", || Rc::new(budget()), window, cx);
    let (cells, store) = (kept.read(cx).clone(), kept.clone());
    section(
        "Spreadsheet",
        "Lettered columns and numbered rows. Formulas such as =SUM(B3:B7) recalculate as cells change; the title spans six merged cells; the first two rows and the first column stay put. Drag the dot at the range's corner to fill a series or carry a formula on.",
        cx,
    )
    .child(probe(
        "sheet",
        div().child(
            Spreadsheet::new("sheet", 40, 12)
                .cells(cells)
                .merge((0, 0), (0, 5))
                .freeze(2, 1)
                .on_change(move |edits, _, cx| {
                    let mut next = (**store.read(cx)).clone();
                    for (row, col, text) in edits {
                        if text.is_empty() {
                            next.remove(&(*row, *col));
                        } else {
                            next.insert((*row, *col), text.clone());
                        }
                    }
                    set(&store, Rc::new(next), cx)
                })
                .w(px(640.))
                .h(px(300.)),
        ),
    ))
}

pub fn tree_table(cx: &mut App) -> impl IntoElement + use<> {
    let folder = |key: &str, name: &str, size: f64, files: f64, children: Vec<TreeRow>| {
        TreeRow::new(
            key.to_string(),
            [name.to_string().into(), size.into(), files.into()],
        )
        .children(children)
    };
    let leaf = |key: &str, name: &str, size: f64| {
        TreeRow::new(
            key.to_string(),
            [name.to_string().into(), size.into(), 1.0.into()],
        )
    };
    let rows = [
        folder(
            "src",
            "src",
            842.0,
            64.0,
            vec![
                folder(
                    "ui",
                    "ui",
                    512.0,
                    38.0,
                    vec![
                        leaf("button", "button.rs", 18.0),
                        leaf("table", "table.rs", 41.0),
                    ],
                ),
                leaf("main", "main.rs", 6.0),
            ],
        ),
        folder(
            "assets",
            "assets",
            2480.0,
            12.0,
            vec![
                leaf("inter", "Inter.ttf", 804.0),
                leaf("mono", "JetBrains Mono.ttf", 268.0),
            ],
        ),
        leaf("cargo", "Cargo.toml", 2.0),
    ];
    section(
        "TreeTable",
        "Rows that open and close under columns; the first column steps in by depth.",
        cx,
    )
    .child(
        div().w(px(560.)).child(
            TreeTable::new(
                "sizes",
                [
                    Column::new("name", "Name"),
                    Column::new("size", "Size")
                        .end()
                        .suffix(" KB")
                        .width(rems(7.)),
                    Column::new("files", "Files").end().width(rems(5.)),
                ],
                rows,
            )
            .open(["src"]),
        ),
    )
}

pub fn pivot(cx: &mut App) -> impl IntoElement + use<> {
    let records: Vec<Vec<Cell>> = [
        ("Europe", "Q1", 120.0),
        ("Europe", "Q2", 132.0),
        ("Europe", "Q3", 128.0),
        ("Americas", "Q1", 180.0),
        ("Americas", "Q2", 171.0),
        ("Americas", "Q3", 199.0),
        ("Asia", "Q1", 90.0),
        ("Asia", "Q2", 118.0),
        ("Asia", "Q3", 141.0),
        ("Europe", "Q3", 22.0),
    ]
    .into_iter()
    .map(|(region, quarter, sales)| vec![region.into(), quarter.into(), sales.into()])
    .collect();
    section("PivotTable", "Sales cross-tabbed: regions down the side, quarters across, summed where they meet, with totals.", cx)
        .child(div().w(px(560.)).child(PivotTable::new(["Region", "Quarter", "Sales"], records).pivot("Region", "Quarter", "Sales", Aggregate::Sum)))
        .child(Caption::new("Sales in thousands."))
}
