"""Writes the gallery's map tiles, z0 to z3, from Natural Earth's 110m countries.

Usage: python3 scripts/tiles.py <ne_110m_admin_0_countries.geojson>
Land is gray with alpha, so one set reads on light and dark maps.
"""

import json
import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw

TILE, GRAIN, ZOOMS = 256, 4, range(4)
LAND, BORDER = (128, 72), (128, 150)
OUT = Path(__file__).resolve().parent.parent / "crates/ely-gallery/assets/tiles"


def project(lon, lat):
    lat = max(-85.05112878, min(85.05112878, lat))
    phi = math.radians(lat)
    return (lon + 180) / 360, (1 - math.log(math.tan(phi) + 1 / math.cos(phi)) / math.pi) / 2


def rings(geometry):
    kind, coordinates = geometry["type"], geometry["coordinates"]
    polygons = [coordinates] if kind == "Polygon" else coordinates
    assert kind in ("Polygon", "MultiPolygon"), kind
    return [ring for polygon in polygons for ring in polygon]


def main(source):
    shapes = [rings(feature["geometry"]) for feature in json.load(open(source, encoding="utf-8"))["features"]]
    for z in ZOOMS:
        count = 2**z
        side = TILE * GRAIN
        for x in range(count):
            for y in range(count):
                image = Image.new("LA", (side, side), (0, 0))
                draw = ImageDraw.Draw(image)
                for shape in shapes:
                    for ring in shape:
                        points = [
                            ((px * count - x) * side, (py * count - y) * side)
                            for px, py in (project(lon, lat) for lon, lat in ring)
                        ]
                        draw.polygon(points, fill=LAND, outline=BORDER, width=GRAIN)
                path = OUT / str(z) / str(x) / f"{y}.png"
                path.parent.mkdir(parents=True, exist_ok=True)
                image.resize((TILE, TILE), Image.LANCZOS).save(path, optimize=True)
    print(f"wrote {sum(4**z for z in ZOOMS)} tiles to {OUT}")


if __name__ == "__main__":
    main(sys.argv[1])
