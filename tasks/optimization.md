# T56 Optimization

Source: `OPTIMIZATION.md`, 28 items, baseline `17f936b`. Work runs top to bottom; no item is skipped.
Each item ends with a codex review (`gpt-6-astra`, effort `max`, fast) in a herdr split pane on the right, five rounds at most. Rounds go to `tasks/reviews.md`.

Owner's calls, 2026-10-22:

- Full workspace split: `ely-core` plus one crate per vertical chapter, and `ely-desktop`.
- O-13: prepare the publish, do not publish. The owner runs `cargo publish`.
- OPTIMIZATION.md wins over the 500-line cap and the one-line comment rule: 800-line soft warning, rustdoc on every public item, a doctest per component.
- O-18, O-19, O-24 run on the Mac test host and WSL Ubuntu. O-27 needs the owner's signature. O-20 ships machine translations marked for native review.

Crates. Core is O-01's list and what it imports: chapters 1–14 (primitives, typography, layout, shell, buttons, forms, navigation, menus, overlays, feedback, data_display, lists, tables, motion), theme and i18n. Each other chapter is a crate `ely-<chapter>`, in PRD order: charts, finance, editor, terminal, git, debug, documents, collab, chat, agent, generative, media, files, messaging, mail, calendar, project, canvas, devtools, dashboard, settings, account, onboarding, interaction, maps, misc, tooling, rendering. `ely-desktop` holds the tray, the Dock badge and Omarchy. No vertical imports a later chapter, so the crates form a DAG.

## A. Ground

- [x] A1 O-14 Tests are hermetic: the font tests carry their own fixture, the pty test skips under root; 100% pass as root in `rust:1.95` on WSL.
- [x] A2 O-06 Caret versions for every third-party crate but `gpui-pre*`.
- [x] A3 O-05 One Markdown parser; the 128 KiB one-letter case reads in ≤100 ms in release. Find the quadratic first.
- [ ] A4 O-07 No duplicate in `cargo tree -d` starts at an Ely direct dependency.
  - Done: `iso_currency` drops its `iterator` feature, so `strum` 0.27 leaves; currencies list by ISO 4217 number.
  - Open, partial: only third-party crates at their newest releases still bring a second version, and Ely can align none without capping a version (O-06). `bit-set`/`bit-vec` 0.8 (RaTeX 0.1.15), the `digest` 0.11 line (`ratex-svg` 0.1.15, `rust-embed` 8.13), `phf` 0.11 (`isolang` 2.4, RaTeX). B9 may drop `rust-embed`; the rest needs upstream releases.
- [x] A5 O-08 Platform calls live in one platform module, on objc2; every `unsafe` block has a SAFETY line; `undocumented_unsafe_blocks` clean.

## B. Split

- [x] B1 O-01 Workspace skeleton: `crates/`, shared `[workspace.package]`, `[workspace.dependencies]`, lints; the gallery and docs examples become package `ely-gallery`; ADR for the split.
- [x] B2 O-01 `ely-core`: core chapters move; Latex leaves typography for documents; `init` binds core keys alone.
- [x] B3 O-01 Verticals, chapters 15–20: charts, finance, editor, terminal, git, debug.
  - From B2, decided in B3: core's built-in catalogs (`i18n/builtin.rs`) keep the verticals' strings. Splitting them needs a lookup API core does not have, and O-01 asks only that no vertical folder sits in core. `src/shipped.rs` reads each crate's sources by path.
- [x] B4 O-01 Verticals, chapters 21–27: documents, collab, chat, agent, generative, media, files.
- [x] B5 O-01 Verticals, chapters 28–34: messaging, mail, calendar, project, canvas, devtools, dashboard.
- [ ] B6 O-01 Verticals, chapters 35–44: settings, account, onboarding, interaction, maps, misc, tooling, rendering.
  - From B2: `maps/countries.geojson` sits in core's assets; it leaves with maps.
