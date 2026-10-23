//! Each vertical chapter's measures.

use gpui::{Pixels, Rems, px};

use super::{Theme, tokens::px_to_rems};

/// A chart's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartSizes {
    /// A chart's height, legend and axes included.
    pub height: Rems,
    /// Room beside a plot for its value labels.
    pub gutter: Rems,
    /// Room under a plot for its category labels.
    pub foot: Rems,
    /// Room a plot keeps above and to its right, so dots and strokes stay whole.
    pub inset: Rems,
    /// The least room a category label takes along its axis; closer labels thin out.
    pub label: Rems,
    /// A line series' width.
    pub stroke: Rems,
    /// Widest a bubble's radius grows.
    pub bubble: Rems,
    /// A market chart's room for each candle before any zoom.
    pub candle: Rems,
    /// How far a slice under the pointer lifts out.
    pub lift: Rems,
    /// Gridlines and rules.
    pub hairline: Pixels,
}

impl Theme {
    /// A chart's measures.
    pub fn chart(&self) -> ChartSizes {
        ChartSizes {
            height: px_to_rems(280.0),
            gutter: px_to_rems(48.0),
            foot: px_to_rems(28.0),
            inset: px_to_rems(8.0),
            label: px_to_rems(64.0),
            stroke: px_to_rems(2.0),
            bubble: px_to_rems(24.0),
            candle: px_to_rems(8.0),
            lift: px_to_rems(6.0),
            hairline: px(1.0),
        }
    }
}

/// Generative tools' measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GenerativeSizes {
    /// A style preset's picture, square.
    pub preset: Rems,
    /// A second of video along a timeline.
    pub second: Rems,
    /// A shot's height on a timeline.
    pub shot: Rems,
    /// A loss chart's height on a job's card.
    pub loss: Rems,
}

impl Theme {
    /// Generative tools' measures.
    pub fn generative(&self) -> GenerativeSizes {
        GenerativeSizes {
            preset: px_to_rems(88.0),
            second: px_to_rems(24.0),
            shot: px_to_rems(56.0),
            loss: px_to_rems(140.0),
        }
    }
}

/// Media tools' measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MediaSizes {
    /// A crop box's handle, square.
    pub crop_handle: Rems,
    /// A mark's line on a picture.
    pub mark_stroke: Rems,
    /// An arrow's head, from its tip.
    pub arrow_head: Rems,
    /// A numbered pin, round.
    pub pin: Rems,
    /// A strip of video frames, tall.
    pub strip: Rems,
    /// A trim's handle, wide.
    pub trim_handle: Rems,
    /// A waveform's bar, wide.
    pub wave_bar: Rems,
    /// The gap between a waveform's bars.
    pub wave_gap: Rems,
    /// A waveform or a spectrum, tall.
    pub waveform: Rems,
    /// A song's cover, square.
    pub artwork: Rems,
}

impl Theme {
    /// Media tools' measures.
    pub fn media(&self) -> MediaSizes {
        MediaSizes {
            crop_handle: px_to_rems(10.0),
            mark_stroke: px_to_rems(2.0),
            arrow_head: px_to_rems(12.0),
            pin: px_to_rems(20.0),
            strip: px_to_rems(48.0),
            trim_handle: px_to_rems(8.0),
            wave_bar: px_to_rems(3.0),
            wave_gap: px_to_rems(2.0),
            waveform: px_to_rems(48.0),
            artwork: px_to_rems(64.0),
        }
    }
}

/// File views' measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileSizes {
    /// A tile in a grid of files, wide.
    pub tile: Rems,
    /// A column of a folder along a path, wide.
    pub column: Rems,
    /// The columns view, tall.
    pub columns: Rems,
}

impl Theme {
    /// File views' measures.
    pub fn files(&self) -> FileSizes {
        FileSizes {
            tile: px_to_rems(96.0),
            column: px_to_rems(200.0),
            columns: px_to_rems(288.0),
        }
    }
}

