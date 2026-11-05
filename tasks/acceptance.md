# Acceptance: OPTIMIZATION.md, item by item

Checked 2026-10-26 at `d77a171`. Each row names the acceptance test from OPTIMIZATION.md, the evidence, and where the work is recorded in `tasks/optimization.md`. "Met" means the test passes as written; "Open" names what is missing and why.

| Item | Acceptance | Evidence | Task | Status |
| --- | --- | --- | --- | --- |
| O-01 | No vertical folder under `ely-core/src` | `crates/ely-core/src` holds chapters 1–14 and `theme`, `motion`, `i18n`, `platform`; each vertical is its own crate | B1–B6 | Met |
| O-02 | A Button-only program pulls none of the seven heavy crates | A Button app on `ely-core` with its `platform` feature pulls none of the seven on macOS and Windows (`cargo tree --features platform --target <triple> -e normal -i <crate>`). On Linux it pulls zbus: gpui's Linux backend links it (`gpui-pre-linux` through `accesskit_unix`, ashpd and oo7), and every gpui window on Linux takes that backend; the other six are absent. `isolang` is gone altogether. `ely-core`'s own graph, dev edges too, holds none of the seven, which `scripts/check.sh` holds | B8, A4 | Partial: met on macOS and Windows; on Linux zbus stays while gpui's Linux backend links it
| O-03 | `cargo tree -p ely-core -i notify` and `-i zbus` print nothing | Both commands, as written, find neither crate in `ely-core`'s graph (`--target x86_64-unknown-linux-gnu` too): Omarchy, the tray and the Dock badge live in `ely-desktop`; gpui_platform sits behind `ely-core`'s off-by-default `platform` feature (ADR 0059); the `five_icons` example, its last dev-dependency user, needs that feature | B7 | Met
| O-04 | A 5-icon release binary holds no other icon | Each icon is its own `static`; `cargo build -p ely-core --release --features platform --example five_icons` holds five SVGs | B9 | Met |
| O-05 | One Markdown parser; the 128 KiB case ≤100 ms in release | pulldown-cmark alone; 20 ms in release | A3 | Met |
| O-06 | `"=` only for the three `gpui-pre` crates | `grep -c '"=' Cargo.toml` gives 3: gpui-pre, gpui-pre-platform, gpui-pre-web, at `=0.3.10` | A2 | Met |
| O-07 | No `cargo tree -d` group starts at an Ely direct dependency | 11 groups remain, every one through RaTeX 0.1.15, its newest release: bit-set 0.8 (its fancy-regex 0.14), phf 0.11 (ratex-font), the digest 0.11 line (rust-embed in ratex-katex-fonts). Ely's own, rust-embed and isolang, are gone. A RaTeX change is ready and tested (phf 0.13, fancy-regex 0.20, fonts through `include_bytes!`); patched in, no group comes through RaTeX on macOS or Windows, and on Linux bit-set 0.8 stays until fancy-regex takes 0.10 | A4 | Open: erweixin/RaTeX#179 sent; on Linux, fancy-regex#291 (bit-set 0.11, parked on its MSRV) and a gpui on wgpu with bit-set 0.11 |
| O-08 | `unsafe` only in platform modules; `undocumented_unsafe_blocks` clean | Every `unsafe` block sits in a `platform` module with a `// SAFETY:` line; clippy enforces it in the gate | A5 | Met |
| O-09 | CI fails on a public change missing from the snapshot | `scripts/api.py` against `api/`; CI's `api` job, a required check on `main` | C1, C2, E1 | Met |
| O-10 | `missing_docs` clean; a doctest per component; docs.rs builds | `#![deny(missing_docs)]` in every library crate; doctests run in the gate; the docs.rs build runs in the gate | C6–C8 | Met |
| O-11 | Every capable component implements its trait; generic code compiles | `Sizable`, `Disableable`, `Selectable`, `Changeable`; 30, 19, 5 and 80 impls; a generic test compiles over them | C3 | Met |
| O-12 | A container list, 100% `ParentElement`, held by a test | `src/containers.rs` lists 87 containers; its test reads every impl | C4 | Met |
| O-13 | `cargo add ely-core` works; a breaking change without a version bump fails CI | All 31 library crates are on crates.io, `ely-core` at 0.2.0 and the rest at 0.1.1; `cargo add ely-core ely-gpui-component` in a fresh project resolves both. `scripts/semver.py` in CI's `semver` job, a required check | E2 | Met |
| O-14 | Tests pass as root in `rust:1.95` (now the declared minimum, `rust:1.98`) | Fonts come from fixtures; the pty test skips under root and says so. On 2026-10-24, 2,216 passed as root in `rust:1.95`; since gpui 0.3.10 needs 1.98, on 2026-10-26 the 31 library crates' 2,261 lib tests passed as root in `rust:1.98` (Docker on WSL, `--all-features`), none failed | A1 | Met |
| O-15 | Removing a documented public item breaks a test | `tests/` holds three apps on the public API; the `api` job fails on any removed item | D1, E1 | Met |
| O-16 | CI prints panic counts and fails on a rise | `scripts/panics.py` on Linux, Windows and macOS, each against the base commit | D2 | Met |
| O-17 | Benchmarks in CI; a regression past 10% fails | `benches/budget.rs`, four cases; CI's `bench` job benches base and head on one runner | D3 | Met |
| O-18 | A three-platform diff report per PR; past the threshold it blocks the merge | `scripts/pixels.sh` and `pixels.py` on Linux, Windows and macOS; the three `pixels` legs are required checks | E5, E3 | Met |
| O-19 | Every interactive component named, held by a test; VoiceOver walk of form, menu, dialog recorded | `gallery --audit` in CI walks 2,272 stops on 44 pages, 0 faults. The VoiceOver walk of a form, a menu and a dialog is `tasks/voiceover.md`; what it found in Ely is fixed and tested, and two gpui limits are recorded there | D4 | Met |
| O-20 | Every declared locale has every key, by a test | Seven locales, no fallback; a test holds each to English's keys and arguments | D5 | Met |
| O-21 | Each component's doc names its platforms, matched to its cfgs by a script | 13 items say `Platforms:`; `scripts/platforms.py` in the gate. The fix for review round 5's last finding came after the round cap | D6 | Met |
| O-22 | `check.sh` never fails on line count; files under 40 lines are real units | An 800-line warning; small files merged by responsibility | C5 | Met |
| O-23 | Root `AGENTS.md` ≤10 KB, each rule ≤3 lines | 5,340 bytes; `check.sh` fails past 10 KB or a 300-character rule | F1 | Met |
| O-24 | Any platform's `check.sh` failure blocks the merge, without a self-hosted runner | `gate` on GitHub-hosted Linux, Windows and macOS, three required checks | E4, E3 | Met |
| O-25 | Four jobs exist and are required | `api`, `semver`, `deny`, `msrv`, all required on `main` | E3 | Met |
| O-26 | `cargo check -p ely-core` ≤20 s after one edit | 1.30 s incremental, 3.47 s without incremental, on the WSL box | B10 | Met |
| O-27 | Every crate marked stable has a human review record and outside feedback | `tasks/stability.md` and `scripts/stability.py` enforce it; no crate is marked stable yet, so none is due. OPTIMIZATION.md's goal, a human sign-off on the core API, waits for the owner | F2 | Open: the owner's sign-off |
| O-28 | README has the section; its links work | "How it is built" in README; `scripts/links.py` checks every relative link in the gate | F3 | Met |

Count: 25 met, one met in part (O-02, on Linux zbus through gpui's Linux backend), two open: O-07 on erweixin/RaTeX#179 and fancy-regex#291, O-27 on the owner's sign-off.
