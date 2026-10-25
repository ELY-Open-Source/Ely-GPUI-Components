#!/bin/sh
# Shoots the gallery at BASE and here on this machine and compares them: scripts/pixels.sh BASE [OUT].
# A Linux runner with no display runs it under xvfb-run.
# Hosted screens are 1024x768, so the window asks for less than the system would trim it to.
set -eu
base=${1:?usage: scripts/pixels.sh BASE [OUT]}
out=${2:-target/pixels}
py=$(command -v python3 || command -v python) || { echo "no python3 or python on PATH" >&2; exit 1; }
case $base in *[!0]*) ;; *) echo "pixels: base $base is no commit, as on a branch's first push; nothing to compare"; exit 0 ;; esac
git cat-file -e "$base^{commit}" || { echo "pixels: no commit $base to compare against" >&2; exit 1; }
if ! git cat-file -e "$base:scripts/pixels.py" 2>/dev/null; then
  echo "pixels: $base predates frozen captures; nothing to compare"
  exit 0
fi
exe=""
[ "$(uname -o 2>/dev/null)" = Msys ] && exe=.exe
rm -rf "$out" && mkdir -p "$out"
here=$(pwd)
tree="$out/base-tree"
git worktree add --detach "$tree" "$base" >/dev/null
trap 'git -C "$here" worktree remove --force "$tree"' EXIT
# One target serves both builds; each gallery is copied out before the other overwrites it.
export CARGO_TARGET_DIR="$here/target"
for side in head base; do
  dir=$here
  [ $side = base ] && dir=$here/$tree
  # Cargo keys a path crate by its place in the workspace, so both trees share fingerprints; fresh times rebuild this one.
  (cd "$dir" && git ls-files -z | xargs -0 touch && cargo build -p ely-gallery --quiet)
  cp "target/debug/gallery$exe" "$out/gallery-$side$exe"
done
for side in head base; do
  dir=$here
  [ $side = base ] && dir=$here/$tree
  (cd "$dir" && "$here/$out/gallery-$side$exe" --window 1000x600 --still --capture "$here/$out/$side")
done
accept=""
if git log --format=%B "$base..HEAD" | grep -q '^Pixels-accepted:'; then
  accept=--accept
fi
"$py" scripts/pixels.py "$out/base" "$out/head" "$out/report" $accept
