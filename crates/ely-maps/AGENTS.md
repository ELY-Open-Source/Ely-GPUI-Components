# AGENTS.md: ely-maps

Rules for `ely-maps`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Stack

- Maps: Natural Earth's 110m countries (public domain). `crates/ely-maps/assets/countries.geojson`, compiled into `ely-maps` with `include_str!` and trimmed by `scripts/countries.py` to an ISO code, a name and positions to 0.01°, backs `maps::WorldMap`; the gallery's tiles are cut from the same file, z0 to z3, by `scripts/tiles.py` with Pillow.

## Rules

- gpui packs pictures in its atlas without padding and samples them with a linear filter, on the web as on macOS, so a scaled picture's edge takes color from its atlas neighbor, and pictures laid edge to edge show seams. A map tile carries a one-pixel rim copied from its own edge and draws one of its pixels larger each side, clipped to its box (`maps::tile`).
- A map keeps places in `f64`: at zoom 19 the world is 134 million pixels across. Its wheel pans and Command or Control with the wheel zooms about the pointer, as the canvas's does. Its tiles stay in gpui's asset cache, as every `img()` picture does. Its popup is an overlay that is not modal: it takes focus when it shows, and the frame it stops showing, by any way out, hands focus back. Its layers, lines and areas unwrap across the date line and repeat on every world in view. Clustered pins gather by squares of the world, not of the view, so a pan never regroups them; a zoom may, so it hands a focused pin's or cluster's focus to the map.
