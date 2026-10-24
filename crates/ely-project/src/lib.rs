//! Project, chapter 31: tasks, issues, boards, databases, roadmaps, milestones, workloads and timers.
#![deny(missing_docs)]

use ely_calendar as calendar;
use ely_core::{
    buttons, data_display, forms, i18n, layout, lists, motion, overlays, primitives, tables, theme,
    typography,
};
use ely_documents as documents;
use ely_mail as mail;

mod board;
mod card;
mod database;
mod labels;
mod marks;
mod milestone;
mod moves;
mod panel;
mod pomodoro;
mod roadmap;
mod tasks;
mod tracker;
mod work;
mod workload;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use board::{KanbanBoard, KanbanCard, KanbanColumn};
pub use card::IssueCard;
pub use database::{DatabaseLayout, DatabaseView};
pub use labels::LabelManager;
pub use marks::{AssigneePicker, IssueIdBadge, PriorityIndicator, StatusSelect};
pub use milestone::{MilestoneProgress, Standing};
pub use panel::TaskDetailPanel;
pub use pomodoro::{Phase, PomodoroTimer};
pub use roadmap::{Initiative, Roadmap};
pub use tasks::{TaskItem, TaskList};
pub use tracker::{TimeEntry, TimeTracker};
pub use work::{IssueId, Person, Priority, Status, Task};
pub use workload::{Load, WorkloadView};
