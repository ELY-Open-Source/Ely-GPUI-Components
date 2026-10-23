//! Ely's charts for GPUI: lines, bars, areas, pies, scatter, flows, schedules and the axes under them.
#![deny(missing_docs)]

use ely_core::{buttons, motion, primitives, theme, typography};

mod axes;
mod burndown;
mod calendar;
mod cartesian;
mod distribution;
mod export;
mod flow;
mod funnel;
mod geometry;
mod glide;
mod layout;
mod network;
mod paint;
mod parallel;
mod parts;
mod pie;
mod plot;
mod pointer;
mod radar;
mod realtime;
mod relations;
mod scale;
mod schedule;
mod series;
mod stats;
mod sunburst;
mod targets;
#[cfg(all(test, feature = "test-support"))]
mod tests;
mod tiles;

pub use burndown::BurndownChart;
pub use calendar::CalendarHeatmap;
pub use cartesian::{AreaChart, BarChart, LineChart, ScatterChart};
pub use distribution::{BoxPlot, Histogram, ViolinPlot, WaterfallChart};
pub use flow::SankeyChart;
pub use funnel::FunnelChart;
pub use network::NetworkGraph;
pub use parallel::ParallelCoordinates;
pub use parts::{ChartLegend, ChartTooltip};
pub use pie::PieChart;
pub use radar::RadarChart;
pub use realtime::RealtimeChart;
pub use relations::ChordDiagram;
pub use schedule::{GanttChart, Task};
pub use series::LIMIT;
pub use series::{Points, Series};
pub use sunburst::{Slice, Sunburst};
pub use targets::{Bullet, BulletChart, ProgressChart};
pub use tiles::{HeatmapChart, Treemap};

pub use axes::anchored;
pub use geometry::{Dot, Geometry, Ink, Line, Rect};
pub use layout::squarify;
pub use paint::{at, finish, measure, place, ring, tint};
pub use plot::plot;
pub use plot::{Layout, Pick, Plot, Scene, Tips};
pub use scale::{Linear, compact, nice, nice_step};
pub use tiles::{Tiles, tracked};
