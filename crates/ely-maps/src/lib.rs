//! Maps, chapter 39: tiled maps with pins, popups, routes, GeoJSON, heat and choropleths, a world map and a place picker.
#![deny(missing_docs)]

use ely_canvas as canvas;
use ely_core::{buttons, i18n, layout, primitives, theme, typography};

mod choropleth;
mod cluster;
mod coords;
mod geo;
mod geojson;
mod heat;
mod layer;
mod marker;
mod picker;
#[cfg(all(test, feature = "test-support"))]
mod tests;
mod tile;
mod view;
mod world;

pub use choropleth::{ChoroplethKey, ChoroplethLayer, ChoroplethMap};
pub use coords::{CoordFormat, CoordinateDisplay};
pub use geo::{LatLon, MAX_LAT, MapViewport, Tile};
pub use geojson::{Feature, Geometry, read_geojson};
pub use layer::{GeoHeatmap, GeoJsonLayer, MapLayer, RouteLine};
pub use marker::{MapMarker, MapPopup};
pub use picker::LocationPicker;
pub use view::MapView;
pub use world::WorldMap;
