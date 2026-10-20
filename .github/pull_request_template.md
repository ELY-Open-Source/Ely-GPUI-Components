<!--
Title: start with the type, for example
  [FEATURE REQUEST] Add dark mode to the settings page
  [BUG REPORT] Fix a crash when opening an empty file
  [Other] Update the README install steps
-->

## Type

- [ ] **[FEATURE REQUEST]**: a new feature or an improvement
- [ ] **[BUG REPORT]**: a bug fix
- [ ] **[Other]**: docs, refactoring, CI, dependencies or chores

## What and why

<!-- One or two sentences. Link the issue: Closes #123 -->

## Bug details

<!-- [BUG REPORT] only; delete this section otherwise. -->

- Steps to reproduce:
- Expected:
- Actual:
- Cause and fix:

## How it was tested

<!-- Commands you ran, and what you checked by hand. -->

## Screenshots

<!-- For anything that draws: light and dark, and 280px wide if it holds a row or header. -->

## Checklist

- [ ] The title starts with `[FEATURE REQUEST]`, `[BUG REPORT]` or `[Other]`
- [ ] `scripts/check.sh` passes
- [ ] The gallery shows it: a story on its chapter's page, and `python3 scripts/stories.py` rerun
- [ ] Keyboard works: Tab reaches it, focus ring shows, Enter and Space press it
- [ ] Reduced motion holds its motion still
- [ ] `AGENTS.md` names any new rule or dependency
- [ ] For `frontend/`: `pnpm test`, `pnpm build` and `pnpm e2e` pass
