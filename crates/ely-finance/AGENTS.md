# AGENTS.md: ely-finance

Rules for `ely-finance`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- Rise reads green and fall red unless a component's `red_up` swaps them, as markets in East Asia read. `finance::quotes::moves` picks the pair.
