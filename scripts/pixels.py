#!/usr/bin/env python3
"""Compares two gallery captures shot by shot: a report and a picture of each change; fails past the threshold unless `--accept`."""
import os
import shutil
import sys
from pathlib import Path

from PIL import Image, ImageChops

# A channel may move this much before its pixel counts as changed.
TOLERANCE = 8
# A shot fails when a larger share of its pixels changed.
THRESHOLD = 0.0005
# A shot whose moving pixels pass this share has too little still to compare: a capture fault no Pixels-accepted line lets through.
MOVING = 0.5
# Changed pixels in the picture of a change, over the head shot faded to a fifth.
MARK = (220, 30, 30)


def moving(base, head, size):
    """The pixels a capture saw move while it waited for the view to hold still, on either side."""
    seen = Image.new("L", size, 0)
    for path in (base.with_suffix(".moving.png"), head.with_suffix(".moving.png")):
        if path.is_file():
            mask = Image.open(path).convert("L")
            if mask.size != size:
                sys.exit(f"pixels: {path} is {mask.size}, its shot {size}")
            seen = ImageChops.lighter(seen, mask)
    return seen


def own_moving(shot):
    """The share of one shot its own capture saw move, checked against the shot's size."""
    path = shot.with_suffix(".moving.png")
    if not path.is_file():
        return 0.0
    mask, size = Image.open(path).convert("L"), Image.open(shot).size
    if mask.size != size:
        sys.exit(f"pixels: {path} is {mask.size}, its shot {size}")
    return mask.histogram()[255] / (size[0] * size[1])


def changed(base, head):
    """The share of still pixels that changed, a mask of them or, for a new size, both sizes, and the share passed over as moving."""
    a, b = Image.open(base).convert("RGB"), Image.open(head).convert("RGB")
    if a.size != b.size:
        return 1.0, f"{a.size[0]}x{a.size[1]} to {b.size[0]}x{b.size[1]}", 0.0
    r, g, bl = ImageChops.difference(a, b).split()
    mask = ImageChops.lighter(ImageChops.lighter(r, g), bl).point(lambda v: 255 if v > TOLERANCE else 0)
    skipped = moving(base, head, a.size)
    mask = ImageChops.subtract(mask, skipped)
    area = a.size[0] * a.size[1]
    return mask.histogram()[255] / area, mask, skipped.histogram()[255] / area


def picture(head, mask, path):
    shot = Image.open(head).convert("RGB")
    faded = Image.blend(Image.new("RGB", shot.size, (255, 255, 255)), shot, 0.2)
    faded.paste(Image.new("RGB", shot.size, MARK), mask=mask)
    faded.save(path)


def main():
    args = [arg for arg in sys.argv[1:] if not arg.startswith("--")]
    if len(args) != 3:
        sys.exit("usage: python3 scripts/pixels.py BASE_DIR HEAD_DIR REPORT_DIR [--accept]")
    base, head, out = map(Path, args)
    accept = "--accept" in sys.argv
    out.mkdir(parents=True, exist_ok=True)
    shots = lambda folder: {path.name for path in folder.glob("*.png") if not path.name.endswith(".moving.png")}
    # Each side must have shot, whatever a Pixels-accepted line lets through.
    for folder in (base, head):
        if not shots(folder):
            sys.exit(f"pixels: no shots in {folder}")
    names = sorted(shots(base) | shots(head))
    rows, failed, unusable = [], [], []
    for name in names:
        # Each side's own mask first, so no size or presence difference hides a capture fault.
        worst = max(own_moving(folder / name) for folder in (base, head) if (folder / name).is_file())
        if worst > MOVING:
            share, note = 1.0, f"{worst:.3%} moving, too little still to compare"
            unusable.append(name)
            for folder, kept in ((head, name), (base, name.replace(".png", ".base.png"))):
                if (folder / name).is_file():
                    shutil.copyfile(folder / name, out / kept)
        elif not (base / name).is_file() or not (head / name).is_file():
            share, note = 1.0, "only in head" if (head / name).is_file() else "only in base"
            shutil.copyfile(head / name if (head / name).is_file() else base / name, out / name)
        else:
            share, mask, skipped = changed(base / name, head / name)
            note = f"new size, {mask}" if isinstance(mask, str) else ""
            if skipped > MOVING:
                share, note = 1.0, f"{skipped:.3%} moving, too little still to compare"
                unusable.append(name)
            elif skipped:
                note = f"{skipped:.3%} moving, passed over"
            if isinstance(mask, str) or skipped > MOVING:
                shutil.copyfile(head / name, out / name)
                shutil.copyfile(base / name, out / name.replace(".png", ".base.png"))
            elif share > 0:
                picture(head / name, mask, out / name)
        if share > 0:
            rows.append((share, name, note))
        if share > THRESHOLD:
            failed.append(name)
    rows.sort(reverse=True)
    lines = [
        f"## Pixels: {len(names)} shots, {len(rows)} changed, {len(failed)} past {THRESHOLD:.2%}",
        "",
        f"A pixel counts when a channel moves more than {TOLERANCE}, unless the capture saw it move while it waited; changed pixels show red in the report's pictures. A shot only one side has is kept as it is; a shot of a new size, or mostly moving, is kept as head's `NAME.png` beside base's `NAME.base.png`.",
        "",
    ]
    if rows:
        lines += ["| Shot | Changed | |", "| --- | ---: | --- |"]
        lines += [f"| {name} | {share:.3%} | {', '.join(filter(None, ['fails' if name in failed else '', note]))} |" for share, name, note in rows]
    report = "\n".join(lines) + "\n"
    (out / "report.md").write_text(report, encoding="utf-8", newline="\n")
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8", newline="\n") as file:
            file.write(report)
    print(report)
    if unusable:
        sys.exit(f"pixels: {len(unusable)} shots kept moving over more than {MOVING:.0%} of the window; hold them still for the capture")
    if failed and accept:
        print(f"pixels: {len(failed)} shots changed past the threshold; accepted by a Pixels-accepted line")
    elif failed:
        sys.exit(f"pixels: {len(failed)} shots changed past the threshold; read the report, then add a `Pixels-accepted: <why>` line to a commit message if the change is meant")


if __name__ == "__main__":
    main()