/// Messaging's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MessagingSizes {
    /// A call's strip of tiles beside a shared screen, wide.
    pub tiles: Rems,
    /// A call's tile at its narrowest, before a grid drops a column.
    pub tile: Rems,
    /// A sticker's cell in a picker, square.
    pub sticker: Rems,
}

impl Theme {
    /// Messaging's measures.
    pub fn messaging(&self) -> MessagingSizes {
        MessagingSizes {
            tiles: px_to_rems(200.0),
            tile: px_to_rems(160.0),
            sticker: px_to_rems(64.0),
        }
    }
}

/// Calendar's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalendarSizes {
    /// An event's chip in a day, tall.
    pub chip: Rems,
    /// Rows of events a month's day holds under its number.
    pub day_rows: usize,
    /// A day in a year's small months, square.
    pub mini_day: Rems,
    /// An hour in a time grid, tall.
    pub hour: Rems,
    /// A time grid's column of hours, wide.
    pub gutter: Rems,
    /// An event's details in a panel, wide.
    pub card: Rems,
}

impl Theme {
    /// Calendar's measures.
    pub fn calendar(&self) -> CalendarSizes {
        CalendarSizes {
            chip: px_to_rems(20.0),
            day_rows: 3,
            mini_day: px_to_rems(28.0),
            hour: px_to_rems(48.0),
            gutter: px_to_rems(56.0),
            card: px_to_rems(320.0),
        }
    }
}

/// Project's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectSizes {
    /// A board's column, wide.
    pub column: Rems,
    /// A roadmap's month at least, wide.
    pub month: Rems,
    /// A pomodoro's ring, wide.
    pub pomodoro: Rems,
}

impl Theme {
    /// Project's measures.
    pub fn project(&self) -> ProjectSizes {
        ProjectSizes {
            column: px_to_rems(272.0),
            month: px_to_rems(72.0),
            pomodoro: px_to_rems(176.0),
        }
    }
}

/// Canvas measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasSizes {
    /// The top ruler, tall.
    pub ruler: Rems,
    /// The left ruler, wide enough for four figures.
    pub side: Rems,
    /// Grid dots at least this far apart on screen.
    pub grid: Rems,
    /// Ruler labels at least this far apart on screen.
    pub label: Rems,
    /// A grid dot, square.
    pub dot: Pixels,
    /// Lines, wide.
    pub stroke: Rems,
    /// Frames and outlines, wide.
    pub hairline: Pixels,
    /// A note's inset and corner at a zoom of one.
    pub note_inset: Rems,
    /// Its folded corner.
    pub note_corner: Rems,
    /// A minimap, wide and tall.
    pub map: (Rems, Rems),
    /// A handle, square.
    pub handle: Rems,
    /// An asset's tile, square, and the room around its drawing.
    pub asset: Rems,
    /// The room around an asset's drawing.
    pub asset_inset: Rems,
    /// The constraints picture, square.
    pub pins: Rems,
    /// A place in the auto layout grid, square; the chosen one's mark and the rest's pips.
    pub place: Rems,
    /// The chosen place's mark.
    pub place_mark: Rems,
    /// The other places' pips.
    pub place_pip: Rems,
}

impl Theme {
    /// Canvas measures.
    pub fn canvas(&self) -> CanvasSizes {
        CanvasSizes {
            ruler: px_to_rems(20.0),
            side: px_to_rems(32.0),
            grid: px_to_rems(16.0),
            label: px_to_rems(64.0),
            dot: px(1.5),
            stroke: px_to_rems(1.5),
            hairline: px(1.0),
            note_inset: px_to_rems(12.0),
            note_corner: px_to_rems(4.0),
            map: (px_to_rems(200.0), px_to_rems(140.0)),
            handle: px_to_rems(8.0),
            asset: px_to_rems(64.0),
            asset_inset: px_to_rems(10.0),
            pins: px_to_rems(72.0),
            place: px_to_rems(20.0),
            place_mark: px_to_rems(8.0),
            place_pip: px_to_rems(3.0),
        }
    }
}

/// Dashboard measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DashboardSizes {
    /// A grid row, tall.
    pub row: Rems,
    /// Between tiles, either way.
    pub gap: Rems,
    /// A status dot, square.
    pub dot: Rems,
    /// Under this width the grid stacks its tiles.
    pub stack_below: Rems,
}

