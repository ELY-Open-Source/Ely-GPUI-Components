# 0001 A Cargo workspace

Date: 2026-10-22. Status: accepted. Source: OPTIMIZATION.md, O-01, O-02, O-03, O-26.

## Context

One crate held 44 chapters: 974 files, 231,710 lines. A downstream app that drew one Button compiled the terminal, mail, finance and their dependencies. After one touched file, `cargo check --lib` took 63 s on two cores, since one crate compiles on one thread.

## Decision

The repository is a Cargo workspace.

- `crates/ely-core`: chapters 1–14, theme and i18n.
- `crates/ely-<chapter>`: one crate per later chapter, in PRD order. A chapter imports only earlier ones, so the crates form a DAG.
- `crates/ely-desktop`: the tray, the Dock badge and Omarchy.
- `crates/ely-gallery`: the gallery, binary `gallery`, and the docs examples. It is never published.

The root manifest holds `[workspace.package]` (edition, Rust version, license, repository), `[workspace.dependencies]` for what more than one crate uses, gpui's exact pin among them, `[workspace.lints]` and the profiles. Each crate keeps its own version.

Two items move home so the chapters stay a DAG: Latex moves from typography to `documents::Latex`, since it typesets through the documents' svg reader; WCAG contrast (`luminance`, `contrast`, `over`, `grades`) moves from devtools to `theme`, which core's palette tests need. A core item a vertical uses turns `pub`.

## Steps

B1 moves the gallery and the docs examples; the library stays at the root as `ely-gpui-component`. B2 moves the core chapters; the root re-exports core's modules, so the verticals' `crate::buttons` paths hold until B3–B6 move them out. B7 moves the desktop parts. Once the last chapter leaves, the root holds no package.

## Consequences

- `cargo run -p ely-gallery` opens the gallery; `--example docs_NAME` runs a docs example.
- Lints and checks run with `--workspace`.
- A downstream app depends on each crate it uses.
