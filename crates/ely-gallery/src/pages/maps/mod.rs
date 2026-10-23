use ely_gpui_component::{
    maps::{
        CoordFormat, CoordinateDisplay, LatLon, LocationPicker, MapMarker, MapPopup, MapView,
        MapViewport, Tile,
    },
    theme::ActiveTheme,
};
use gpui::{
    AnyElement, App, ImageSource, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
};

mod layers;
mod world;

use super::Page;
use crate::probe::probe;
use crate::step::Step;
use crate::ui::{keep, picture, section, set, specimen, specimens};

pub const PAGE: Page = Page {
    number: 41,
    slug: "maps",
    title: "Maps",
    summary: "Tiles the host supplies, pins with a popup, a picker's crosshair, routes, GeoJSON, heat, clusters and choropleths. The land is Natural Earth's.",
    render,
    script: &[
        Step::Click("world-map"),
        Step::Key("tab"),
        Step::Key("enter"),
        Step::Wait(300),
        Step::Shot("popup"),
        Step::Key("escape"),
        Step::Key("="),
        Step::Wait(300),
        Step::Shot("zoomed"),
        Step::Click("picker"),
        Step::Key("right"),
        Step::Key("down"),
        Step::Wait(300),
        Step::Shot("picked"),
        Step::Click("cluster-map"),
        Step::Key("tab"),
        Step::Key("enter"),
        Step::Wait(300),
        Step::Shot("cluster-opened"),
    ],
};

const TILES: &str = asset!("tiles");
pub(super) const ATTRIBUTION: &str = "Land: Natural Earth";

const CITIES: [(&str, &str, f64, f64, usize); 6] = [
    ("london", "London", 51.5072, -0.1276, 0),
    ("new-york", "New York", 40.7128, -74.006, 3),
    ("sao-paulo", "São Paulo", -23.5505, -46.6333, 5),
    ("cape-town", "Cape Town", -33.9249, 18.4241, 2),
    ("tokyo", "Tokyo", 35.6762, 139.6503, 4),
    ("sydney", "Sydney", -33.8688, 151.2093, 1),
];

/// The gallery's tiles, cut from Natural Earth by `scripts/tiles.py`.
pub(super) fn tile(tile: Tile) -> ImageSource {
    picture(format!("{TILES}/{}/{}/{}.png", tile.z, tile.x, tile.y))
}

fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .child(world(window, cx))
        .child(coordinates(cx))
        .child(picker(cx))
        .child(layers::routes(cx))
        .child(layers::heat(cx))
        .child(layers::clusters(cx))
        .child(world::world(cx))
        .into_any_element()
}

fn world(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = keep("map-open", || None::<SharedString>, window, cx);
    let shown = open.read(cx).clone();
    let markers = CITIES.map(|(key, name, lat, lon, hue)| {
        MapMarker::new(key, LatLon::new(lat, lon))
            .label(name)
            .hue(hue)
    });
    let popup = shown.as_ref().map(|key| {
        let (_, name, lat, lon, _) = *CITIES
            .iter()
            .find(|(each, ..)| *each == key.as_ref())
            .expect("a listed city");
        let closed = open.clone();
        MapPopup::new(LatLon::new(lat, lon), name)
            .child(
                CoordinateDisplay::new("map-popup-place", LatLon::new(lat, lon))
                    .format(CoordFormat::Dms),
            )
            .on_close(move |_, cx| set(&closed, None, cx))
    });
    let map = MapView::new("world-map", MapViewport::new(LatLon::new(20.0, 10.0), 1.2))
        .tiles(tile)
        .tile_zooms(0, 3)
        .attribution(ATTRIBUTION)
        .markers(markers)
        .on_marker(move |key, _, cx| set(&open, Some(key.clone()), cx));
    let map = match popup {
        Some(popup) => map.popup(popup),
        None => map,
    };
    section(
        "MapView · MapMarker · MapPopup",
        "Tiles come from the host by zoom, column and row; past the zooms it cuts, the nearest scales. Drag or scroll to pan, Command-scroll or the buttons to zoom; focused, the arrows pan and + and - zoom. A pin opens its popup; Escape closes it.",
        cx,
    )
    .child(probe("world-map", div().w(px(720.0)).h(px(360.0)).child(map)))
}

fn coordinates(cx: &App) -> impl IntoElement + use<> {
    let place = LatLon::new(-33.9249, 18.4241);
    section(
        "CoordinateDisplay",
        "A place in degrees or in degrees, minutes and seconds, with its hemispheres; figures keep one width, and the button copies it.",
        cx,
    )
    .child(
        specimens()
            .child(specimen("decimal", CoordinateDisplay::new("coords-decimal", place), cx))
            .child(specimen(
                "degrees, minutes, seconds",
                CoordinateDisplay::new("coords-dms", place).format(CoordFormat::Dms),
                cx,
            )),
    )
}

fn picker(cx: &App) -> impl IntoElement + use<> {
    let muted = cx.theme().colors.fg_muted;
    let map = MapView::new(
        "picker-map",
        MapViewport::new(LatLon::new(48.8566, 2.3522), 3.0),
    )
    .tiles(tile)
    .tile_zooms(0, 3)
    .attribution(ATTRIBUTION);
    section(
        "LocationPicker",
        "The place under the crosshair is the pick: pan by pointer or keys, and the owner hears each move.",
        cx,
    )
    .child(
        probe(
            "picker",
            div().w(px(480.0)).h(px(320.0)).child(
                LocationPicker::new("picker", map)
                    .format(CoordFormat::Dms)
                    .on_pick(|at, _, _| log::info!("maps demo: picked {:.4}, {:.4}", at.lat, at.lon)),
            ),
        ),
    )
    .child(div().text_color(muted).child("The pick reaches the owner as it moves; the log shows it."))
}
