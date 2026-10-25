# 0045 Incremental check times

Date: 2026-10-23. Status: accepted.

After one edited line, `cargo check -p ely-core` takes 1.3 s incremental and 3.5–4.0 s with `CARGO_INCREMENTAL=0` on the WSL box, 2 or 32 CPUs; the single crate at `17f936b` took 3.4–3.7 s and 11–13 s there (OPTIMIZATION.md O-26, `tasks/optimization.md` B10).
