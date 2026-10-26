#!/usr/bin/env python3
"""Holds README.md's links to the tree: each relative one names a path that exists, read literally; links are Markdown, never HTML."""
import re
import sys
from pathlib import Path

README = Path("README.md")
# The whole text at once, so a link split over lines is still read.
INLINE = re.compile(r"\]\(\s*(<[^>]*>|[^\s)]+)", re.S)
REFERENCE = re.compile(r"^ {0,3}\[[^\]]+\]:\s*(<[^>]*>|\S+)", re.M)
HTML = re.compile(r"\b(href|src)\s*=", re.I)


def targets(text):
    """Each link destination, its angle brackets and anchor dropped."""
    for match in [*INLINE.finditer(text), *REFERENCE.finditer(text)]:
        target = match.group(1).strip("<>").split("#")[0]
        line = text.count("\n", 0, match.start()) + 1
        yield line, target


def main():
    text = README.read_text(encoding="utf-8")
    faults = []
    for match in HTML.finditer(text):
        faults.append(f"{README}:{text.count(chr(10), 0, match.start()) + 1}: links through HTML; write a Markdown link")
    for line, target in targets(text):
        if not target or re.match(r"^[a-z][a-z0-9+.-]*:", target, re.I):
            continue
        if not Path(target).exists():
            faults.append(f"{README}:{line}: links {target}, which does not exist")
    if faults:
        sys.exit("links:\n" + "\n".join(f"  {fault}" for fault in faults))
    print(f"links: {README} holds {sum(1 for _ in targets(text))} links, every relative one in the tree")


if __name__ == "__main__":
    main()
