use std::rc::Rc;

use ely_gpui_component::{
    devtools::{
        Outcome, PlanStep, QueryHistory, QueryPlanViewer, QueryRun, RedisKey, RedisKeyBrowser,
        RedisValue,
    },
    editor::CodeEditor,
    forms::TextInput,
    tables::DataGrid,
    theme::{ActiveTheme, Radius},
};
use gpui::{
    App, AppContext as _, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
};
use jiff::Timestamp;

use crate::ui::{change, keep, section};

const SQL: &str = "SELECT u.email, count(o.id) AS orders, sum(o.total) AS spent
FROM users u
JOIN orders o ON o.user_id = u.id
WHERE o.placed_at > now() - interval '30 days'
GROUP BY u.email
ORDER BY spent DESC
LIMIT 5;";

fn runs(now: Timestamp) -> Vec<QueryRun> {
    let ago = |minutes: i64| now - jiff::SignedDuration::from_mins(minutes);
    let run = |key: &str, sql: &str, minutes, took_ms, outcome| QueryRun {
        key: key.to_string().into(),
        sql: sql.to_string().into(),
        at: ago(minutes),
        took_ms,
        outcome,
    };
    vec![
        run(
            "r4",
            "SELECT u.email, count(o.id) AS orders",
            2,
            18,
            Outcome::Rows(5),
        ),
        run(
            "r3",
            "UPDATE orders SET total = total * 1.1 WHERE id = 42",
            9,
            4,
            Outcome::Rows(1),
        ),
        run(
            "r2",
            "SELECT * FROM order_item",
            14,
            2,
            Outcome::Failed("relation \"order_item\" does not exist".into()),
        ),
        run("r1", "SELECT count(*) FROM users", 61, 7, Outcome::Rows(1)),
    ]
}

fn plan() -> PlanStep {
    PlanStep::new("limit", "Limit", 1284.0, 5, 12.4).child(
        PlanStep::new("sort", "Sort", 1283.0, 212, 12.1).child(
            PlanStep::new("group", "Group", 1270.0, 212, 11.2).child(
                PlanStep::new("join", "Hash Join", 1150.0, 3480, 9.8)
                    .child(PlanStep::new("orders", "Seq Scan", 920.0, 3480, 7.6).on("orders"))
                    .child(PlanStep::new("users", "Index Scan", 180.0, 1200, 1.4).on("users_pkey")),
            ),
        ),
    )
}

/// The query demo's editor, result, history search and clock.
struct Console {
    editor: gpui::Entity<CodeEditor>,
    rows: Rc<Vec<Vec<SharedString>>>,
    search: gpui::Entity<TextInput>,
    now: Timestamp,
    runs: Vec<QueryRun>,
}

pub fn queries(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let console = window.use_keyed_state("devtools-console", cx, |window, cx| {
        let now: Timestamp = "2026-09-27T12:00:00Z".parse().expect("a time");
        let rows = [
            ["ada@example.com", "7", "1,284.00"],
            ["grace@example.com", "5", "962.50"],
            ["linus@example.com", "4", "711.20"],
            ["margaret@example.com", "4", "640.00"],
            ["alan@example.com", "3", "388.90"],
        ];
        Console {
            editor: cx.new(|cx| CodeEditor::new(SQL, window, cx).language("SQL")),
            rows: Rc::new(
                rows.iter()
                    .map(|row| row.iter().map(|cell| SharedString::from(*cell)).collect())
                    .collect(),
            ),
            search: cx.new(|cx| TextInput::new(window, cx).placeholder("Find a query")),
            now,
            runs: runs(now),
        }
    });
    let now = console.read(cx);
    let (editor, rows, search, clock, history) = (
        now.editor.clone(),
        now.rows.clone(),
        now.search.clone(),
        now.now,
        now.runs.clone(),
    );
    let again = console.clone();
    let theme = cx.theme();
    let boxed = |width: f32, height: f32| {
        div()
            .w(px(width))
            .h(px(height))
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .overflow_hidden()
    };
    section(
        "QueryEditor → editor::CodeEditor · QueryResultGrid → tables::DataGrid · QueryHistory · QueryPlanViewer",
        "A query in the code editor over its result in the data grid; the queries that ran, found by their text, each with how it ended, and one run again on Enter; and the plan, each step with its share of the cost.",
        cx,
    )
    .child(
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(boxed(760.0, 170.0).child(editor))
            .child(
                DataGrid::new("devtools-result", ["email", "orders", "spent"], rows)
                    .w(px(760.))
                    .h(px(170.)),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_6()
                    .child(
                        div().w(px(340.)).child(
                            QueryHistory::new("devtools-history", history, &search, clock).on_run(move |key, _, cx| {
                                change(&again, cx, |console| {
                                    let run = console.runs.iter().find(|run| run.key == *key).expect("a listed run").clone();
                                    console.runs.insert(0, QueryRun { key: format!("r{}", console.runs.len() + 1).into(), at: console.now, ..run });
                                })
                            }),
                        ),
                    )
                    .child(div().w(px(520.)).child(QueryPlanViewer::new("devtools-plan", plan()))),
            ),
    )
}

pub fn redis(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let pattern = window.use_keyed_state("devtools-redis-pattern", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Pattern, such as user:*")
    });
    let picked = keep(
        "devtools-redis-picked",
        || Some(SharedString::from("user:42")),
        window,
        cx,
    );
    let selected = picked.read(cx).clone();
    let keys = vec![
        RedisKey {
            key: "user:42".into(),
            value: RedisValue::Hash(vec![
                ("name".into(), "Ada".into()),
                ("plan".into(), "team".into()),
                ("seats".into(), "12".into()),
            ]),
            ttl: None,
        },
        RedisKey {
            key: "user:42:session".into(),
            value: RedisValue::Text("f3a9c1d2-77e0-4b1a".into()),
            ttl: Some(1800),
        },
        RedisKey {
            key: "queue:mail".into(),
            value: RedisValue::List(vec![
                "welcome:42".into(),
                "receipt:1009".into(),
                "digest:7".into(),
            ]),
            ttl: None,
        },
        RedisKey {
            key: "tags:popular".into(),
            value: RedisValue::Set(vec!["rust".into(), "gpui".into(), "design".into()]),
            ttl: None,
        },
        RedisKey {
            key: "board:weekly".into(),
            value: RedisValue::Sorted(vec![
                ("ada".into(), 1284.0),
                ("grace".into(), 962.5),
                ("alan".into(), 388.9),
            ]),
            ttl: Some(86_400),
        },
    ];
    section(
        "RedisKeyBrowser",
        "Keys found by a glob, each with its type and the time it has left; the key picked shows what it holds, laid out for its type.",
        cx,
    )
    .child(
        div().w(px(760.)).child(
            RedisKeyBrowser::new("devtools-redis", keys, &pattern)
                .selected(selected)
                .on_select(move |key, _, cx| change(&picked, cx, |picked| *picked = Some(key.clone()))),
        ),
    )
}
