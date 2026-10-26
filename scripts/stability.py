#!/usr/bin/env python3
"""Holds tasks/stability.md to its rules: every published crate a row, and stable only with the owner's sign-off."""
import json
import re
import subprocess
import sys
from pathlib import Path
from urllib.parse import urlparse

LEDGER = Path("tasks/stability.md")
SIGNOFFS = Path("tasks/signoffs")
UMBRELLA = "ely-gpui-component"
CORE = "ely-core"
FIELDS = ("Crate", "Version", "Commit", "Signed by", "Date", "Verdict")


def published():
    """Each published crate's version, from cargo's own metadata."""
    out = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1"], capture_output=True, text=True, check=True).stdout
    return {p["name"]: p["version"] for p in json.loads(out)["packages"] if p["publish"] != []}


def rows():
    """The ledger's rows: crate, status, sign-off and feedback cells."""
    found = []
    for line in LEDGER.read_text(encoding="utf-8").splitlines():
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if line.startswith("|") and len(cells) == 4 and cells[0] not in ("Crate",) and not set(cells[0]) <= set("-"):
            found.append(cells)
    return found


def signoff(crate, faults):
    """Checks the owner's record for a stable crate."""
    path = SIGNOFFS / f"{crate}.md"
    if not path.is_file():
        faults.append(f"{crate} is stable without {path}")
        return
    text = path.read_text(encoding="utf-8")
    fields = dict(re.findall(r"^- ([A-Za-z ]+): (.*)$", text, re.MULTILINE))
    for name in FIELDS:
        value = fields.get(name, "").strip()
        if not value or "<" in value:
            faults.append(f"{path}: '{name}' is not filled in")
    if fields.get("Crate", "").strip() != crate:
        faults.append(f"{path}: names crate {fields.get('Crate')!r}, not {crate}")
    if fields.get("Verdict", "").strip() != "stable":
        faults.append(f"{path}: its verdict is not 'stable'")
    commit = fields.get("Commit", "").strip()
    if commit and "<" not in commit:
        # A hash, not a ref that moves, and one whose snapshot of this crate was there to read.
        kind = subprocess.run(["git", "cat-file", "-t", commit], capture_output=True, text=True).stdout.strip()
        if not re.fullmatch(r"[0-9a-f]{7,40}", commit) or kind != "commit":
            faults.append(f"{path}: {commit!r} is not the hash of a commit")
        elif subprocess.run(["git", "cat-file", "-e", f"{commit}:api/{crate}.txt"], capture_output=True).returncode:
            faults.append(f"{path}: commit {commit} holds no api/{crate}.txt")


def feedback_links(cell):
    """The usable web addresses in a feedback cell: a host with a dot and a path past it."""
    links = []
    for url in re.findall(r"https?://[^\s)\]>|]+", cell):
        parsed = urlparse(url)
        if "@" not in parsed.netloc and "." in (parsed.hostname or "") and parsed.path.strip("/"):
            links.append(url)
    return links


def main():
    versions, faults = published(), []
    ledger = rows()
    named = [cells[0] for cells in ledger]
    for crate in sorted(set(versions) - set(named)):
        faults.append(f"{crate} has no row in {LEDGER}")
    for crate in sorted(set(named) - set(versions)):
        faults.append(f"{LEDGER} names {crate}, which is no published crate")
    for crate in sorted({c for c in named if named.count(c) > 1}):
        faults.append(f"{crate} has more than one row in {LEDGER}")
    status = {}
    for crate, state, _, feedback in ledger:
        status[crate] = state
        if state not in ("unstable", "stable"):
            faults.append(f"{crate}: status {state!r} is neither unstable nor stable")
        if state == "stable":
            signoff(crate, faults)
            if crate not in (CORE, UMBRELLA) and ("<" in feedback or not feedback_links(feedback)):
                faults.append(f"{crate} is stable without a link to an outside user's feedback, or with a placeholder")
        major = int(versions.get(crate, "0").split(".")[0])
        if major >= 1 and state != "stable":
            faults.append(f"{crate} is at {versions[crate]} but not stable in {LEDGER}")
    if status.get(UMBRELLA) == "stable":
        for crate in sorted(c for c in status if c != UMBRELLA and status[c] != "stable"):
            faults.append(f"{UMBRELLA} is stable while {crate} is not")
    if faults:
        sys.exit("stability:\n" + "\n".join(f"  {fault}" for fault in faults))
    stable = sum(1 for state in status.values() if state == "stable")
    print(f"stability: {len(status)} crates, {stable} stable")


if __name__ == "__main__":
    main()
