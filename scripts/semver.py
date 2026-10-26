#!/usr/bin/env python3
"""Fails a breaking change since a crate's last release that kept its version: each released library crate that may have changed since the base commit, against crates.io, by default, with no features and with all."""
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# cargo-semver-checks reads rustdoc's JSON, which moves with Rust; CI installs this one.
TOOL = "0.51.0"
FEATURES = [["--default-features"], ["--only-explicit-features"], ["--all-features"]]
# A change here can move every crate's API.
EVERYWHERE = {"Cargo.toml", "Cargo.lock"}


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)


def released(name):
    """Whether crates.io holds any release of `name`."""
    ask = urllib.request.Request(f"https://crates.io/api/v1/crates/{name}", headers={"User-Agent": "ely-semver (ELY-Open-Source/Ely-GPUI-Components)"})
    try:
        with urllib.request.urlopen(ask, timeout=30):
            return True
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return False
        raise


def rel(path):
    return Path(path).resolve().relative_to(ROOT).as_posix()


def packaged(name, package):
    """The files cargo packages for `name`, relative to the root; its whole folder when cargo cannot list them."""
    home = rel(Path(package["manifest_path"]).parent)
    prefix = "" if home == "." else home + "/"
    listed = subprocess.run(
        ["cargo", "package", "--list", "--allow-dirty", "-p", name], cwd=ROOT, capture_output=True, text=True,
    )
    if listed.returncode != 0:
        print(f"semver: cargo lists no package for {name}; every file in {home} counts", flush=True)
        return lambda path: path.startswith(prefix)
    files = {prefix + line.replace("\\", "/") for line in listed.stdout.splitlines() if line}
    return files.__contains__


def touched(base, packages):
    """The packages a file of which changed since the base, with every package that depends on one."""
    # A published crate reads only what cargo packages for it, `#[path]` and `include!` sources too.
    # --no-renames names a moved file's old path too, as a deletion.
    diff = git("diff", "--name-only", "--no-renames", base)
    if diff.returncode != 0:
        sys.exit(f"semver: cannot diff against {base}: {diff.stderr.strip()}")
    changed = set(diff.stdout.splitlines())
    if changed & EVERYWHERE:
        return set(packages)
    hit = {name for name, package in packages.items() if any(map(packaged(name, package), changed))} if changed else set()
    # A file gone from the tree is in no listing; the package whose folder held it may have read it.
    homes = {name: rel(Path(package["manifest_path"]).parent) for name, package in packages.items()}
    for path in changed:
        if not (ROOT / path).exists():
            owners = [name for name, home in homes.items() if home == "." or path.startswith(home + "/")]
            if owners:
                hit.add(max(owners, key=lambda name: 0 if homes[name] == "." else len(homes[name])))
    by_home = {Path(package["manifest_path"]).resolve().parent: name for name, package in packages.items()}
    uses = {
        name: {by_home.get(Path(dep["path"]).resolve()) for dep in package["dependencies"] if dep.get("path") and dep["kind"] != "dev"}
        for name, package in packages.items()
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
    packages = {package["name"]: package for package in meta["packages"]}
    libraries = {
        name: package
        for name, package in packages.items()
        if any("lib" in target["kind"] for target in package["targets"])
    }
    changed = touched(base, packages) & set(libraries)
    unreleased = {name for name in changed if not released(name)}
    for name in sorted(unreleased):
        print(f"semver: {name} has no release on crates.io; nothing to break")
    shared = sorted(changed - unreleased)
    print(f"semver: {len(shared)} of {len(libraries)} crates may have changed since {base}; each checks against its newest release")
    for features in FEATURES:
        # Without a `default` feature, the defaults and none build the same API; `default = []` still sets cfg(feature = "default").
        names = [name for name in shared if features[0] != "--only-explicit-features" or "default" in libraries[name]["features"]]
        if not names:
            continue
        print(f"semver: {len(names)} crates, {features[0]}", flush=True)
        packages = [arg for name in names for arg in ("-p", name)]
        run = subprocess.run(
            ["cargo", "semver-checks", *packages, *features], cwd=ROOT, env=env,
        )
        if run.returncode != 0:
            sys.exit(f"semver: a breaking change under {features[0]} needs its crate's version raised past its last release")


if __name__ == "__main__":
    main()
