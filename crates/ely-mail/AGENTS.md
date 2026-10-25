# AGENTS.md: ely-mail

Rules for `ely-mail`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- A menu of times goes through `mail::times`: each row names its day and hour, a moment comes once (tomorrow is next Monday on a Sunday), and "Pick a time…" opens one dialog that waits for a time ahead of now (`ScheduleSend`, `SnoozePicker`).
