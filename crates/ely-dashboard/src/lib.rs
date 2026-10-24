//! Dashboard, chapter 34: cards on a grid, filters, refresh, alerts, incidents, events, service status and health checks.
#![deny(missing_docs)]

use ely_core::{buttons, data_display, forms, i18n, lists, primitives, theme, typography};

mod alerts;
mod card;
mod controls;
mod events;
mod grid;
mod status;
mod tiles;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use alerts::{
    AlertList, AlertState, Incident, IncidentCard, IncidentPhase, IncidentUpdate, MonitorAlert,
};
pub use card::DashboardCard;
pub use controls::{DashboardFilter, DashboardFilterBar, RefreshIntervalSelector, TimeWindow};
pub use events::{EventStream, StreamEvent};
pub use grid::DashboardGrid;
pub use status::{Check, Health, HealthCheck, Service, ServiceStatus, UptimeBar, overall, uptime};
pub use tiles::{Tile, arranged, stepped};