- [ ] B7 O-03 `ely-desktop`: tray, Dock badge, Omarchy; `cargo tree -p ely-core -i notify` and `-i zbus` print nothing.
- [ ] B8 O-02 Features: heavy dependencies default off where a crate can stand without them; a Button-only program pulls none of zbus, ratex-svg, mermaid-rs-renderer, alacritty_terminal, isolang, rqrr, notify.
- [ ] B9 O-04 Icons embed by use; fonts can be left out or replaced; a 5-icon release binary holds no other icon.
- [ ] B10 O-26 `cargo check -p ely-core` after one edit ≤20 s on the WSL box; the number recorded.
- [ ] B11 Gallery, site, check.sh, compat/kit, CI and AGENTS.md follow the new layout; the web gallery builds.
  - Found in B1: `e2e/docs.spec.ts`'s `GUIDES` lacks `gpui-kit`, added to `ORDER` in `b3985b9`; on Windows the Copy test reads CRLF from the clipboard. CI's site job runs neither.

## C. Public API

- [ ] C1 O-09 Public surface audit, core: non-essential items to `pub(crate)`.
- [ ] C2 O-09 Public surface audit, verticals.
- [ ] C3 O-11 Capability traits (size, disabled, selected, value change) after GPUI Kit's `Sizable`, `Disableable`, `Selectable`; every component with the capability implements them; a generic test compiles.
- [ ] C4 O-12 Container list in the repo; every listed component implements `ParentElement`; a test reads the list.
- [ ] C5 O-22 Line cap becomes an 800-line warning in check.sh; files under 40 lines merge by responsibility.
- [ ] C6 O-10 Rustdoc, core chapters 1–14 and theme, i18n: `#![deny(missing_docs)]`, a doctest per component.
- [ ] C7 O-10 Rustdoc, verticals 15–27.
- [ ] C8 O-10 Rustdoc, verticals 28–44 and desktop; `cargo doc` with docs.rs settings passes.

## D. Quality

- [ ] D1 O-15 `tests/` apps on public API alone: a form, a list with a table, a workspace with overlays.
- [ ] D2 O-16 clippy `expect_used`, `panic`, `unreachable` on library code; an allow list with reasons; CI counts and fails on a rise.
- [ ] D3 O-17 criterion benches: 100k-row table scroll, 10k-line editor typing, terminal scrollback, 2 MiB Markdown read; CI fails past 10% regression.
- [ ] D4 O-19 Role and name on every interactive component; a test holds the list; VoiceOver walk of form, menu, dialog on the Mac, written down.
- [ ] D5 O-20 Every declared locale has every key; a test walks them.
- [ ] D6 O-21 Each component's doc page names its platforms; a script matches them to `cfg(target_os)`.

## E. Release and CI

- [ ] E1 O-09 `cargo-public-api` snapshots in the repo; CI fails on an unsnapshotted change.
- [ ] E2 O-13 Crate metadata, CHANGELOG.md, `cargo publish --dry-run` per crate in DAG order; `cargo-semver-checks` in CI.
- [ ] E3 O-25 `cargo-deny` (licenses, advisories) and MSRV 1.95 jobs; all four O-25 jobs required.
- [ ] E4 O-24 check.sh on GitHub-hosted Linux, Windows and macOS; the self-hosted runner leaves the gate.
- [ ] E5 O-18 Pixel baselines on three platforms; each PR gets a diff report; past the threshold it fails.

## F. Docs and process

- [ ] F1 O-23 Root AGENTS.md ≤10 KB, each rule ≤3 lines; module rules in each crate's AGENTS.md; Decisions to `docs/adr/`.
- [ ] F2 O-27 Review record template under `tasks/`; core modules stay unstable until the owner signs.
- [ ] F3 O-28 README section on the agent-driven process, links checked.
- [ ] F4 Acceptance table, item by item, against OPTIMIZATION.md.
