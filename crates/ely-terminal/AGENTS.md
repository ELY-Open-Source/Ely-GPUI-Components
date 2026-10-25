# AGENTS.md: ely-terminal

Rules for `ely-terminal`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Stack

- Terminal: `alacritty_terminal` 0.26 (Apache-2.0, default features off, native only) for the grid, its parser and the pseudo-terminal; `futures` carries its events. `vte` 0.15 (Apache-2.0 OR MIT), the parser it re-exports, reads `AnsiText`'s codes alone, so colored output builds for wasm32.

## Rules

- A terminal takes Tab and Shift-Tab through bindings under `ElyTerminal`, ahead of focus moves; other keys go through `terminal::keys` in a key listener, and typed text through its input handler.
