use ely_gpui_component::{
    buttons::Button,
    misc::{CsvImporter, ExportDialog, ExportFormat, ImportDialog, ImportField},
    theme::ActiveTheme,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div};

use crate::probe::probe;
use crate::ui::section;

const PEOPLE: &str = "Name,Email,Team\nAda Lovelace,ada@example.com,Engines\n\"Lin, Wei\",lin@example.com,Maps\nGrace Hopper,grace@example.com,Compilers\n";

/// Which dialog is open.
#[derive(Clone, Copy, PartialEq)]
enum Open {
    None,
    Export,
    Import,
    Csv,
}

fn fields() -> [ImportField; 3] {
    [
        ImportField::new("name").required(),
        ImportField::new("email").required(),
        ImportField::new("phone"),
    ]
}

pub fn render(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = window.use_keyed_state("transfer-open", cx, |_, _| Open::None);
    let said = window.use_keyed_state("transfer-said", cx, |_, _| SharedString::default());
    let shut = |words: String,
                open: &gpui::Entity<Open>,
                said: &gpui::Entity<SharedString>,
                cx: &mut App| {
        open.update(cx, |open, cx| {
            *open = Open::None;
            cx.notify();
        });
        said.update(cx, |said, cx| {
            *said = words.into();
            cx.notify();
        });
    };
    let opener = |which: Open, label: &'static str, key: &'static str| {
        let open = open.clone();
        probe(
            key,
            div().child(Button::new(key, label).on_click(move |_, _, cx| {
                open.update(cx, |open, cx| {
                    *open = which;
                    cx.notify();
                })
            })),
        )
    };
    let (o, s) = (open.clone(), said.clone());
    let answer = move |words: String, cx: &mut App| shut(words, &o, &s, cx);
    let dialog = match *open.read(cx) {
        Open::None => None,
        Open::Export => {
            let (done, cancel) = (answer.clone(), answer.clone());
            Some(
                ExportDialog::new(
                    "export",
                    "Export the table",
                    [
                        ExportFormat::Pdf,
                        ExportFormat::Png,
                        ExportFormat::Csv,
                        ExportFormat::Json,
                    ],
                    move |format, _, cx| done(format!("Export as .{}", format.extension()), cx),
                    move |_, cx| cancel("Export cancelled.".into(), cx),
                )
                .into_any_element(),
            )
        }
        Open::Import => {
            let (done, cancel) = (answer.clone(), answer.clone());
            Some(
                ImportDialog::new(
                    "import",
                    "Import people",
                    ["Full name", "E-mail", "Team"],
                    fields(),
                    move |mapping, _, cx| done(format!("Columns by field: {mapping:?}"), cx),
                    move |_, cx| cancel("Import cancelled.".into(), cx),
                )
                .preview([
                    vec![
                        "Ada Lovelace".into(),
                        "ada@example.com".into(),
                        "Engines".into(),
                    ],
                    vec![
                        "Grace Hopper".into(),
                        "grace@example.com".into(),
                        "Compilers".into(),
                    ],
                ])
                .into_any_element(),
            )
        }
        Open::Csv => {
            let (done, cancel) = (answer.clone(), answer.clone());
            Some(
                CsvImporter::new(
                    "csv",
                    "Import people.csv",
                    PEOPLE,
                    fields(),
                    move |records, _, cx| done(format!("{} people imported.", records.len()), cx),
                    move |_, cx| cancel("CSV import cancelled.".into(), cx),
                )
                .into_any_element(),
            )
        }
    };
    div()
        .child(
            section(
                "ExportDialog / ImportDialog / CSVImporter",
                "Data out in a format the owner writes, and data in: columns mapped to the app's fields, matched by name to start, and a CSV file read into records through the same mapping.",
                cx,
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(opener(Open::Export, "Export…", "open-export"))
                    .child(opener(Open::Import, "Import…", "open-import"))
                    .child(opener(Open::Csv, "Import people.csv…", "open-csv")),
            )
            .child(
                div()
                    .text_color(cx.theme().colors.fg_muted)
                    .child(said.read(cx).clone()),
            ),
        )
        .children(dialog)
}
