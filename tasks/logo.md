# Logo round, 2026-10-04

The site's mark: the letter E in blocks, drawn by the site's own iso engine, its middle arm in the accent. The user chose it from four, and asked for it in the header, the favicon and touch icon, and the OG image.

- [x] L1 Plan: these lines, the T52 line in `TASKS.md`
- [x] L2 Mark: one geometry in `src/iso/mark.ts`; a `Mark` in the header before "Ely", in the page's colors, light and dark
- [x] L3 Favicon: `scripts/favicon.mjs` writes `public/favicon.svg`, which follows the system's mode, and a 180px `apple-touch-icon.png`; `index.html` links both
- [x] L4 OG: `scripts/og.mjs` keeps the header's mark and name; `public/og.png` rendered again
- [x] L5 Tests and docs: unit and Playwright tests; AGENTS.md; acceptance, item by item
