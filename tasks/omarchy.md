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
- [x] O5 Acceptance: the checklist, item by item
- [x] O6 CI: Windows' CRLF checkout changed the fixtures' bytes, and a test's replace missed; `.gitattributes` holds `src/settings/omarchy/themes` to LF
- [x] O7 Linux windows: `gpui_platform` lacked `wayland` and `x11`, so on Linux the gallery and every example ran headless and opened no window (found by a WSLg probe); the manifest, CI's Linux job, README, the install guide and AGENTS.md take both; CI's Linux leg then failed in `yeslogic-fontconfig-sys`'s build script, so it installs `libfontconfig-dev` too

## Acceptance

| # | What was asked | Where | Proof | State |
|---|----------------|-------|-------|-------|
| 0 | A gate that passes | O0: `gpui-pre =0.3.9` as GPUI Kit 0.8.0 pins it; the web lint and house rules fixed | `scripts/check.sh` exit 0; the web bundle 22.8 MB | Done |
| 1 | Every component follows Omarchy's theme | `settings::follow_omarchy` sets the palette every component reads | A scan of `src` outside `theme` and tests for `rgb(`, `rgba(`, `hsla(`, `black()`, `white()` and hex strings finds no theme color: the color picker's color-space gradients, the reader's own black and white lift ends, `diagram_svg`'s export, light by design, and the terminal's colors a program sets | Done |
| 2 | Colors as Omarchy resolves them | `settings/omarchy/colors.rs` | Omarchy's 22 themes keep their page, accent and selection; ANSI-only and loose files; mode order; bash's last line; `-f` | Done |
| 3 | Text that reads | `settings/omarchy/palette.rs`, `devtools::audit` | Every theme passes the audit and a table written apart from it; a gray page lifts toward black | Done |
| 3a | One audit table | O1: `devtools::audit`, rendered by `tooling::A11yChecker` and checked by the reader | `tooling/a11y.rs` holds no table; the audit's five tests moved with it | Done |
| 3b | The hues a theme lacks named | `OmarchyTheme::missing`, logged as a warning | `a_theme_that_lacks_a_hue_is_told_apart_from_one_that_has_it` | Done |
| 4 | The terminal as Omarchy's | `Palette::ansi` | Each of the 16 colors equals Ghostty's from Omarchy's template; a mutant with `fg_subtle` fails | Done |
| 5 | Switches followed | `follow.rs` | Omarchy's swap order, a 200 ms slow swap, no colors, a bad theme, stop; a mutant that drops events fails | Done |
| 6 | No self-made reloads on Linux | `is_read`, `Following::reads` | Under WSL, reading the theme starts no read; without the filter 31 reads became 61 | Done |
| 7 | Fail fast, observable | errors name the file and key; logs at each read, wear and stop | `the_cause_is_named_...`, `following_twice_...` | Done |
| 8 | One call to opt in | `follow_omarchy`, `stop_omarchy`, `omarchy_current_dir`, `read_omarchy_theme` | the theme guide, README | Done |
| 9 | Ordered list, one item at a time, review in a herdr split | this file, `tasks/reviews.md` | codex `gpt-6.1-sol` xhigh fast; O0 2 rounds, O1 1, O2 2, O3 2, O4 2 | Done |
| 10 | check.sh, files under 500 lines, one-line comments, no dead code | `scripts/check.sh` | passes on Windows at each item; the crate's tests on Linux (WSL) | Done |
| 11 | AGENTS.md follows the stack | Stack, Rules, Decisions | O0, O4 | Done |
| 12 | A live Linux window | O7 | Under WSLg's X11, a window followed a folder from Tokyo Night to Catppuccin Latte in `omarchy-theme-set`'s order and repainted, shot by `import`; Omarchy's Hyprland itself not run | Done, Hyprland open |
| 13 | ElyOffice and ELYTerminal on the library's reader | — | Not needed: both build on this library (the owner, 2026-10-21) | Closed |

Notes: the terminal draws on `surface`, Ely's own choice, so its ground sits 4% from Omarchy's background. CI's Mac job sat queued all day and its Linux leg fails a font test older than this work. Three O2 fixes were proven red only by the reviewer's probes.
