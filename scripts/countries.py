"""Writes crates/ely-maps/assets/countries.geojson, Ely's world, from Natural Earth's 110m countries.

Usage: python3 scripts/countries.py <ne_110m_admin_0_countries.geojson>
Keeps each country's ISO 3166 alpha-2 code and name; positions round to 0.01 degrees.
"""

import json
import sys
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "crates/ely-maps/assets/countries.geojson"


def ring(points):
    kept = []
    for lon, lat in points:
        point = [round(lon, 2), round(lat, 2)]
        if not kept or kept[-1] != point:
            kept.append(point)
    if kept[0] != kept[-1]:
        kept.append(kept[0])
    return kept if len(kept) >= 4 else None


def polygon(rings):
    kept = [ring(each) for each in rings]
    if kept[0] is None:
        return None
    return [each for each in kept if each is not None]


def main(source):
    features = []
    for feature in json.load(open(source, encoding="utf-8"))["features"]:
        props, shape = feature["properties"], feature["geometry"]
        polygons = [shape["coordinates"]] if shape["type"] == "Polygon" else shape["coordinates"]
        kept = [each for each in (polygon(rings) for rings in polygons) if each]
        iso = props["ISO_A2_EH"]
        properties = {"name": props["NAME"]} | ({"iso": iso} if iso != "-99" else {})
        features.append({
            "type": "Feature",
            "properties": properties,
            "geometry": {"type": "MultiPolygon", "coordinates": kept},
        })
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"type": "FeatureCollection", "features": features}, separators=(",", ":")), encoding="utf-8")
    print(f"wrote {len(features)} countries, {OUT.stat().st_size} bytes, to {OUT}")


if __name__ == "__main__":
    main(sys.argv[1])
