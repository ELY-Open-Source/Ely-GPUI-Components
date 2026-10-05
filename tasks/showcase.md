# Showcase round, 2026-10-04

An apps page like gpui-kit.com/apps, fed by a public repo, `ZacharyZhang-NY/Ely-GPUI-Showcases`.
Choices, the user's: the catalog starts empty; the site fetches the repo at a pinned commit when it builds; categories and search, stars from CI, a detail page from the app's README, a featured order.

- [x] S1 Plan: these lines, the T51 lines in `TASKS.md`
- [x] S2 Repo · Catalog: `apps/<id>/manifest.json`, `featured.json`, README, CONTRIBUTING; `scripts/validate.mjs` with tests
- [x] S3 Repo · CI: validate on every push and pull request; stars refreshed on main and weekly, `scripts/stars.mjs` with tests
- [x] S4 Repo · Publish: the public repo on GitHub, pushed, CI green
- [x] S5 Site · Data: `frontend/showcase.lock` pins a commit; `scripts/showcase.mjs` fetches it, runs the repo's validator, renders READMEs, writes the catalog; dev, build and CI run it
- [x] S6 Site · List: `/showcase/`, a Showcase link in the header, featured first, the rest newest then stars, categories with counts, search, the empty state, Submit
- [x] S7 Site · Detail: `/showcase/$app/`, previews, links, platforms, stars, the README
- [x] S8 Site · Tests: unit tests for order and filters; Playwright for the page, the empty state and the link
- [x] S9 Docs: AGENTS.md; acceptance, item by item
