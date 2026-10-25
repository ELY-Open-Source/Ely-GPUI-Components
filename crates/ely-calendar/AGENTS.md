# AGENTS.md: ely-calendar

Rules for `ely-calendar`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- An event repeats as RFC 5545 has it: weeks count from the start's own week, and a month or a year without the start's date is passed over (`calendar::Recurrence`).
