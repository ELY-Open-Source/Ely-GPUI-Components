use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, relative,
};

use super::work::{IssueId, Person, Priority, Status};
use crate::{
    forms::{Choice, Select},
    i18n,
    primitives::{Icon, IconName, Tooltip},
    theme::{ActiveTheme, ControlSize, IconSize, TextSize},
    typography::tabular,
};

type OnStatus = Rc<dyn Fn(Status, &mut Window, &mut App)>;
type OnAssignee = Rc<dyn Fn(Option<&Person>, &mut Window, &mut App)>;

/// A priority at a glance: three bars, as many lit as it is high, or a red mark for urgent. Its words show in a tip, or beside it when labeled.
///
/// ```
/// use ely_project::{Priority, PriorityIndicator};
///
/// let mark = PriorityIndicator::new("priority", Priority::High).labeled();
/// ```
#[derive(IntoElement)]
pub struct PriorityIndicator {
    id: ElementId,
    priority: Priority,
    labeled: bool,
}

impl PriorityIndicator {
    /// A mark under `id` for `priority`, its words in a tip.
    pub fn new(id: impl Into<ElementId>, priority: Priority) -> Self {
        Self {
            id: id.into(),
            priority,
            labeled: false,
        }
    }

    /// Its words beside it.
    pub fn labeled(mut self) -> Self {
        self.labeled = true;
        self
    }
}

impl RenderOnce for PriorityIndicator {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let mark = match self.priority {
            Priority::Urgent => Icon::new(IconName::CircleAlert)
                .size(IconSize::Sm)
                .color(colors.danger)
                .into_any_element(),
            priority => div()
                .size(theme.icon_size(IconSize::Sm))
                .py_0p5()
                .flex()
                .items_end()
                .justify_center()
                .gap_0p5()
                .children([0.5, 0.75, 1.0].into_iter().enumerate().map(|(ix, tall)| {
                    let lit = ix < priority.bars();
                    div().w_0p5().h(relative(tall)).rounded_full().bg(if lit {
                        colors.fg_muted
                    } else {
                        colors.border_strong
                    })
                }))
                .into_any_element(),
        };
        let words = self.priority.words(cx);
        div()
            .id(self.id)
            .flex_none()
            .flex()
            .items_center()
            .gap_1p5()
            .child(mark)
            .map(|row| match self.labeled {
                true => row.child(
                    div()
                        .text_size(theme.text_size(TextSize::Sm))
                        .text_color(colors.fg_muted)
                        .child(words),
                ),
                false => row.tooltip(Tooltip::text(words)),
            })
    }
}

/// An issue's name, as ENG-123, quiet and in even figures.
///
/// ```
/// use ely_project::{IssueId, IssueIdBadge};
///
/// let badge = IssueIdBadge::new(IssueId::new("ENG", 123));
/// ```
#[derive(IntoElement)]
pub struct IssueIdBadge {
    issue: IssueId,
}

impl IssueIdBadge {
    /// The badge for `issue`.
    pub fn new(issue: IssueId) -> Self {
        Self { issue }
    }
}

impl RenderOnce for IssueIdBadge {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        tabular(div())
            .flex_none()
            .text_size(theme.text_size(TextSize::Sm))
            .text_color(theme.colors.fg_muted)
            .child(self.issue.to_string())
    }
}

/// Where a task stands, picked from its workflow, each status with its mark.
///
/// ```
/// use ely_project::{Status, StatusSelect};
///
/// let select = StatusSelect::new("status", Status::InProgress).on_change(|_status, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct StatusSelect {
    id: ElementId,
    status: Status,
    size: ControlSize,
    on_change: Option<OnStatus>,
}

impl StatusSelect {
    /// A select under `id` showing `status`.
    pub fn new(id: impl Into<ElementId>, status: Status) -> Self {
        Self {
            id: id.into(),
            status,
            size: ControlSize::default(),
            on_change: None,
        }
    }

    /// How large the trigger is.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Gets the status picked.
    pub fn on_change(mut self, handler: impl Fn(Status, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for StatusSelect {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let choices = Status::ALL
            .map(|status| Choice::new(status.key(), status.words(cx)).icon(status.icon()));
        let (id, on_change) = (self.id.clone(), self.on_change);
        Select::new(self.id, choices)
            .label(i18n::text(cx, "project.status", &[]))
            .selected(self.status.key())
            .size(self.size)
            .on_change(move |key: &SharedString, window, cx| {
                let status = Status::of_key(key);
                log::info!("status select {id:?}: {}", status.key());
                if let Some(on_change) = &on_change {
                    on_change(status, window, cx);
                }
            })
    }
}

/// Who has a task, picked from people by name, or no one.
///
/// ```
/// use ely_project::{AssigneePicker, Person};
///
/// let picker = AssigneePicker::new("assignee", [Person::new("ana", "Ana Silva"), Person::new("ben", "Ben Ito")])
///     .assignee("ana")
///     .on_change(|_person, _, _| {});
/// ```
#[derive(IntoElement)]
pub struct AssigneePicker {
    id: ElementId,
    people: Vec<Person>,
    assignee: Option<SharedString>,
    size: ControlSize,
    on_change: Option<OnAssignee>,
}

impl AssigneePicker {
    /// A picker under `id` over `people`, no one picked; panics on a key found twice.
    pub fn new(id: impl Into<ElementId>, people: impl IntoIterator<Item = Person>) -> Self {
        let people: Vec<Person> = people.into_iter().collect();
        for (ix, person) in people.iter().enumerate() {
            let twice = people[..ix].iter().any(|other| other.key == person.key);
            assert!(!twice, "person {} twice", person.key);
        }
        Self {
            id: id.into(),
            people,
            assignee: None,
            size: ControlSize::default(),
            on_change: None,
        }
    }

    /// Who has it now, by key.
    pub fn assignee(mut self, key: impl Into<SharedString>) -> Self {
        self.assignee = Some(key.into());
        self
    }

    /// How large the trigger is.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Gets the person picked, or none for no one.
    pub fn on_change(
        mut self,
        handler: impl Fn(Option<&Person>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for AssigneePicker {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(key) = &self.assignee {
            let known = self.people.iter().any(|person| person.key == *key);
            if !known {
                log::error!(
                    "assignee picker {:?}: no person {key}; nobody shown",
                    self.id
                );
            }
        }
        let nobody = Choice::new("", "No assignee").icon(IconName::User);
        let choices: Vec<Choice> = std::iter::once(nobody)
            .chain(self.people.iter().map(|person| {
                Choice::new(person.key.clone(), person.name.clone()).avatar(person.name.clone())
            }))
            .collect();
        let (id, people, on_change) = (self.id.clone(), self.people, self.on_change);
        Select::new(self.id, choices)
            .label(i18n::text(cx, "project.assignee", &[]))
            .selected(self.assignee.unwrap_or_default())
            .size(self.size)
            .on_change(move |key: &SharedString, window, cx| {
                let person = people.iter().find(|person| person.key == *key);
                log::info!(
                    "assignee picker {id:?}: {}",
                    person.map_or("no one", |person| person.name.as_ref())
                );
                if let Some(on_change) = &on_change {
                    on_change(person, window, cx);
                }
            })
    }
}

ely_core::capable!(StatusSelect: size, change(Status));
ely_core::capable!(AssigneePicker: size);

impl ely_core::primitives::Changeable for AssigneePicker {
    type Value<'a> = Option<&'a Person>;

    fn on_change(self, handler: impl Fn(Option<&Person>, &mut Window, &mut App) + 'static) -> Self {
        AssigneePicker::on_change(self, handler)
    }
}
