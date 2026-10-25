#!/usr/bin/env python3
"""Holds benches/budget.rs to its budgets; `--against NAME` also fails a regression past 10% over that baseline."""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RESULTS = ROOT / "target" / "criterion"
FRAME = 16e6
# Nanoseconds each bench may take: a 60 Hz frame, and crates/ely-documents/AGENTS.md's 2 MiB read.
BUDGETS = {
    "table_scroll_100k_rows": FRAME,
    "editor_typing_10k_lines": FRAME,
    "terminal_scrollback_10k_lines": FRAME,
    "markdown_read_2mib": 330e6,
}
REGRESSION = 0.10


def mean(path):
    if not path.is_file():
        sys.exit(f"bench: no result at {path.relative_to(ROOT)}; run cargo bench first")
    return json.loads(path.read_text(encoding="utf-8"))["mean"]


def main():
    against = sys.argv[sys.argv.index("--against") + 1] if "--against" in sys.argv else None
    failures = []
    for name, budget in BUDGETS.items():
        took = mean(RESULTS / name / "new" / "estimates.json")["point_estimate"]
        line = f"bench: {name}: {took / 1e6:.2f} ms, budget {budget / 1e6:.0f} ms"
        if took > budget:
            failures.append(f"over budget: {name} {took / 1e6:.2f} ms > {budget / 1e6:.0f} ms")
        if against:
            if not (RESULTS / name / against).is_dir():
                sys.exit(f"bench: {name} has no baseline {against}")
            change = mean(RESULTS / name / "change" / "estimates.json")
            rise, low = change["point_estimate"], change["confidence_interval"]["lower_bound"]
            line += f", {rise:+.1%} against {against} (lower bound {low:+.1%})"
            if rise > REGRESSION and low > 0:
                failures.append(f"regression: {name} {rise:+.1%} against {against}")
        print(line)
    if failures:
        print("\n".join(failures))
        sys.exit(f"bench: {len(failures)} failed")


main()
