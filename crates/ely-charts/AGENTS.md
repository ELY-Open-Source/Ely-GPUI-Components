# AGENTS.md: ely-charts

Rules for `ely-charts`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- Charts lay out in `f32` pixels of their own box, tested without gpui, and paint through `charts::paint::at`: a canvas's paint gets window coordinates, and `with_element_offset` works only in prepaint. Their sizes come from `theme.chart()`. A chart value is finite and within ±`charts::LIMIT` (1e30), so sums of values and their pixels stay inside `f32`; the builders refuse others, and a scale works in halves so a domain across the limit maps.
- A label pinned to a point sits in a zero-size absolute box that centers it with flex (`charts::axes`), so no offset guesses at text size.
