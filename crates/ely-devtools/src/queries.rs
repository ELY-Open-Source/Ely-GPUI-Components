use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, rems,
};
use jiff::Timestamp;

use crate::{
    forms::{Input, TextInput},
    layout::on_axis,
    lists::{ListItem, SelectableList},
    primitives::{Icon, IconName},
    tables::{Cell, Column, TreeRow, TreeTable},
    theme::{ActiveTheme, IconSize},
    typography::format,
};

/// How a query run ended: with so many rows, or failing.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// It gave so many rows.
    Rows(u64),
    /// It failed, and why.
    Failed(SharedString),
}

/// A query that ran: its key, its text, when it ran, how long it took in milliseconds, and how it ended.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryRun {
    /// Tells it apart from the others.
    pub key: SharedString,
    /// Its text.
    pub sql: SharedString,
    /// When it ran.
    pub at: Timestamp,
    /// How long it took, in milliseconds.
    pub took_ms: u32,
    /// How it ended.
    pub outcome: Outcome,
}

type OnKey = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// Queries that ran, newest first as given, found by their text. Each shows its first line, how it ended, how long it took and when; Enter or a double press asks to run one again.
///
/// ```
/// use ely_core::forms::TextInput;
/// use ely_devtools::{Outcome, QueryHistory, QueryRun};
/// use gpui::Entity;
/// use jiff::Timestamp;
///
/// fn history(search: &Entity<TextInput>) -> QueryHistory {
///     let now = Timestamp::from_second(1_792_746_000).unwrap();
///     let run = QueryRun {
///         key: "q1".into(),
///         sql: "select * from users".into(),
///         at: now,
///         took_ms: 12,
///         outcome: Outcome::Rows(42),
///     };
///     QueryHistory::new("history", [run], search, now).on_run(|_key, _, _| {})
/// }
/// ```
#[derive(IntoElement)]
pub struct QueryHistory {
    id: ElementId,
    runs: Vec<QueryRun>,
    search: gpui::Entity<TextInput>,
    now: Timestamp,
    on_run: Option<OnKey>,
}

impl QueryHistory {
    /// `search` is the find field's text, which the owner keeps; `now` dates each run.
    pub fn new(
        id: impl Into<ElementId>,
        runs: impl IntoIterator<Item = QueryRun>,
        search: &gpui::Entity<TextInput>,
        now: Timestamp,
    ) -> Self {
        Self {
            id: id.into(),
            runs: runs.into_iter().collect(),
            search: search.clone(),
            now,
            on_run: None,
        }
    }

    /// Gets the key of the query to run again.
    pub fn on_run(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_run = Some(Rc::new(handler));
        self
    }
}

/// A run's line under its text: rows or the failure, the time it took, and how long ago.
pub(crate) fn summary(run: &QueryRun, now: Timestamp) -> String {
    let ended = match &run.outcome {
        Outcome::Rows(rows) => format::plural(*rows, "row", "rows"),
        Outcome::Failed(why) => why.to_string(),
    };
    format!(
        "{ended} · {} ms · {}",
        run.took_ms,
        format::relative(run.at, now)
    )
}

impl RenderOnce for QueryHistory {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let query = self.search.read(cx).text().trim().to_lowercase();
        let theme = cx.theme();
        let (good, bad) = (theme.colors.success, theme.colors.danger);
        let id = self.id;
        let list = self
            .runs
            .iter()
            .filter(|run| run.sql.to_lowercase().contains(&query))
            .fold(SelectableList::new((id.clone(), "runs")), |list, run| {
                let first = run.sql.lines().next().unwrap_or_default().to_string();
                let (icon, ink) = match run.outcome {
                    Outcome::Rows(_) => (IconName::CircleCheck, good),
                    Outcome::Failed(_) => (IconName::CircleX, bad),
                };
                list.row(
                    run.key.clone(),
                    ListItem::new((id.clone(), format!("run-{}", run.key)), first)
                        .description(summary(run, self.now))
                        .leading(Icon::new(icon).size(IconSize::Sm).color(ink)),
                )
            });
        let on_run = self.on_run;
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Input::new(&self.search).prefix(Icon::new(IconName::Search).size(IconSize::Sm)))
            .child(list.on_activate(move |key, window, cx| {
                log::info!("query history: run {key} again");
                if let Some(on_run) = &on_run {
                    on_run(key, window, cx);
                }
            }))
    }
}

