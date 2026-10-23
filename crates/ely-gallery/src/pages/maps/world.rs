use ely_gpui_component::maps::{
    ChoroplethLayer, ChoroplethMap, LatLon, MapView, MapViewport, WorldMap, read_geojson,
};
use gpui::{App, IntoElement, ParentElement, Styled, div, px};

use super::{ATTRIBUTION, tile};
use crate::ui::section;

const VISITORS: [(&str, f64); 20] = [
    ("US", 120_400.0),
    ("IN", 51_300.0),
    ("GB", 45_100.0),
    ("DE", 38_200.0),
    ("JP", 30_800.0),
    ("FR", 26_400.0),
    ("BR", 22_100.0),
    ("CA", 18_300.0),
    ("AU", 14_700.0),
    ("ES", 13_200.0),
    ("CN", 12_600.0),
    ("IT", 12_100.0),
    ("KR", 11_400.0),
    ("MX", 9_100.0),
    ("SG", 8_300.0),
    ("RU", 7_200.0),
    ("AR", 6_300.0),
    ("ZA", 5_400.0),
    ("EG", 4_100.0),
    ("NG", 3_200.0),
];

const REGIONS: &str = r#"{"type": "FeatureCollection", "features": [
    {"type": "Feature", "properties": {"region": "west"},
     "geometry": {"type": "Polygon", "coordinates": [[[-9, 36], [3, 36], [3, 43.5], [-9, 43.5], [-9, 36]]]}},
    {"type": "Feature", "properties": {"region": "middle"},
     "geometry": {"type": "Polygon", "coordinates": [[[-4, 43.5], [8, 43.5], [8, 51], [-4, 51], [-4, 43.5]]]}},
    {"type": "Feature", "properties": {"region": "south"},
     "geometry": {"type": "Polygon", "coordinates": [[[7, 37], [18, 37], [18, 46], [7, 46], [7, 37]]]}},
    {"type": "Feature", "properties": {"region": "north"},
     "geometry": {"type": "Polygon", "coordinates": [[[5, 55], [15, 55], [15, 60], [5, 60], [5, 55]]]}}
]}"#;

pub fn world(cx: &App) -> impl IntoElement + use<> {
    let regions = read_geojson(REGIONS).expect("the demo's regions read");
    let rainfall = ChoroplethLayer::new(
        regions,
        [("west", 540.0), ("middle", 810.0), ("south", 700.0)],
    )
    .key_by("region")
    .hue(1);
    let over = MapView::new("rain-map", MapViewport::new(LatLon::new(46.0, 5.0), 3.2))
        .tiles(tile)
        .tile_zooms(0, 3)
        .attribution(ATTRIBUTION);
    let visits = MapView::new(
        "world-visits",
        MapViewport::new(LatLon::new(25.0, 10.0), 1.4),
    );
    section(
        "ChoroplethMap · WorldMap",
        "Areas washed by value on one hue's scale, with a legend; an area without a value reads as no data. WorldMap washes Natural Earth's countries by ISO code: Singapore, too small at this scale, draws nowhere and is logged once.",
        cx,
    )
    .child(
        div()
            .w(px(720.0))
            .h(px(360.0))
            .child(WorldMap::new("world-visits", visits, VISITORS).hue(0)),
    )
    .child(
        div()
            .w(px(480.0))
            .h(px(320.0))
            .child(ChoroplethMap::new("rain", over, rainfall)),
    )
}
