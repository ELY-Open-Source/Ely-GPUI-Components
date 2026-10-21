# Omarchy

Started 2026-10-21. Every component wears Omarchy's current theme when the app asks. Measured before work:

- Omarchy 4 writes the current theme to `~/.local/state/omarchy/current/theme/colors.toml`, under `$HOME` whatever `XDG_STATE_HOME` says. `omarchy-theme-set` alone writes it: it fills `next-theme`, removes `theme`, moves `next-theme` in, then writes `theme.name`. `omarchy-theme-color` resolves the keys: loose quotes, `colorN` and short names, `purple`, `selection_background`; mode from `mode`, `theme_type`, a `light.mode` file, then the background's brightness, a sum past 382 reading light.
- ElyOffice maps that file to an Ely `Palette` in `ely-ui`, lifting colors until the library's contrast audit passes, and watches `current` through `notify`. It copies the audit's pairs, which `tooling::a11y` keeps private, and chapter 35 cannot import chapter 43.
- ELYTerminal reads `ghostty.conf` twice a second, for terminal colors alone.

The owner chose (2026-10-21): an app opts in with one call; `notify` watches; the library alone, ElyOffice and ELYTerminal later; no gallery story.

Rules: items run in PRD chapter order. Each passes `scripts/check.sh` and a codex review (`gpt-6.1-sol`, effort `xhigh`, fast) in a herdr split, three rounds at most, then commits.

## Items

- [x] O0 gpui `gpui-pre =0.3.9`, as GPUI Kit 0.8.0 pins it: `scripts/check.sh` could not resolve Kit 0.8.0 beside 0.3.8 (the owner chose to bump first, 2026-10-21); the web lint also failed at `shell::native`'s `Route::pick`, dead on wasm32 since the tray commit `5be304c`; its house rules failed on four raw `px(` conversions (`documents`, `typography::math`), `shell/title_tests.rs` outside a tests folder and std's `Instant` in `shell::native`'s tests, unseen while CI's Mac job sat queued. The crate built with no change; the web bundle is 22.8 MB after `wasm-opt`, 20.0 at K01
- [x] O1 Devtools (ch. 33): `devtools::audit` and its `Check` move out of `tooling::a11y`, which renders them; one table of pairs and needs
- [x] O2 Settings (ch. 35): `settings::read_omarchy_theme` reads a current folder's `colors.toml` as `omarchy-theme-color` resolves it, lifts the palette until `devtools::audit` passes and refuses one that cannot, and names the hues Ely's colors stand in for; tested on Omarchy's 22 themes
- [x] O3 Settings (ch. 35): `settings::follow_omarchy` and `stop_omarchy`: the theme worn on its mode now and after each switch, watched through `notify`; a bad theme logs and the colors stay; tested on Windows and on Linux (WSL)
- [x] O4 Theme (ch. 39): the Theme guide, README, AGENTS.md; `tasks/ch31-43.md` stays, as Omarchy is no PRD entry and T55 carries it
- [ ] O5 Acceptance: the checklist, item by item
