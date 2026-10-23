use ely_gpui_component::maps::{
    GeoHeatmap, GeoJsonLayer, LatLon, MapMarker, MapView, MapViewport, RouteLine, read_geojson,
};
use gpui::{App, IntoElement, ParentElement, Styled, div, px};

use super::{ATTRIBUTION, tile};
use crate::probe::probe;
use crate::ui::{noise, section};

const ROUTE: [(f64, f64); 4] = [
    (51.5072, -0.1276),
    (64.1466, -21.9426),
    (44.6488, -63.5752),
    (40.7128, -74.006),
];

const SHAPES: &str = r#"{"type": "FeatureCollection", "features": [
    {"type": "Feature", "properties": {"name": "The Bermuda Triangle"},
     "geometry": {"type": "Polygon", "coordinates": [
        [[-80.19, 25.76], [-64.78, 32.3], [-66.1, 18.47], [-80.19, 25.76]]]}},
    {"type": "Feature", "properties": {"name": "A ring around the Azores"},
     "geometry": {"type": "Polygon", "coordinates": [
        [[-35, 33], [-20, 33], [-20, 42], [-35, 42], [-35, 33]],
        [[-30, 36], [-25, 36], [-25, 39], [-30, 39], [-30, 36]]]}},
    {"type": "Feature", "properties": {"name": "The equator"},
     "geometry": {"type": "LineString", "coordinates": [[-50, 0], [10, 0]]}},
    {"type": "Feature", "properties": {"name": "Ports"},
     "geometry": {"type": "MultiPoint", "coordinates": [[-9.14, 38.72], [-17.44, 14.69], [-23.51, 14.93]]}}
]}"#;

const QUAKES: [(f64, f64); 5] = [
    (35.68, 139.69),
    (-33.45, -70.67),
    (-6.21, 106.85),
    (41.01, 28.98),
    (37.77, -122.42),
];

const CAPITALS: [(f64, f64); 14] = [
    (51.51, -0.13),
    (48.86, 2.35),
    (52.52, 13.4),
    (40.42, -3.7),
    (41.9, 12.5),
    (48.21, 16.37),
    (52.37, 4.9),
    (50.85, 4.35),
    (38.72, -9.14),
    (50.08, 14.44),
    (52.23, 21.01),
    (59.33, 18.07),
    (55.68, 12.57),
    (47.5, 19.04),
];

fn map(id: &'static str, center: (f64, f64), zoom: f64) -> MapView {
    MapView::new(id, MapViewport::new(LatLon::new(center.0, center.1), zoom))
        .tiles(tile)
        .tile_zooms(0, 3)
        .attribution(ATTRIBUTION)
}

pub fn routes(cx: &App) -> impl IntoElement + use<> {
    let features = read_geojson(SHAPES).expect("the demo's GeoJSON reads");
    let route = RouteLine::new(ROUTE.map(|(lat, lon)| LatLon::new(lat, lon))).hue(0);
    section(
        "RouteLine · GeoJSON Layer",
        "A route along places in order, the short way across the date line. GeoJSON reads to features: areas washed and outlined with their holes open, lines stroked, points dotted. Both repeat on each world in view.",
        cx,
    )
    .child(
        div().w(px(720.0)).h(px(360.0)).child(
            map("routes-map", (38.0, -40.0), 2.3)
                .layer(GeoJsonLayer::new(features).hue(3))
                .layer(route),
        ),
    )
}

pub fn heat(cx: &App) -> impl IntoElement + use<> {
    let mut next = noise(41);
    let points: Vec<(LatLon, f32)> = QUAKES
        .iter()
        .flat_map(|(lat, lon)| (0..30).map(move |_| (*lat, *lon)))
        .map(|(lat, lon)| {
            let place = LatLon::new(lat + (next() - 0.5) * 6.0, lon + (next() - 0.5) * 6.0);
            (place, 0.5 + next() as f32 * 1.5)
        })
        .collect();
    section(
        "GeoHeatmap",
        "Weighted places as heat, hotter where they crowd, scaled to the densest spot in view.",
        cx,
    )
    .child(
        div()
            .w(px(720.0))
            .h(px(360.0))
            .child(map("heat-map", (20.0, 30.0), 1.0).layer(GeoHeatmap::new(points).hue(6))),
    )
}

pub fn clusters(cx: &App) -> impl IntoElement + use<> {
    let mut next = noise(7);
    let pins: Vec<MapMarker> = CAPITALS
        .iter()
        .flat_map(|(lat, lon)| (0..3).map(move |_| (*lat, *lon)))
        .enumerate()
        .map(|(ix, (lat, lon))| {
            let place = LatLon::new(lat + (next() - 0.5) * 1.2, lon + (next() - 0.5) * 1.8);
            MapMarker::new(format!("pin-{ix}"), place).hue(ix % 8)
        })
        .collect();
    section(
        "MapCluster",
        "Pins that crowd one square of the world gather into a count; a pan keeps each group, a press shows its members whole.",
        cx,
    )
    .child(probe(
        "cluster-map",
        div().w(px(720.0)).h(px(360.0)).child(
            map("cluster-map", (42.0, 5.0), 3.0)
                .markers(pins)
                .cluster_markers()
                .on_marker(|key, _, _| log::info!("maps demo: pin {key}")),
        ),
    ))
}
