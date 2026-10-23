"""Writes the gallery's captcha pictures: each answer's letters turned and set off, over noise.

Usage: python3 scripts/captcha.py
The seed is fixed, so the pictures come out the same each run.
"""

import random
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "crates/ely-gallery/assets/captcha"
FONT = ImageFont.truetype(str(ROOT / "crates/ely-core/assets/fonts/Inter-SemiBold.ttf"), 76)
ANSWERS = ["W7XK", "Q3NP"]
SIZE = (480, 160)


def picture(answer, rng):
    image = Image.new("RGB", SIZE, (236, 238, 242))
    draw = ImageDraw.Draw(image)
    for _ in range(900):
        x, y = rng.randrange(SIZE[0]), rng.randrange(SIZE[1])
        draw.point((x, y), fill=tuple(rng.randrange(150, 210) for _ in range(3)))
    step = SIZE[0] // (len(answer) + 1)
    for ix, letter in enumerate(answer):
        tile = Image.new("RGBA", (120, 120), (0, 0, 0, 0))
        ImageDraw.Draw(tile).text((60, 60), letter, font=FONT, anchor="mm", fill=(40, 48, 70, 255))
        tile = tile.rotate(rng.uniform(-28, 28), resample=Image.BICUBIC)
        image.paste(tile, (step * (ix + 1) - 60, 20 + rng.randrange(-14, 14)), tile)
    for _ in range(5):
        points = [(x, rng.randrange(20, SIZE[1] - 20)) for x in range(0, SIZE[0] + 60, 60)]
        draw.line(points, fill=(90, 100, 130), width=3)
    return image


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    rng = random.Random(42)
    for answer in ANSWERS:
        picture(answer, rng).save(OUT / f"{answer.lower()}.png")
        print(f"wrote {answer.lower()}.png")


main()
