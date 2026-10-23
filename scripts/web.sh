#!/bin/sh
# Builds the browser gallery; ELY_GALLERY_ASSETS addresses <out>/assets/.
set -eu
out=$(mkdir -p "${1:?usage: scripts/web.sh <out dir>}" && cd "$1" && pwd)
: "${ELY_GALLERY_ASSETS:?set ELY_GALLERY_ASSETS to the address of <out>/assets/, ending in /}"
cd "$(dirname "$0")/.."
version=$(sed -n '/^name = "wasm-bindgen"$/{n;s/^version = "\(.*\)"$/\1/p;}' Cargo.lock)
wasm-bindgen --version | grep -qx "wasm-bindgen $version" || {
  echo "wasm-bindgen $version needed: cargo install wasm-bindgen-cli --version $version --locked" >&2
  exit 1
}
command -v wasm-opt >/dev/null || { echo "wasm-opt needed: brew install binaryen, or npm i -g binaryen" >&2; exit 1; }
# gpui_web's wasm_thread needs an unstable feature, as in Zed.
RUSTC_BOOTSTRAP=1 cargo build -p ely-gallery --target wasm32-unknown-unknown --profile web
wasm="${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/web/gallery.wasm"
wasm-bindgen "$wasm" --out-dir "$out" --out-name gallery --target web --no-typescript \
  --remove-name-section --remove-producers-section
# Cloudflare serves no file over 25 MiB; -Oz leaves room.
wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
  --enable-mutable-globals --enable-reference-types --enable-multivalue \
  "$out/gallery_bg.wasm" -o "$out/gallery_bg.wasm"
cp crates/ely-gallery/index.html "$out/"
rm -rf "$out/assets"
cp -R crates/ely-gallery/assets "$out/assets"
size=$(wc -c < "$out/gallery_bg.wasm" | tr -d ' ')
[ "$size" -le 26214400 ] || { echo "gallery_bg.wasm is $size bytes, over Cloudflare's 25 MiB" >&2; exit 1; }
echo "gallery_bg.wasm: $size bytes"
