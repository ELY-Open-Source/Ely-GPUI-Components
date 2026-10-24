#!/bin/sh
# Pre-review gate: build rules plus house rules.
set -eu
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
python3 scripts/panics.py
cargo test --workspace --exclude ely-gallery --lib --all-features --quiet
cargo test --workspace --exclude ely-gallery --doc --all-features --quiet
cargo test -p ely-gpui-component --all-features --test '*' --quiet
# The docs as docs.rs builds them: default features, `docsrs` set.
RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo doc --workspace --exclude ely-gallery --no-deps --quiet
# Ely beside the newest GPUI Kit: a release that moves Kit's gpui pin fails to resolve here.
kit="--manifest-path compat/kit/Cargo.toml"
newest=$(cargo search gpui-kit --limit 1 --color never | sed -n 's/^gpui-kit = "\([^"]*\)".*/\1/p')
[ -n "$newest" ] || { echo "crates.io named no gpui-kit release" >&2; exit 1; }
cargo generate-lockfile $kit
cargo update $kit -p gpui-kit --precise "$newest"
cargo fmt $kit --check
cargo clippy $kit --all-targets --target-dir target/kit -- -D warnings
cargo test $kit --target-dir target/kit --quiet
rustup target list --installed | grep -qx wasm32-unknown-unknown || {
  echo "the web lint needs wasm32: rustup target add wasm32-unknown-unknown" >&2
  exit 1
}
# gpui_web's wasm_thread asks for an unstable feature, as in scripts/web.sh.
RUSTC_BOOTSTRAP=1 ELY_GALLERY_ASSETS=https://example.invalid/ \
  cargo clippy -p ely-gallery --target wasm32-unknown-unknown -- -D warnings
python3 scripts/stories.py --check
fail=0
# A Button-only app needs ely-core alone; zbus comes with gpui's Linux backend.
for c in ratex-svg mermaid-rs-renderer alacritty_terminal isolang rqrr notify; do
  [ -z "$(cargo tree -p ely-core -e normal --target all -i $c 2>/dev/null || true)" ] || { echo "ely-core pulls $c"; fail=1; }
done
# Each published crate carries Ely's MIT notice, the root's own text.
for d in $(ls -d crates/*/ | grep -v ely-gallery); do
  cmp -s LICENSE "${d}LICENSE" || { echo "${d}LICENSE differs from LICENSE"; fail=1; }
done
files=$(git ls-files -co --exclude-standard '*.rs')
multi=$(awk 'FNR==1{prev=0} /^[[:space:]]*\/\/([^\/!]|$|\/\/)/ {if(prev) print FILENAME":"FNR; prev=1; next} {prev=0}' $files)
[ -z "$multi" ] || { echo "multi-line comments:"; echo "$multi"; fail=1; }
for f in $files AGENTS.md README.md TASKS.md tasks/*.md; do
  n=$(wc -l < "$f")
  [ "$n" -le 800 ] || echo "warning: $f has $n lines; split it by responsibility"
done
core=crates/ely-core/src
libs=$(ls -d crates/*/src | grep -v ely-gallery)
components=$(find src $libs -name '*.rs' ! -path "$core/theme/*" ! -path "$core/motion/*" ! -path '*/tests/*' ! -name 'tests.rs' ! -name 'testing.rs')
raw=$(awk '/#\[cfg\(test\)\]/{nextfile} /^[[:space:]]*\/\//{next} /(^|[^.[:alnum:]_])px\(/{print FILENAME":"FNR": "$0}' $components)
[ -z "$raw" ] || { echo "raw px in components:"; echo "$raw"; fail=1; }
deferred=$(grep -rn 'deferred(' src $libs --include='*.rs' | grep -v "^$core/primitives/layer.rs" || true)
[ -z "$deferred" ] || { echo "gpui deferred outside primitives::raise:"; echo "$deferred"; fail=1; }
instant=$(grep -rnE '(^|[^_[:alnum:]])time::(Instant|\{[^}]*Instant)' src crates || true)
[ -z "$instant" ] || { echo "std Instant panics in the browser, use web_time::Instant:"; echo "$instant"; fail=1; }
gallery=crates/ely-gallery
for f in $gallery/examples/docs/*.rs; do
  [ -e "$f" ] || continue
  n=$(basename "$f" .rs)
  grep -qx "path = \"examples/docs/$n.rs\"" $gallery/Cargo.toml && grep -qx "name = \"docs_$n\"" $gallery/Cargo.toml || { echo "$f needs [[example]] docs_$n in $gallery/Cargo.toml"; fail=1; }
done
exit $fail
