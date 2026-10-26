# AGENTS.md

Ely GPUI Component. A component library for GPUI, in light and dark.

This file holds what every change needs. Before working in a crate or folder, read its own `AGENTS.md`; every crate also follows `crates/ely-core/AGENTS.md`, the component rules. Decisions live in `docs/adr/`.

## Stack

- Rust 1.98 (gpui 0.3.10's floor on macOS), edition 2024. A Cargo workspace (`docs/adr/0001-workspace.md`): `ely-core` (chapters 1–14), one crate per later chapter in `crates/ely-<chapter>`, `ely-desktop`, the root `ely-gpui-component` re-exporting all, and `crates/ely-gallery`.
- gpui, gpui_platform and gpui_web come from crates.io's `gpui-pre` at `=0.3.10`, the pin GPUI Kit 0.8.1 holds. These three alone pin with `=`; every other dependency takes a caret, and `Cargo.lock` holds the build.
- A dependency beyond gpui is named, with its license and why it is there, in the `AGENTS.md` of the crate that brings it. A new one adds its line there and passes `cargo deny`.
- `ely_core::init` loads assets and fonts and binds core keys; `ely_gpui_component::init` calls it, then binds the editor's, terminal's and documents' keys.

## Commands

- `sh scripts/check.sh` is the gate: fmt, clippy, the wasm32 lint, the docs.rs build, lib, doc and app tests, the panic list and the house scans. It fails by its exit status; a pipe into `tail` hides it.
- CI runs it on GitHub-hosted Linux, Windows and macOS, beside `api`, `semver`, `deny`, `msrv`, `pixels`, `bench` and `site`; `scripts/AGENTS.md` says what each checks and how to run it.
- `cargo run -p ely-gallery` opens the gallery; `crates/ely-gallery/AGENTS.md` names its flags, captures and the web build. The site's commands are in `frontend/AGENTS.md`.
- After each task item: `rm -rf target/debug/incremental target/kit/debug/incremental` and `cargo clean -p ely-gpui-component`. Old builds of the crate filled the disk once.

## Layout

- `crates/ely-core/src/<chapter>/`: chapters 1–14 of `gpui-components.md`, with `theme`, `motion`, `i18n`, `platform` and the bundled `assets`.
- `crates/ely-<chapter>/src`: a vertical chapter. It imports core's modules and earlier verticals under their chapter names, so `crate::theme` reads as before. Chapters import only earlier chapters.
- `src/lib.rs`: `ely-gpui-component`, which re-exports every crate under its chapter name; its `init` binds every crate's keys. `tests/` holds three apps an outside crate would write.
- `crates/ely-gallery`: the gallery, one page per chapter. `compat/kit`: Ely beside GPUI Kit. `api/`: the public API snapshots. `docs/adr/`: decisions.
- `TASKS.md` and `tasks/`: progress. Every PRD entry has a line and a tag; `tasks/reviews.md` keeps every review round.

## Rules

- Fail fast. No silent fallbacks. Log state changes with `log`. Fix the root cause, not the symptom.
- A state live data or the outside world can reach logs an error naming its owner and draws a visible state, and the app runs on. An assert stays only on a caller's argument or a fact true by construction.
- Every `expect`, `panic!` and `unreachable!` in library code is a row of `scripts/panics.tsv` with its reason; `scripts/panics.py` holds each host to its rows.
- Colors, sizes, radii and shadows come from `cx.theme()`; no raw `px()` inside components. Gaps and padding use gpui's rem helpers; borders use its fixed-pixel ones.
- One component, one home. Duplicates point to it.
- A handler a component needs is an argument of `new`, so leaving it out fails to compile.
- Code that needs `unsafe` lives in a crate's `platform` module, each block with a `// SAFETY:` line; components call its safe functions.
- An item public only for another Ely crate carries `#[doc(hidden)]`; a `pub` item no path reaches is `pub(crate)`.
- A public API change commits its new `api/` snapshot (`scripts/api.py --write`, on Linux).
- A change an app would notice adds a line under Unreleased in `CHANGELOG.md`; a breaking one raises its crate's version, the minor below 1.0.
- A file holds one responsibility; `check.sh` warns past 800 lines. A file under 40 lines is a module entry or a type of its own.
- Comments are one short line, and rare. Every public item carries rustdoc, and each component's doc holds an example that compiles and runs.
- Copy a component draws reads `i18n::text`; `src/shipped.rs` fails on fixed English in a shipped component. A missing key fails, with no fallback.
- Every pressable thing is a Tab stop with a role and a name on the element that takes focus; `gallery --audit` proves it on Windows.
- An item that runs on fewer than macOS, Windows, Linux and the web says `Platforms:` in its doc; `scripts/platforms.py` holds the line to its cfgs.
- Time reads `web_time::Instant`, and a drawn moment `typography::format::now()`. What cannot build for wasm32 is absent there through `cfg`, never faked.
- Tests never depend on the host: what a test asserts holds on any machine. The few that read host fonts or a clock, and why they stay sound, are named in `crates/ely-core/AGENTS.md`.
- Every crate stays unstable until the owner signs `tasks/signoffs/<crate>.md`; a vertical crate also needs an outside user's feedback. A model never writes a sign-off or marks a crate stable (`tasks/stability.md`).
- A rule learned while working goes into the `AGENTS.md` nearest the code it governs; a choice between options goes into `docs/adr/`.
