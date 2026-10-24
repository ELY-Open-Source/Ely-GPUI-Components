#!/usr/bin/env python3
"""Holds each library crate's public API, or the named ones', to its snapshot in api/; `--write` takes new snapshots. Linux reads the cfg CI reads."""
import difflib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SNAPSHOTS = ROOT / "api"
# The rustdoc JSON cargo-public-api reads moves with nightly; CI installs these two.
NIGHTLY = "nightly-2026-10-24"
TOOL = "0.52.0"


def metadata():
    run = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    )
    meta = json.loads(run.stdout)
    crates = {
        package["name"]: sorted(set(package["features"]) - {"default"})
        for package in meta["packages"]
        if any("lib" in target["kind"] for target in package["targets"])
    }
    return crates, Path(meta["target_directory"])


def configs(features):
    """The feature sets an item may need: the defaults, none, each alone, and all of them."""
    sets = [("default", []), ("none", ["--no-default-features"])]
    sets += [(feature, ["--no-default-features", "--features", feature]) for feature in features]
    if len(features) > 1:
        sets.append(("all", ["--all-features"]))
    return sets


def reexports(doc):
    """Each public `use` with the item it names, which cargo-public-api leaves out."""
    index, paths = doc["index"], doc["paths"]
    lines = []

    def walk(id, path, chain):
        if id in chain:
            return
        chain = chain | {id}
        for child in index[str(id)]["inner"]["module"]["items"]:
            item = index[str(child)]
            if item["visibility"] != "public":
                continue
            inner = item["inner"]
            if "module" in inner:
                walk(child, f"{path}::{item['name']}", chain)
            elif "use" in inner:
                use = inner["use"]
                target = paths.get(str(use["id"])) if use["id"] is not None else None
                to = "::".join(target["path"]) if target else use["source"]
                name = "*" if use["is_glob"] else use["name"]
                lines.append(f"pub use {path}::{name} = {to}{'::*' if use['is_glob'] else ''}")
                # A module of this crate's own, reached through the use, shows under its public path.
                local = index.get(str(use["id"])) if use["id"] is not None else None
                if local and "module" in local["inner"]:
                    walk(use["id"], path if use["is_glob"] else f"{path}::{use['name']}", chain)

    walk(doc["root"], index[str(doc["root"])]["name"], frozenset())
    return lines


def surface(name, flags, target):
    run = subprocess.run(
        ["cargo", f"+{NIGHTLY}", "public-api", "-p", name, *flags, "-ss", "--color", "never"],
        cwd=ROOT, capture_output=True, text=True,
    )
    if run.returncode != 0:
        sys.exit(f"api: {name} {' '.join(flags)}: cargo public-api failed\n{run.stderr}")
    doc = json.loads((target / "doc" / f"{name.replace('-', '_')}.json").read_text(encoding="utf-8"))
    return set(run.stdout.splitlines()) | set(reexports(doc))


def snapshot(name, features, target):
    """Every item, and the feature sets that have it when not all do."""
    seen = {label: surface(name, flags, target) for label, flags in configs(features)}
    lines = []
    for line in sorted(set().union(*seen.values())):
        having = [label for label, items in seen.items() if line in items]
        lines.append(line if len(having) == len(seen) else f"{line}  [with: {', '.join(having)}]")
    return "\n".join(lines) + "\n"


def main():
    if "--toolchain" in sys.argv:
        print(NIGHTLY)
        return
    if "--tool" in sys.argv:
        print(TOOL)
        return
    if sys.platform != "linux":
        sys.exit("api: snapshots hold Linux's cfg; run on Linux")
    write = "--write" in sys.argv
    crates, target = metadata()
    asked = [arg for arg in sys.argv[1:] if not arg.startswith("--")]
    unknown = sorted(set(asked) - set(crates))
    if unknown:
        sys.exit(f"api: no library crate {', '.join(unknown)}")
    stale = []
    for name, features in sorted(crates.items()):
        if asked and name not in asked:
            continue
        path = SNAPSHOTS / f"{name}.txt"
        now = snapshot(name, features, target)
        if write:
            SNAPSHOTS.mkdir(exist_ok=True)
            path.write_text(now, encoding="utf-8", newline="\n")
            print(f"api: {name}: {now.count(chr(10))} lines written")
            continue
        was = path.read_text(encoding="utf-8") if path.is_file() else ""
        if now != was:
            diff = difflib.unified_diff(
                was.splitlines(), now.splitlines(), f"api/{name}.txt", "now", lineterm="", n=0,
            )
            print("\n".join(diff))
            stale.append(name)
        else:
            print(f"api: {name}: as snapshotted")
    gone = [] if asked else sorted({path.stem for path in SNAPSHOTS.glob("*.txt")} - set(crates))
    for name in gone:
        if write:
            (SNAPSHOTS / f"{name}.txt").unlink()
        else:
            stale.append(name)
            print(f"api: api/{name}.txt snapshots no crate")
    if stale:
        sys.exit(f"api: the public API changed in {', '.join(stale)}; run python3 scripts/api.py --write on Linux and commit api/")


if __name__ == "__main__":
    main()
