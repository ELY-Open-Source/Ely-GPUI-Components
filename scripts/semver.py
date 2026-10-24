#!/usr/bin/env python3
"""Fails a breaking change whose crate kept its version: each library crate the base also has and that may have changed, against the base, by default, with no features and with all."""
import json
import os
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# cargo-semver-checks reads rustdoc's JSON, which moves with Rust; CI installs this one.
TOOL = "0.51.0"
FEATURES = [["--default-features"], ["--only-explicit-features"], ["--all-features"]]
# A change here can move every crate's API.
EVERYWHERE = {"Cargo.toml", "Cargo.lock"}


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


def touched(base, libraries):
    """The libraries whose sources, manifest or build script changed since the base, with every crate that depends on one."""
    diff = git("diff", "--name-only", base)
    if diff.returncode != 0:
        sys.exit(f"semver: cannot diff against {base}: {diff.stderr.strip()}")
    changed = set(diff.stdout.splitlines())
    if changed & EVERYWHERE:
        return set(libraries)
    homes = {name: Path(package["manifest_path"]).resolve().parent for name, package in libraries.items()}
    hit = set()
    for name, home in homes.items():
        prefix = "" if home == ROOT else home.relative_to(ROOT).as_posix() + "/"
        own = {f"{prefix}Cargo.toml", f"{prefix}build.rs"}
        if any(path.startswith(f"{prefix}src/") or path in own for path in changed):
            hit.add(name)
    by_home = {home: name for name, home in homes.items()}
    uses = {
        name: {by_home.get(Path(dep["path"]).resolve()) for dep in package["dependencies"] if dep.get("path") and dep["kind"] != "dev"}
        for name, package in libraries.items()
    }
    grew = True
    while grew:
        more = {name for name, deps in uses.items() if deps & hit} - hit
        hit |= more
        grew = bool(more)
    return hit


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
    meta = json.loads(meta.stdout)
    # Unset, the tool builds each crate in a target of its own, gpui and all, once per crate and tree.
    env = {**os.environ, "CARGO_TARGET_DIR": os.environ.get("CARGO_TARGET_DIR", meta["target_directory"])}
    libraries = {
        package["name"]: package
        for package in meta["packages"]
        if any("lib" in target["kind"] for target in package["targets"])
    }
    before = named(base)
    for name in sorted(set(libraries) - before):
        print(f"semver: {name} is new since {base}; nothing to break")
    shared = sorted((set(libraries) & before) & touched(base, libraries))
    print(f"semver: {len(shared)} of {len(libraries)} crates may have changed since {base}")
    for features in FEATURES:
        # With no default features, the defaults and none build the same API.
        names = [name for name in shared if features[0] != "--only-explicit-features" or libraries[name]["features"].get("default")]
        if not names:
            continue
        print(f"semver: {len(names)} crates, {features[0]}", flush=True)
        packages = [arg for name in names for arg in ("-p", name)]
        run = subprocess.run(
            ["cargo", "semver-checks", *packages, *features, "--baseline-rev", base], cwd=ROOT, env=env,
        )
        if run.returncode != 0:
            sys.exit(f"semver: a breaking change under {features[0]} needs its crate's version raised")


if __name__ == "__main__":
    main()
