#!/usr/bin/env python3
"""Fails a breaking change whose crate kept its version: each library crate the base also has, against the base, by default, with no features and with all."""
import json
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# cargo-semver-checks reads rustdoc's JSON, which moves with Rust; CI installs this one.
TOOL = "0.51.0"
FEATURES = [["--default-features"], ["--only-explicit-features"], ["--all-features"]]


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)


def named(base):
    """The package names the base's manifests declare, wherever their crates sat."""
    tree = git("ls-tree", "-r", "--name-only", base)
    if tree.returncode != 0:
        sys.exit(f"semver: cannot list {base}: {tree.stderr.strip()}")
    names = set()
    for path in tree.stdout.splitlines():
        if path == "Cargo.toml" or path.endswith("/Cargo.toml"):
            manifest = tomllib.loads(git("show", f"{base}:{path}").stdout)
            if "package" in manifest:
                names.add(manifest["package"]["name"])
    return names


def main():
    if "--tool" in sys.argv:
        print(TOOL)
        return
    if len(sys.argv) != 2:
        sys.exit("semver: give the base commit")
    base = sys.argv[1]
    if base.strip("0") == "":
        print(f"semver: base {base} is no commit, as on a branch's first push; nothing to compare")
        return
    if git("cat-file", "-e", f"{base}^{{commit}}").returncode != 0:
        sys.exit(f"semver: no commit {base} to compare against")
    meta = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    )
    before = named(base)
    shared, new = [], []
    for package in json.loads(meta.stdout)["packages"]:
        if any("lib" in target["kind"] for target in package["targets"]):
            (shared if package["name"] in before else new).append(package["name"])
    for name in sorted(new):
        print(f"semver: {name} is new since {base}; nothing to break")
    if not shared:
        print(f"semver: no library crate is in both {base} and here; nothing to compare")
        return
    packages = [arg for name in sorted(shared) for arg in ("-p", name)]
    for features in FEATURES:
        print(f"semver: {len(shared)} crates, {features[0]}", flush=True)
        run = subprocess.run(
            ["cargo", "semver-checks", *packages, *features, "--baseline-rev", base], cwd=ROOT,
        )
        if run.returncode != 0:
            sys.exit(f"semver: a breaking change under {features[0]} needs its crate's version raised")


if __name__ == "__main__":
    main()
