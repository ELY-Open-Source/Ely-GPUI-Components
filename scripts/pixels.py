#!/usr/bin/env python3
"""Compares two gallery captures shot by shot: a report and a picture of each change; fails past the threshold unless `--accept`."""
import os
import sys
from pathlib import Path

from PIL import Image, ImageChops

# A channel may move this much before its pixel counts as changed.
TOLERANCE = 8
# A shot fails when a larger share of its pixels changed.
THRESHOLD = 0.0005
# Changed pixels in the picture of a change, over the head shot faded to a fifth.
MARK = (220, 30, 30)


def changed(base, head):
    """The share of pixels that changed, and a mask of them or, for a new size, both sizes; a new size changes every pixel."""
    a, b = Image.open(base).convert("RGB"), Image.open(head).convert("RGB")
    if a.size != b.size:
        return 1.0, f"{a.size[0]}x{a.size[1]} to {b.size[0]}x{b.size[1]}"
    r, g, bl = ImageChops.difference(a, b).split()
    mask = ImageChops.lighter(ImageChops.lighter(r, g), bl).point(lambda v: 255 if v > TOLERANCE else 0)
    count = mask.histogram()[255]
    return count / (a.size[0] * a.size[1]), mask


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
    names = sorted({path.name for path in base.glob("*.png")} | {path.name for path in head.glob("*.png")})
    if not names:
        sys.exit(f"pixels: no shots in {base} or {head}")
    rows, failed = [], []
    for name in names:
        if not (base / name).is_file() or not (head / name).is_file():
            share, note = 1.0, "only in head" if (head / name).is_file() else "only in base"
        else:
            share, mask = changed(base / name, head / name)
            note = f"new size, {mask}" if isinstance(mask, str) else ""
            if not isinstance(mask, str) and share > 0:
                picture(head / name, mask, out / name)
        if share > 0:
            rows.append((share, name, note))
        if share > THRESHOLD:
            failed.append(name)
    rows.sort(reverse=True)
    lines = [
        f"## Pixels: {len(names)} shots, {len(rows)} changed, {len(failed)} past {THRESHOLD:.2%}",
        "",
        f"A pixel counts when a channel moves more than {TOLERANCE}; changed pixels show red in the report's pictures.",
        "",
    ]
    if rows:
        lines += ["| Shot | Changed | |", "| --- | ---: | --- |"]
        lines += [f"| {name} | {share:.3%} | {note or ('fails' if name in failed else '')} |" for share, name, note in rows]
    report = "\n".join(lines) + "\n"
    (out / "report.md").write_text(report, encoding="utf-8", newline="\n")
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8", newline="\n") as file:
            file.write(report)
    print(report)
    if failed and accept:
        print(f"pixels: {len(failed)} shots changed past the threshold; accepted by a Pixels-accepted line")
    elif failed:
        sys.exit(f"pixels: {len(failed)} shots changed past the threshold; read the report, then add a `Pixels-accepted: <why>` line to a commit message if the change is meant")


if __name__ == "__main__":
    main()
