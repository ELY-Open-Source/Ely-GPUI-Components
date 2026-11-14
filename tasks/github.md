# GitHub round, 2026-10-03

- [x] G1 PR #1: build on Windows with `escape_args`; review, verify, merge, answer CodeRabbit
- [x] G2 Issue #2: tree guides line up under the chevron and run unbroken across rows
- [x] G3 Issue #3: Checkbox, Switch, Radio, Slider and Select name their role, label and state
- [x] G4 Push, comment on #2 and #3, close #2
- [x] G5 PR #4: `Icon::from_path` draws an app's SVG; a path the asset source lacks fails at render

# GitHub round, 2026-10-05

- [x] G6 Issue #7: an IME composition restarts the caret's blink, as every edit does; its ranges are logged; tested
- [x] G7 Issue #6: `shell::ColumnShell`, a sidebar that runs to the window's top with the traffic lights, beside one or two panes that bring their own headers; tested
- [x] G8 Issue #6: a Layout story, `stories.json`, AGENTS.md and the PRD line; the story runs live on the site
- [x] G9 Push, deploy, reply on #6 and #7, close #6

# GitHub round, 2026-10-08

- [x] G10 CI: `cargo fmt` on the files the last commits left unformatted; then the narrow-layout commits: fmt, and `Theme::hairline` for their raw px
- [x] G11 Issue #7: a caret is a whole number of device pixels wide at a device pixel, so it never shows thin then thick; tested
- [x] G12 Issue #7: an IME asks for its candidate box past the laid-out text; the box clamps to the last layout, as `bounds_for` does; tested
- [x] G13 Issue #12: a floating option list occludes, so the wheel scrolls the list and not the page under it; tested
- [x] G14 Issue #7: BlockEditor scrolls in its own `ScrollArea` with its scrollbar and keeps the caret in view as blocks are added; tested
- [x] G15 Issue #7: BlockEditor turns into Markdown and back; tested. Six reviews: the fifth finding (a slash ended an unquoted HTML value) fixed and tested; the sixth, at the owner's word, passed
- [x] G16 Issue #7: a selection across blocks: drag or Shift-press selects, copy, cut, delete and typing act on it, undo restores; tested
- [x] G17 Push, reply on #7 and #12

# GitHub round, 2026-11-14

A PR lands by merging its head into `main`, so GitHub marks it merged; fixes follow as commits.

- [x] G18 Pin: `scripts/check.sh` fails to resolve GPUI Kit 0.9.1, which pins `gpui-pre =0.3.12` (zed@d2a4fb6); Ely's three gpui pins follow, with an ADR, the docs that name the pin and a CHANGELOG line; two two-line comments on `main` become one line, so the gate passes again
- [x] G19 Issue #7: the date, range, time, date-time, week and period pickers' popups occlude, so a click on the calendar reaches nothing below; the issue's test lands
- [x] G20 Issue #15: gpui-pre 0.3.12 has no disabled setter but hands an element its own AccessKit node through `a11y_synthetic_children`; every disabled control sets the flag there, so UIA reports `IsEnabled=false`; the Windows audit checks a disabled `Button`
- [x] G21 PR #5: Enter and Space tests for `Button` and `IconButton` merged over 111 commits of `main`, the import conflict resolved
- [x] G22 PR #6: `as_chunks` already on `main`, its first commit is PR #5's; `NetworkGraph::on_node_click` lands with the drag log kept, API snapshot and CHANGELOG line
- [x] G23 PR #17: MenuBar shows a checked action's mark and honours its disabled flag, tested; its sync-then-deferred double dispatch replaced by one dispatch at the root cause; zh-TW gets a gallery catalog and the i18n docs; `EditorTabs::on_empty` and the 32px title bar land with API snapshot and CHANGELOG lines
- [x] G24 Push; a short reply with thanks on #7, #15, #5, #6 and #17; close #7 and #15
- [x] G25 CI: `deny` fails on RUSTSEC-2026-0486, unsound xattr functions in rustix 1.1.5; `Cargo.lock` takes rustix 1.1.6, the fixed release