impl Theme {
    /// Dashboard measures.
    pub fn dashboard(&self) -> DashboardSizes {
        DashboardSizes {
            row: px_to_rems(88.0),
            gap: px_to_rems(16.0),
            dot: px_to_rems(8.0),
            stack_below: px_to_rems(560.0),
        }
    }
}

/// Onboarding's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnboardingSizes {
    /// A hotspot's dot, wide; its rings reach twice that.
    pub hotspot: Rems,
}

impl Theme {
    /// Onboarding's measures.
    pub fn onboarding(&self) -> OnboardingSizes {
        OnboardingSizes {
            hotspot: px_to_rems(10.0),
        }
    }
}

/// Interaction's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InteractionSizes {
    /// What an arrow key resizes a box by.
    pub step: Rems,
    /// A resize corner's grip, square.
    pub grip: Rems,
}

impl Theme {
    /// Interaction's measures.
    pub fn interaction(&self) -> InteractionSizes {
        InteractionSizes {
            step: px_to_rems(8.0),
            grip: px_to_rems(12.0),
        }
    }
}

/// A map's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapSizes {
    /// A tile's side at its own zoom, as tile servers cut them.
    pub tile: Pixels,
    /// A popup's width, less where the map is narrower.
    pub popup: Rems,
    /// Room a popup keeps from the map's edges and from its point.
    pub margin: Rems,
    /// A route's stroke.
    pub route: Rems,
    /// A GeoJSON line's and outline's stroke.
    pub line: Rems,
    /// A GeoJSON point's side.
    pub dot: Rems,
    /// A heat cell's side.
    pub heat_cell: Rems,
    /// How far a heat point spreads.
    pub heat_reach: Rems,
    /// A cluster's side.
    pub cluster: Rems,
    /// The square of the world that gathers pins into one cluster.
    pub gather: Rems,
    /// A choropleth legend's scale bar.
    pub legend: Rems,
}

impl Theme {
    /// A map's measures.
    pub fn maps(&self) -> MapSizes {
        MapSizes {
            tile: px(256.0),
            popup: px_to_rems(240.0),
            margin: px_to_rems(8.0),
            route: px_to_rems(3.0),
            line: px_to_rems(1.5),
            dot: px_to_rems(8.0),
            heat_cell: px_to_rems(4.0),
            heat_reach: px_to_rems(28.0),
            cluster: px_to_rems(32.0),
            gather: px_to_rems(56.0),
            legend: px_to_rems(96.0),
        }
    }
}

/// The misc chapter's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiscSizes {
    /// A clock's face.
    pub face: Rems,
    /// A world clock row's face.
    pub small_face: Rems,
    /// A flashcard's least height.
    pub card: Rems,
    /// A license viewer's list, and the least width of the text beside it.
    pub licenses: Rems,
    /// How tall a license's text grows before it scrolls.
    pub reading: Rems,
}

impl Theme {
    /// The misc chapter's measures.
    pub fn misc(&self) -> MiscSizes {
        MiscSizes {
            face: px_to_rems(112.0),
            small_face: px_to_rems(32.0),
            card: px_to_rems(176.0),
            licenses: px_to_rems(240.0),
            reading: px_to_rems(320.0),
        }
    }
}

/// The tooling chapter's measures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolingSizes {
    /// An FPS meter's graph, across and tall.
    pub graph: (Rems, Rems),
    /// How tall an event log grows before it scrolls.
    pub log: Rems,
    /// A token viewer's tile, and its samples' least width.
    pub tile: Rems,
    /// A token viewer's swatch.
    pub swatch: Rems,
}

impl Theme {
    /// The tooling chapter's measures.
    pub fn tooling(&self) -> ToolingSizes {
        ToolingSizes {
            graph: (px_to_rems(160.0), px_to_rems(40.0)),
            log: px_to_rems(200.0),
            tile: px_to_rems(200.0),
            swatch: px_to_rems(28.0),
        }
    }
}