/// A step of a query plan: what it does and on what, its cost, the rows it yields, its time in milliseconds, and the steps it reads from.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanStep {
    /// Tells it apart from the others.
    pub key: SharedString,
    /// What it does, such as Seq Scan.
    pub op: SharedString,
    /// What it works on, such as a table.
    pub on: Option<SharedString>,
    /// Its cost; the root's is the whole plan's, which each bar divides.
    pub cost: f64,
    /// The rows it yields.
    pub rows: u64,
    /// Its time, in milliseconds.
    pub ms: f64,
    /// The steps it reads from.
    pub children: Vec<PlanStep>,
}

impl PlanStep {
    /// A step keyed `key` doing `op`, with its `cost`, `rows` and `ms`, reading from none.
    pub fn new(
        key: impl Into<SharedString>,
        op: impl Into<SharedString>,
        cost: f64,
        rows: u64,
        ms: f64,
    ) -> Self {
        Self {
            key: key.into(),
            op: op.into(),
            on: None,
            cost,
            rows,
            ms,
            children: Vec::new(),
        }
    }

    /// What it works on.
    pub fn on(mut self, target: impl Into<SharedString>) -> Self {
        self.on = Some(target.into());
        self
    }

    /// A step it reads from, after the others.
    pub fn child(mut self, step: PlanStep) -> Self {
        self.children.push(step);
        self
    }
}

fn row(step: &PlanStep, whole: f64) -> TreeRow {
    let name = match &step.on {
        Some(target) => format!("{} on {target}", step.op),
        None => step.op.to_string(),
    };
    TreeRow::new(
        step.key.clone(),
        [
            Cell::Text(name.into()),
            Cell::Progress((step.cost / whole) as f32),
            Cell::Number(step.rows as f64),
            Cell::Number(step.ms),
        ],
    )
    .children(step.children.iter().map(|child| row(child, whole)))
}

/// A query's plan as a tree of steps, each with its share of the whole cost drawn as a bar, the rows it yields and its time. Every step starts open; a box narrower than its columns scrolls sideways.
///
/// ```
/// use ely_devtools::{PlanStep, QueryPlanViewer};
///
/// let scan = PlanStep::new("scan", "Seq Scan", 120.0, 500, 3.2).on("users");
/// let plan = PlanStep::new("sort", "Sort", 150.0, 500, 4.0).child(scan);
/// let viewer = QueryPlanViewer::new("plan", plan);
/// ```
#[derive(IntoElement)]
pub struct QueryPlanViewer {
    id: ElementId,
    plan: PlanStep,
}

impl QueryPlanViewer {
    /// A viewer under `id` of `plan`; panics on a plan that costs nothing.
    pub fn new(id: impl Into<ElementId>, plan: PlanStep) -> Self {
        assert!(plan.cost > 0.0, "a plan costs something, not {}", plan.cost);
        Self {
            id: id.into(),
            plan,
        }
    }
}

fn keys(step: &PlanStep, out: &mut Vec<SharedString>) {
    out.push(step.key.clone());
    step.children.iter().for_each(|child| keys(child, out));
}

impl RenderOnce for QueryPlanViewer {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let wide = cx.theme().label_width();
        let mut open = Vec::new();
        keys(&self.plan, &mut open);
        let table = TreeTable::new(
            (self.id.clone(), "table"),
            [
                Column::new("step", "Step").width(wide),
                Column::new("cost", "Cost").unsorted(),
                Column::new("rows", "Rows").end(),
                Column::new("ms", "Time").end().decimals(1).suffix(" ms"),
            ],
            [row(&self.plan, self.plan.cost)],
        )
        .open(open);
        on_axis(div().id((self.id, "sideways")).overflow_x_scroll())
            .child(div().min_w(rems(wide.0 * 2.5)).child(table))
    }
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;

    use super::{Outcome, QueryRun, summary};

    #[test]
    fn a_run_says_how_it_ended_how_long_it_took_and_when() {
        let now: Timestamp = "2026-09-27T12:00:00Z".parse().expect("a time");
        let run = |outcome| QueryRun {
            key: "a".into(),
            sql: "select 1".into(),
            at: "2026-09-27T11:57:00Z".parse().expect("a time"),
            took_ms: 18,
            outcome,
        };
        assert_eq!(
            summary(&run(Outcome::Rows(1)), now),
            "1 row · 18 ms · 3 minutes ago"
        );
        assert_eq!(
            summary(&run(Outcome::Failed("relation missing".into())), now),
            "relation missing · 18 ms · 3 minutes ago"
        );
    }
}
