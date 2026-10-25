# 0052 The library crates are publishable

Date: 2026-10-24. Status: accepted.

The 31 library crates are publishable (OPTIMIZATION.md O-13, E2): `publish = false` leaves all but the gallery; description, license, repository, homepage, readme, keywords and category come from the workspace; the root crate packages `src`, `tests`, `benches`, the README and the license alone; each member crate holds a copy of `LICENSE`, since cargo packages nothing from outside a crate, and `scripts/check.sh` fails a copy that differs. Every name was free on crates.io that day. Publishing itself, which crates.io cannot undo, waits for the owner.
