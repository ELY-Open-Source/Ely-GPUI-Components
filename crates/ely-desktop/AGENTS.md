# AGENTS.md: ely-desktop

Rules for `ely-desktop`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Stack

- Omarchy: `notify` 8 (CC0-1.0, native only) watches Omarchy's current folder for `desktop::follow_omarchy` (`ely-desktop`); Omarchy's 22 themes' `colors.toml` (MIT, its `LICENSE` beside them) in `crates/ely-desktop/src/omarchy/themes` are the reader's fixtures, held to LF by `.gitattributes`, as Omarchy writes them, so a Windows checkout keeps their bytes.

## Rules

- Omarchy's theme reaches every component through `desktop::follow_omarchy`, which holds both modes' palettes while it follows: the theme's on its mode, Ely's on the other and while the folder has no colors; `stop_omarchy` gives Ely's back. Its palette passes `devtools::audit`, the table `tooling::A11yChecker` renders, or the theme is refused.
